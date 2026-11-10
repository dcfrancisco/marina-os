//! Bounded, non-loading GGUF metadata inspection for discovery.

use std::io::{self, Read};
use std::path::Path;

#[derive(Default)]
pub(crate) struct DiscoveryFacts {
    pub architecture: Option<String>,
    pub context: Option<u64>,
}

fn number<R: Read>(reader: &mut R, bytes: usize) -> io::Result<u64> {
    let mut value = [0; 8];
    reader.read_exact(&mut value[..bytes])?;
    Ok(u64::from_le_bytes(value))
}

fn string<R: Read>(reader: &mut R) -> io::Result<String> {
    let len = number(reader, 8)?;
    if len > 1_048_576 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "GGUF string too large",
        ));
    }
    let mut value = vec![0; usize::try_from(len).unwrap_or(0)];
    reader.read_exact(&mut value)?;
    String::from_utf8(value)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid GGUF UTF-8"))
}

fn value<R: Read>(reader: &mut R, kind: u32) -> io::Result<Option<String>> {
    let size = match kind {
        0 | 1 | 7 => 1,
        2 | 3 => 2,
        4..=6 => 4,
        10..=12 => 8,
        8 => return string(reader).map(Some),
        9 => {
            let element = u32::try_from(number(reader, 4)?).unwrap_or(u32::MAX);
            let count = number(reader, 8)?;
            if count > 1_000_000 || element == 9 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid GGUF array",
                ));
            }
            for _ in 0..count {
                value(reader, element)?;
            }
            return Ok(None);
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid GGUF type",
            ))
        }
    };
    let n = number(reader, size)?;
    Ok(matches!(kind, 0 | 2 | 4 | 10).then(|| n.to_string()))
}

pub(crate) fn inspect(path: &Path) -> io::Result<DiscoveryFacts> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file.take(32 * 1_048_576));
    let mut magic = [0; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"GGUF" || !matches!(number(&mut reader, 4)?, 2 | 3) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported GGUF header",
        ));
    }
    let _tensors = number(&mut reader, 8)?;
    let count = number(&mut reader, 8)?;
    if count > 10_000 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "too many GGUF keys",
        ));
    }
    let mut entries = Vec::new();
    for _ in 0..count {
        let key = string(&mut reader)?;
        let kind = u32::try_from(number(&mut reader, 4)?).unwrap_or(u32::MAX);
        let data = value(&mut reader, kind)?;
        if (key == "general.architecture" && kind == 8)
            || (key.ends_with(".context_length") && matches!(kind, 0 | 2 | 4 | 10))
        {
            entries.push((key, data));
        }
    }
    let architecture = entries
        .iter()
        .find(|(key, _)| key == "general.architecture")
        .and_then(|(_, v)| v.clone())
        .filter(|v| !v.is_empty());
    let context = architecture
        .as_ref()
        .and_then(|arch| {
            entries
                .iter()
                .find(|(key, _)| key == &format!("{arch}.context_length"))
        })
        .and_then(|(_, v)| v.as_ref())
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v > 0);
    Ok(DiscoveryFacts {
        architecture,
        context,
    })
}

#[cfg(test)]
mod tests {
    use super::inspect;

    fn text(out: &mut Vec<u8>, value: &str) {
        out.extend_from_slice(&(value.len() as u64).to_le_bytes());
        out.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn reads_metadata_and_rejects_truncation() {
        let path = std::env::temp_dir().join(format!("talvik-gguf-{}.gguf", std::process::id()));
        let mut bytes = b"GGUF".to_vec();
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        text(&mut bytes, "general.architecture");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        text(&mut bytes, "qwen2");
        text(&mut bytes, "qwen2.context_length");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&4096_u32.to_le_bytes());
        std::fs::write(&path, &bytes).unwrap();
        let facts = inspect(&path).unwrap();
        assert_eq!(facts.architecture.as_deref(), Some("qwen2"));
        assert_eq!(facts.context, Some(4096));
        bytes.pop();
        std::fs::write(&path, bytes).unwrap();
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
