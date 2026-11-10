//! Bounded, non-loading GGUF metadata inspection for discovery.

use std::collections::HashSet;
use std::io::{self, Read};
use std::path::Path;

#[derive(Default)]
pub(crate) struct DiscoveryFacts {
    pub architecture: Option<String>,
    pub context: Option<u64>,
    pub quantization: Option<&'static str>,
    pub experts: Option<oid_model_runner::ExpertInfo>,
}

// llama.cpp's GGUF general.file_type identifies the dominant tensor format;
// individual tensors may use different types.
fn file_type(value: u64) -> Option<&'static str> {
    match value {
        0 => Some("F32"),
        1 => Some("F16"),
        2 => Some("Q4_0"),
        3 => Some("Q4_1"),
        7 => Some("Q8_0"),
        8 => Some("Q5_0"),
        9 => Some("Q5_1"),
        10 => Some("Q2_K"),
        11 => Some("Q3_K_S"),
        12 => Some("Q3_K_M"),
        13 => Some("Q3_K_L"),
        14 => Some("Q4_K_S"),
        15 => Some("Q4_K_M"),
        16 => Some("Q5_K_S"),
        17 => Some("Q5_K_M"),
        18 => Some("Q6_K"),
        19 => Some("IQ2_XXS"),
        20 => Some("IQ2_XS"),
        21 => Some("Q2_K_S"),
        22 => Some("IQ3_XS"),
        23 => Some("IQ3_XXS"),
        24 => Some("IQ1_S"),
        25 => Some("IQ4_NL"),
        26 => Some("IQ3_S"),
        27 => Some("IQ3_M"),
        28 => Some("IQ2_S"),
        29 => Some("IQ2_M"),
        30 => Some("IQ4_XS"),
        31 => Some("IQ1_M"),
        32 => Some("BF16"),
        36 => Some("TQ1_0"),
        37 => Some("TQ2_0"),
        38 => Some("MXFP4_MOE"),
        39 => Some("NVFP4"),
        40 => Some("Q1_0"),
        41 => Some("Q2_0"),
        _ => None,
    }
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
            if count > 1_000_000 || element == 9 || element > 12 {
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
    let mut keys = HashSet::new();
    for _ in 0..count {
        let key = string(&mut reader)?;
        if !keys.insert(key.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "duplicate GGUF metadata key",
            ));
        }
        let kind = u32::try_from(number(&mut reader, 4)?).unwrap_or(u32::MAX);
        let data = value(&mut reader, kind)?;
        if (key == "general.architecture" && kind == 8)
            || (key == "general.file_type" && kind == 4)
            || (key.ends_with(".context_length") && matches!(kind, 0 | 2 | 4 | 10))
            || key.ends_with(".expert_count")
            || key.ends_with(".expert_used_count")
        {
            let data = if (key.ends_with(".expert_count") || key.ends_with(".expert_used_count"))
                && !matches!(kind, 0 | 2 | 4 | 10)
            {
                Some(String::new())
            } else {
                data
            };
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
    let expert_value = |suffix: &str| {
        architecture.as_ref().and_then(|arch| {
            entries
                .iter()
                .find(|(key, _)| key == &format!("{arch}.{suffix}"))
                .and_then(|(_, value)| value.as_deref())
        })
    };
    let experts = oid_model_runner::ExpertInfo::from_metadata(
        expert_value("expert_count"),
        expert_value("expert_used_count"),
    );
    Ok(DiscoveryFacts {
        architecture,
        context,
        experts: Some(experts),
        quantization: entries
            .iter()
            .find(|(key, _)| key == "general.file_type")
            .and_then(|(_, v)| v.as_ref())
            .and_then(|v| v.parse::<u64>().ok())
            .and_then(file_type),
    })
}

#[cfg(test)]
mod tests {
    use super::{file_type, inspect};

    #[test]
    fn known_file_types_are_distinct_from_unknown_values() {
        assert_eq!(file_type(2), Some("Q4_0"));
        assert_eq!(file_type(15), Some("Q4_K_M"));
        assert_eq!(file_type(1024), None);
        assert_eq!(file_type(4), None);
    }

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
        bytes.extend_from_slice(&3_u64.to_le_bytes());
        text(&mut bytes, "general.architecture");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        text(&mut bytes, "qwen2");
        text(&mut bytes, "qwen2.context_length");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&4096_u32.to_le_bytes());
        text(&mut bytes, "general.file_type");
        bytes.extend_from_slice(&4_u32.to_le_bytes());
        bytes.extend_from_slice(&2_u32.to_le_bytes());
        std::fs::write(&path, &bytes).unwrap();
        let facts = inspect(&path).unwrap();
        assert_eq!(facts.architecture.as_deref(), Some("qwen2"));
        assert_eq!(facts.context, Some(4096));
        assert_eq!(facts.quantization, Some("Q4_0"));
        bytes.pop();
        std::fs::write(&path, bytes).unwrap();
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_duplicate_keys_and_invalid_empty_array_types() {
        let path =
            std::env::temp_dir().join(format!("talvik-gguf-invalid-{}.gguf", std::process::id()));
        let mut bytes = b"GGUF".to_vec();
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&2_u64.to_le_bytes());
        for _ in 0..2 {
            text(&mut bytes, "general.architecture");
            bytes.extend_from_slice(&8_u32.to_le_bytes());
            text(&mut bytes, "test");
        }
        std::fs::write(&path, &bytes).unwrap();
        assert!(inspect(&path).is_err());
        bytes.truncate(24);
        bytes[16..24].copy_from_slice(&1_u64.to_le_bytes());
        text(&mut bytes, "invalid");
        bytes.extend_from_slice(&9_u32.to_le_bytes());
        bytes.extend_from_slice(&99_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        std::fs::write(&path, &bytes).unwrap();
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn expert_metadata_is_architecture_scoped_and_conservative() {
        let path =
            std::env::temp_dir().join(format!("talvik-gguf-experts-{}.gguf", std::process::id()));
        let mut bytes = b"GGUF".to_vec();
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&4_u64.to_le_bytes());
        for (key, n) in [
            ("other.expert_count", 0_u32),
            ("test.expert_count", 8),
            ("test.expert_used_count", 2),
        ] {
            text(&mut bytes, key);
            bytes.extend_from_slice(&4_u32.to_le_bytes());
            bytes.extend_from_slice(&n.to_le_bytes());
        }
        text(&mut bytes, "general.architecture");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        text(&mut bytes, "test");
        std::fs::write(&path, &bytes).unwrap();
        let info = crate::models::inspect_artifact_experts(&path).unwrap();
        assert_eq!(
            (info.kind, info.expert_count, info.experts_per_token),
            ("moe", Some(8), Some(2))
        );

        // A string-typed count is invalid even if its text looks numeric.
        let mut malformed = b"GGUF".to_vec();
        malformed.extend_from_slice(&3_u32.to_le_bytes());
        malformed.extend_from_slice(&0_u64.to_le_bytes());
        malformed.extend_from_slice(&2_u64.to_le_bytes());
        text(&mut malformed, "general.architecture");
        malformed.extend_from_slice(&8_u32.to_le_bytes());
        text(&mut malformed, "test");
        text(&mut malformed, "test.expert_count");
        malformed.extend_from_slice(&8_u32.to_le_bytes());
        text(&mut malformed, "8");
        std::fs::write(&path, malformed).unwrap();
        assert_eq!(
            crate::models::inspect_artifact_experts(&path).unwrap().kind,
            "unknown"
        );
        std::fs::remove_file(path).unwrap();
    }
}
