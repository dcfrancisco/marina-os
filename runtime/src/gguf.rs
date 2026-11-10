//! Bounded, non-loading GGUF metadata inspection for discovery.

use std::collections::{HashMap, HashSet};
use std::io::{self, Read, Seek};
use std::path::Path;

#[derive(Default)]
pub(crate) struct DiscoveryFacts {
    pub architecture: Option<String>,
    pub context: Option<u64>,
    pub quantization: Option<&'static str>,
    pub experts: Option<oid_model_runner::ExpertInfo>,
    block_count: Option<u64>,
    embedding_length: Option<u64>,
    expert_metadata_present: bool,
    alignment: Option<u64>,
    attention: HashMap<String, u64>,
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
    read_metadata(&mut reader).map(|(facts, _)| facts)
}

#[allow(clippy::too_many_lines)]
fn read_metadata<R: Read>(mut reader: R) -> io::Result<(DiscoveryFacts, u64)> {
    let mut magic = [0; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"GGUF" || !matches!(number(&mut reader, 4)?, 2 | 3) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported GGUF header",
        ));
    }
    let tensors = number(&mut reader, 8)?;
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
            || (key.contains(".attention.") && matches!(kind, 0 | 2 | 4 | 10))
            || key == "general.alignment"
            || (key == "general.file_type" && kind == 4)
            || (key.ends_with(".context_length") && matches!(kind, 0 | 2 | 4 | 10))
            || (key.ends_with(".block_count") && matches!(kind, 0 | 2 | 4 | 10))
            || (key.ends_with(".embedding_length") && matches!(kind, 0 | 2 | 4 | 10))
            || key.ends_with(".expert_count")
            || key.ends_with(".expert_used_count")
        {
            let data = if (key == "general.alignment" && kind != 4)
                || ((key.ends_with(".expert_count") || key.ends_with(".expert_used_count"))
                    && !matches!(kind, 0 | 2 | 4 | 10))
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
    Ok((
        DiscoveryFacts {
            attention: entries
                .iter()
                .filter_map(|(key, value)| value.as_deref()?.parse().ok().map(|n| (key.clone(), n)))
                .collect(),
            alignment: match entries.iter().find(|(key, _)| key == "general.alignment") {
                None => Some(32),
                Some((_, value)) => value
                    .as_deref()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|v| v.is_power_of_two()),
            },
            architecture: architecture.clone(),
            context,
            block_count: expert_value("block_count").and_then(|v| v.parse().ok()),
            embedding_length: expert_value("embedding_length").and_then(|v| v.parse().ok()),
            expert_metadata_present: expert_value("expert_count").is_some()
                || expert_value("expert_used_count").is_some(),
            experts: Some(experts),
            quantization: entries
                .iter()
                .find(|(key, _)| key == "general.file_type")
                .and_then(|(_, v)| v.as_ref())
                .and_then(|v| v.parse::<u64>().ok())
                .and_then(file_type),
        },
        tensors,
    ))
}

pub(crate) fn inspect_tensors(path: &Path) -> io::Result<Vec<crate::models::GgufTensorInfo>> {
    inspect_tensor_evidence(path).map(|(_, tensors)| tensors)
}

pub(crate) fn inspect_tensor_evidence(
    path: &Path,
) -> io::Result<(DiscoveryFacts, Vec<crate::models::GgufTensorInfo>)> {
    let (facts, tensors, _, _) = read_tensor_headers(path)?;
    Ok((facts, tensors))
}

fn read_tensor_headers(
    path: &Path,
) -> io::Result<(DiscoveryFacts, Vec<crate::models::GgufTensorInfo>, u64, u64)> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file.take(32 * 1_048_576));
    let (facts, count) = read_metadata(&mut reader)?;
    if count > 100_000 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "too many GGUF tensors",
        ));
    }
    let mut tensors = Vec::new();
    let mut names = HashSet::new();
    for _ in 0..count {
        let name = string(&mut reader)?;
        if name.is_empty() || !names.insert(name.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "empty or duplicate GGUF tensor name",
            ));
        }
        let dimensions = number(&mut reader, 4)?;
        if !(1..=4).contains(&dimensions) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid GGUF tensor dimensions",
            ));
        }
        let mut shape = Vec::new();
        let mut elements = 1_u64;
        for _ in 0..dimensions {
            let size = number(&mut reader, 8)?;
            elements = elements
                .checked_mul(size)
                .filter(|n| *n > 0)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid GGUF tensor shape")
                })?;
            shape.push(size);
        }
        let tensor_type = u32::try_from(number(&mut reader, 4)?).expect("four bytes fit in u32");
        let offset = number(&mut reader, 8)?;
        tensors.push(crate::models::GgufTensorInfo {
            name,
            shape,
            tensor_type,
            offset,
        });
    }
    let header_end = reader.stream_position()?;
    let file_len = reader.get_ref().get_ref().metadata()?.len();
    Ok((facts, tensors, header_end, file_len))
}

pub(crate) fn kv_bytes_per_token(path: &Path) -> io::Result<Option<u64>> {
    let facts = inspect(path)?;
    Ok(estimate_kv(&facts))
}

// Full-attention F16 K/V only. Hybrid, recurrent and sliding-window layouts
// require their own allocation model and intentionally remain unknown.
fn estimate_kv(facts: &DiscoveryFacts) -> Option<u64> {
    let arch = facts.architecture.as_deref()?;
    if !matches!(arch, "qwen2" | "llama") {
        return None;
    }
    let get = |suffix: &str| {
        facts
            .attention
            .get(&format!("{arch}.attention.{suffix}"))
            .copied()
            .filter(|n| *n > 0)
    };
    let heads = get("head_count")?;
    let kv_heads = get("head_count_kv")?;
    if kv_heads > heads {
        return None;
    }
    let embedding = facts.embedding_length?;
    if embedding == 0 || embedding % heads != 0 {
        return None;
    }
    let dimension = |suffix: &str| match facts.attention.get(&format!("{arch}.attention.{suffix}"))
    {
        Some(0) => None,
        Some(value) => Some(*value),
        None => Some(embedding / heads),
    };
    let key = dimension("key_length")?;
    let value = dimension("value_length")?;
    key.checked_add(value)?
        .checked_mul(kv_heads)?
        .checked_mul(facts.block_count.filter(|n| *n > 0)?.checked_mul(2)?)
}

/// Sizes from the pinned ggml type definitions; unsupported types stay unknown.
fn tensor_block(kind: u32) -> Option<(u64, u64)> {
    // ggml.h and ggml-common.h at e4b6c8ca; checked against installed ggml type metadata.
    match kind {
        0 | 26 => Some((1, 4)),
        1 | 25 | 30 => Some((1, 2)),
        2 | 20 => Some((32, 18)),
        3 => Some((32, 20)),
        6 => Some((32, 22)),
        7 => Some((32, 24)),
        8 => Some((32, 34)),
        9 => Some((32, 36)),
        10 => Some((256, 84)),
        11 | 21 => Some((256, 110)),
        12 => Some((256, 144)),
        13 => Some((256, 176)),
        14 => Some((256, 210)),
        15 => Some((256, 292)),
        16 | 35 => Some((256, 66)),
        17 => Some((256, 74)),
        18 => Some((256, 98)),
        19 => Some((256, 50)),
        22 => Some((256, 82)),
        23 => Some((256, 136)),
        24 | 43 => Some((1, 1)),
        27 | 28 => Some((1, 8)),
        29 => Some((256, 56)),
        34 => Some((256, 54)),
        39 => Some((32, 17)),
        40 => Some((64, 36)),
        41 => Some((128, 18)),
        42 => Some((64, 18)),
        _ => None,
    }
}

pub(crate) fn inspect_storage(path: &Path) -> io::Result<crate::models::GgufStorageInfo> {
    let (facts, tensors, header_end, file_len) = read_tensor_headers(path)?;
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid GGUF tensor storage extent",
        )
    };
    let alignment = facts.alignment.ok_or_else(invalid)?;
    let data_start = header_end.checked_add(alignment - 1).ok_or_else(invalid)? & !(alignment - 1);
    if data_start > file_len {
        return Err(invalid());
    }
    let mut total_elements = 0_u64;
    let mut total_bytes = Some(0_u64);
    let mut extents = Vec::with_capacity(tensors.len());
    for tensor in &tensors {
        let elements = tensor
            .shape
            .iter()
            .try_fold(1_u64, |a, b| a.checked_mul(*b))
            .ok_or_else(invalid)?;
        total_elements = total_elements.checked_add(elements).ok_or_else(invalid)?;
        if tensor.offset % alignment != 0 {
            return Err(invalid());
        }
        let start = data_start.checked_add(tensor.offset).ok_or_else(invalid)?;
        if start >= file_len {
            return Err(invalid());
        }
        let size = if let Some((block, bytes)) = tensor_block(tensor.tensor_type) {
            if tensor.shape[0] % block != 0 {
                return Err(invalid());
            }
            Some((elements / block).checked_mul(bytes).ok_or_else(invalid)?)
        } else {
            None
        };
        let end = size
            .map(|n| start.checked_add(n).ok_or_else(invalid))
            .transpose()?;
        if end.is_some_and(|end| end > file_len) {
            return Err(invalid());
        }
        total_bytes = match (total_bytes, size) {
            (Some(a), Some(b)) => Some(a.checked_add(b).ok_or_else(invalid)?),
            _ => None,
        };
        extents.push((start, end));
    }
    extents.sort_unstable_by_key(|extent| extent.0);
    for pair in extents.windows(2) {
        if pair[0].0 == pair[1].0 || pair[0].1.is_some_and(|end| end > pair[1].0) {
            return Err(invalid());
        }
    }
    Ok(crate::models::GgufStorageInfo {
        tensor_count: tensors.len(),
        elements: total_elements,
        tensor_bytes: total_bytes,
        data_offset: data_start,
        file_bytes: file_len,
    })
}

/// Gemma 4 FFN structure checked against llama.cpp e4b6c8ca6a6e8da70248da30709e897e7cc8615e.
/// This classifies header evidence only, never serving/backend compatibility.
pub(crate) fn classify_experts(
    facts: DiscoveryFacts,
    tensors: &[crate::models::GgufTensorInfo],
) -> oid_model_runner::ExpertInfo {
    use oid_model_runner::ExpertInfo;
    let declared = facts
        .experts
        .unwrap_or_else(|| ExpertInfo::from_metadata(None, None));
    if facts.architecture.as_deref() != Some("gemma4") {
        return declared;
    }
    // Malformed or partial expert declarations must not be rescued by tensor names.
    if facts.expert_metadata_present && declared.kind == "unknown" {
        return declared;
    }
    let unknown = || ExpertInfo {
        kind: "unknown",
        expert_count: None,
        experts_per_token: None,
        explanation: Some("Gemma 4 FFN tensor evidence is incomplete or conflicts with metadata"),
    };
    let (Some(blocks @ 1..=100_000), Some(embedding @ 1..)) =
        (facts.block_count, facts.embedding_length)
    else {
        return unknown();
    };
    let shapes: HashMap<_, _> = tensors
        .iter()
        .map(|tensor| (tensor.name.as_str(), tensor.shape.as_slice()))
        .collect();
    let mut recognized = HashSet::new();
    let mut routed_count = None;
    for layer in 0..blocks {
        let shape = |suffix: &str| {
            shapes
                .get(format!("blk.{layer}.{suffix}.weight").as_str())
                .copied()
        };
        // Every Gemma 4 layer has a dense FFN, also used as the shared expert.
        let Some([input, ff]) = shape("ffn_gate") else {
            return unknown();
        };
        if *input != embedding
            || shape("ffn_up") != Some(&[embedding, *ff])
            || shape("ffn_down") != Some(&[*ff, embedding])
        {
            return unknown();
        }
        if let Some(router) = shape("ffn_gate_inp") {
            let [input, count] = router else {
                return unknown();
            };
            let Ok(count32) = u32::try_from(*count) else {
                return unknown();
            };
            if *input != embedding
                || count32 == 0
                || routed_count.is_some_and(|previous| previous != count32)
            {
                return unknown();
            }
            routed_count = Some(count32);
            if !valid_expert_shapes(&shape, embedding, *count) {
                return unknown();
            }
            for suffix in [
                "ffn_gate_inp",
                "ffn_gate_exps",
                "ffn_up_exps",
                "ffn_gate_up_exps",
                "ffn_down_exps",
            ] {
                if shape(suffix).is_some() {
                    recognized.insert(format!("blk.{layer}.{suffix}.weight"));
                }
            }
        }
    }
    // Orphaned, out-of-range or unrecognized expert/router names defeat dense inference.
    if tensors.iter().any(|tensor| {
        ["expert", "exps", "router", "ffn_gate_inp"]
            .iter()
            .any(|marker| tensor.name.contains(marker))
            && !recognized.contains(&tensor.name)
            && !tensor
                .name
                .strip_suffix(".scale")
                .is_some_and(|name| recognized.contains(&format!("{name}.weight")))
    }) {
        return unknown();
    }
    if let Some(count) = declared.expert_count {
        if count != routed_count.unwrap_or(0) {
            return unknown();
        }
        return declared;
    }
    ExpertInfo {
        kind: if routed_count.is_some() {
            "moe"
        } else {
            "dense"
        },
        // Counts remain metadata facts; shapes alone do not declare routing settings.
        expert_count: None,
        experts_per_token: None,
        explanation: Some("Gemma 4 FFN tensor structure; expert routing counts are not declared"),
    }
}

fn valid_expert_shapes<'a>(
    shape: &impl Fn(&str) -> Option<&'a [u64]>,
    embedding: u64,
    count: u64,
) -> bool {
    let Some([ff, output, experts]) = shape("ffn_down_exps") else {
        return false;
    };
    if *output != embedding || *experts != count {
        return false;
    }
    let separate = shape("ffn_gate_exps") == Some(&[embedding, *ff, count])
        && shape("ffn_up_exps") == Some(&[embedding, *ff, count]);
    let fused = ff
        .checked_mul(2)
        .is_some_and(|width| shape("ffn_gate_up_exps") == Some(&[embedding, width, count]));
    separate || fused
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

    #[test]
    fn tensor_descriptors_are_bounded_and_reject_malformed_headers() {
        let path =
            std::env::temp_dir().join(format!("talvik-gguf-tensors-{}.gguf", std::process::id()));
        let header = |count: u64| {
            let mut bytes = b"GGUF".to_vec();
            bytes.extend_from_slice(&3_u32.to_le_bytes());
            bytes.extend_from_slice(&count.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
            bytes
        };
        let descriptor = |bytes: &mut Vec<u8>, name: &str, shape: &[u64]| {
            text(bytes, name);
            bytes.extend_from_slice(&u32::try_from(shape.len()).unwrap().to_le_bytes());
            for size in shape {
                bytes.extend_from_slice(&size.to_le_bytes());
            }
            bytes.extend_from_slice(&999_u32.to_le_bytes());
            bytes.extend_from_slice(&64_u64.to_le_bytes());
        };
        let mut valid = header(1);
        descriptor(&mut valid, "blk.0.ffn_gate_inp.weight", &[4, 8]);
        std::fs::write(&path, &valid).unwrap();
        let tensors = crate::models::inspect_artifact_tensors(&path).unwrap();
        assert_eq!(tensors.len(), 1);
        assert_eq!(tensors[0].name, "blk.0.ffn_gate_inp.weight");
        assert_eq!(tensors[0].shape, [4, 8]);
        assert_eq!((tensors[0].tensor_type, tensors[0].offset), (999, 64));
        valid.pop();
        std::fs::write(&path, valid).unwrap();
        assert!(crate::models::inspect_artifact_tensors(&path).is_err());

        let mut duplicate = header(2);
        descriptor(&mut duplicate, "same", &[1]);
        descriptor(&mut duplicate, "same", &[1]);
        let mut zero = header(1);
        descriptor(&mut zero, "zero", &[0]);
        let mut overflow = header(1);
        descriptor(&mut overflow, "overflow", &[u64::MAX, 2]);
        let mut dimensions = header(1);
        descriptor(&mut dimensions, "dimensions", &[1, 1, 1, 1, 1]);
        for invalid in [header(100_001), duplicate, zero, overflow, dimensions] {
            std::fs::write(&path, invalid).unwrap();
            assert!(crate::models::inspect_artifact_tensors(&path).is_err());
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn storage_checks_dense_moe_and_invalid_extents() {
        let path = std::env::temp_dir().join(format!("talvik-storage-{}.gguf", std::process::id()));
        let fixture = |descriptors: &[(&str, &[u64], u32, u64)], data_len: usize| {
            let mut bytes = b"GGUF".to_vec();
            bytes.extend_from_slice(&3_u32.to_le_bytes());
            bytes.extend_from_slice(&u64::try_from(descriptors.len()).unwrap().to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
            for (name, shape, kind, offset) in descriptors {
                text(&mut bytes, name);
                bytes.extend_from_slice(&u32::try_from(shape.len()).unwrap().to_le_bytes());
                for dim in *shape {
                    bytes.extend_from_slice(&dim.to_le_bytes());
                }
                bytes.extend_from_slice(&kind.to_le_bytes());
                bytes.extend_from_slice(&offset.to_le_bytes());
            }
            let aligned = (bytes.len() + 31) & !31;
            bytes.resize(aligned + data_len, 0);
            bytes
        };
        let check = |descriptors: &[(&str, &[u64], u32, u64)], len| {
            std::fs::write(&path, fixture(descriptors, len)).unwrap();
            crate::models::inspect_artifact_storage(&path)
        };
        let dense = [("dense", &[32, 2][..], 2, 0)];
        let info = check(&dense, 36).unwrap();
        assert_eq!((info.elements, info.tensor_bytes), (64, Some(36)));
        assert!(check(&dense, 35).is_err());
        let moe = [
            ("router", &[4, 8][..], 0, 0),
            ("experts", &[32, 2, 8][..], 2, 128),
        ];
        let info = check(&moe, 416).unwrap();
        assert_eq!((info.elements, info.tensor_bytes), (544, Some(416)));
        assert!(check(&[("bad", &[31][..], 2, 0)], 32).is_err());
        assert!(check(&[("bad", &[1][..], 0, 1)], 32).is_err());
        assert!(check(&[("bad", &[1][..], 0, u64::MAX - 31)], 32).is_err());
        assert!(check(&[("a", &[16][..], 0, 0), ("b", &[1][..], 0, 32)], 100).is_err());
        assert_eq!(
            check(&[("unknown", &[1][..], 999, 0)], 1)
                .unwrap()
                .tensor_bytes,
            None
        );
        assert!(check(&[("huge", &[u64::MAX][..], 0, 0)], 32).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn all_pinned_types_have_storage_sizes_and_removed_types_stay_unknown() {
        for kind in (0..44).filter(|k| ![4, 5, 31, 32, 33, 36, 37, 38].contains(k)) {
            assert!(super::tensor_block(kind).is_some(), "type {kind}");
        }
        for kind in [4, 5, 31, 32, 33, 36, 37, 38, 44, u32::MAX] {
            assert_eq!(super::tensor_block(kind), None);
        }
        assert_eq!(super::tensor_block(16), Some((256, 66)));
        assert_eq!(super::tensor_block(34), Some((256, 54)));
        assert_eq!(super::tensor_block(40), Some((64, 36)));
    }

    #[test]
    fn kv_estimate_requires_supported_consistent_attention_metadata() {
        let mut facts = super::DiscoveryFacts {
            architecture: Some("qwen2".into()),
            block_count: Some(24),
            embedding_length: Some(896),
            ..Default::default()
        };
        facts
            .attention
            .insert("qwen2.attention.head_count".into(), 14);
        assert_eq!(super::estimate_kv(&facts), None);
        facts
            .attention
            .insert("qwen2.attention.head_count_kv".into(), 2);
        assert_eq!(super::estimate_kv(&facts), Some(12288));
        facts
            .attention
            .insert("qwen2.attention.key_length".into(), 0);
        assert_eq!(super::estimate_kv(&facts), None);
        facts.attention.remove("qwen2.attention.key_length");
        facts.block_count = Some(0);
        assert_eq!(super::estimate_kv(&facts), None);
        facts.block_count = Some(u64::MAX);
        assert_eq!(super::estimate_kv(&facts), None);
        facts.architecture = Some("gemma4".into());
        assert_eq!(super::estimate_kv(&facts), None);
    }

    #[test]
    fn storage_rejects_invalid_alignment_metadata() {
        let path =
            std::env::temp_dir().join(format!("talvik-alignment-{}.gguf", std::process::id()));
        for (kind, alignment) in [(4_u32, 0_u32), (4, 3), (5, 32)] {
            let mut bytes = b"GGUF".to_vec();
            bytes.extend_from_slice(&3_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
            bytes.extend_from_slice(&1_u64.to_le_bytes());
            text(&mut bytes, "general.alignment");
            bytes.extend_from_slice(&kind.to_le_bytes());
            bytes.extend_from_slice(&alignment.to_le_bytes());
            bytes.resize(128, 0);
            std::fs::write(&path, bytes).unwrap();
            assert!(crate::models::inspect_artifact_storage(&path).is_err());
        }
        std::fs::remove_file(path).unwrap();
    }

    fn expert_fixture(
        architecture: &str,
        metadata: &[(&str, u32, u64)],
        tensors: &[(&str, &[u64])],
    ) -> Vec<u8> {
        let mut bytes = b"GGUF".to_vec();
        bytes.extend_from_slice(&3_u32.to_le_bytes());
        bytes.extend_from_slice(&u64::try_from(tensors.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(&u64::try_from(metadata.len() + 1).unwrap().to_le_bytes());
        text(&mut bytes, "general.architecture");
        bytes.extend_from_slice(&8_u32.to_le_bytes());
        text(&mut bytes, architecture);
        for (key, kind, value) in metadata {
            text(&mut bytes, key);
            bytes.extend_from_slice(&kind.to_le_bytes());
            if *kind == 8 {
                text(&mut bytes, &value.to_string());
            } else {
                bytes.extend_from_slice(&value.to_le_bytes()[..if *kind == 10 { 8 } else { 4 }]);
            }
        }
        for (name, shape) in tensors {
            text(&mut bytes, name);
            bytes.extend_from_slice(&u32::try_from(shape.len()).unwrap().to_le_bytes());
            for size in *shape {
                bytes.extend_from_slice(&size.to_le_bytes());
            }
            bytes.extend_from_slice(&0_u32.to_le_bytes());
            bytes.extend_from_slice(&0_u64.to_le_bytes());
        }
        bytes
    }

    #[test]
    fn gemma_classification_requires_complete_consistent_ffn_evidence() {
        let path =
            std::env::temp_dir().join(format!("talvik-gemma-evidence-{}.gguf", std::process::id()));
        let metadata = [
            ("gemma4.block_count", 4, 1),
            ("gemma4.embedding_length", 4, 4),
        ];
        let dense: Vec<(&str, &[u64])> = vec![
            ("blk.0.ffn_gate.weight", &[4, 6]),
            ("blk.0.ffn_up.weight", &[4, 6]),
            ("blk.0.ffn_down.weight", &[6, 4]),
        ];
        let mut moe = dense.clone();
        moe.extend([
            ("blk.0.ffn_gate_inp.weight", &[4, 8][..]),
            ("blk.0.ffn_gate_up_exps.weight", &[4, 12, 8][..]),
            ("blk.0.ffn_down_exps.weight", &[6, 4, 8][..]),
        ]);
        let inspect_case = |arch, metadata: &[(&str, u32, u64)], tensors: &[(&str, &[u64])]| {
            std::fs::write(&path, expert_fixture(arch, metadata, tensors)).unwrap();
            crate::models::inspect_artifact_expert_evidence(&path).unwrap()
        };
        let info = inspect_case("gemma4", &metadata, &dense);
        assert_eq!(info.kind, "dense");
        assert_eq!(info.expert_count, None);
        assert_eq!(inspect_case("gemma4", &metadata, &moe).kind, "moe");
        let mut declared = metadata.to_vec();
        declared.extend([
            ("gemma4.expert_count", 4, 8),
            ("gemma4.expert_used_count", 4, 2),
        ]);
        let info = inspect_case("gemma4", &declared, &moe);
        assert_eq!(
            (info.kind, info.expert_count, info.experts_per_token),
            ("moe", Some(8), Some(2))
        );
        assert_eq!(inspect_case("gemma4", &declared, &dense).kind, "unknown");
        for (kind, count) in [(4, 0), (4, 7), (8, 8), (10, u64::MAX)] {
            let mut invalid = metadata.to_vec();
            invalid.push(("gemma4.expert_count", kind, count));
            assert_eq!(inspect_case("gemma4", &invalid, &moe).kind, "unknown");
        }
        assert_eq!(
            inspect_case("unrecognized", &metadata, &dense).kind,
            "unknown"
        );
        assert_eq!(inspect_case("gemma4", &[], &dense).kind, "unknown");
        assert_eq!(
            inspect_case("gemma4", &metadata, &dense[..2]).kind,
            "unknown"
        );
        assert_eq!(inspect_case("gemma4", &metadata, &moe[..4]).kind, "unknown");
        let mut malformed_shape = moe.clone();
        malformed_shape[3].1 = &[8, 4];
        assert_eq!(
            inspect_case("gemma4", &metadata, &malformed_shape).kind,
            "unknown"
        );
        let mut orphan = dense.clone();
        orphan.push(("blk.1.ffn_gate_inp.weight", &[4, 8]));
        assert_eq!(inspect_case("gemma4", &metadata, &orphan).kind, "unknown");
        // Metadata remains useful for architectures without a verified tensor rule.
        for (count, expected) in [(0, "dense"), (8, "moe")] {
            assert_eq!(
                inspect_case("other", &[("other.expert_count", 4, count)], &[]).kind,
                expected
            );
        }
        std::fs::write(&path, b"GGUF").unwrap();
        assert!(crate::models::inspect_artifact_expert_evidence(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn expert_inspection_preserves_discovery_and_registry_metadata() {
        use crate::models::{ModelDiscovery, ModelRegistry};
        let directory =
            std::env::temp_dir().join(format!("talvik-expert-discovery-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("small.gguf");
        std::fs::write(
            &path,
            expert_fixture(
                "qwen2",
                &[
                    ("qwen2.expert_count", 4, 0),
                    ("qwen2.context_length", 4, 4096),
                    ("general.file_type", 4, 15),
                ],
                &[],
            ),
        )
        .unwrap();
        std::fs::write(directory.join("broken.gguf"), b"GGUF").unwrap();
        let registry = ModelRegistry::in_memory(oid_shared::EventBus::new());
        let discovery = ModelDiscovery::new(vec![directory.clone()]);
        assert_eq!(discovery.discover(&registry, "llama.cpp").unwrap().len(), 2);
        let before = registry.inspect("small").unwrap();
        assert_eq!(
            crate::models::inspect_artifact_expert_evidence(&path)
                .unwrap()
                .kind,
            "dense"
        );
        assert_eq!(registry.inspect("small").unwrap(), before);
        assert_eq!(
            (
                before.family.as_str(),
                before.quantization.as_str(),
                before.context_window
            ),
            ("qwen2", "Q4_K_M", 4096)
        );
        assert!(discovery
            .discover(&registry, "llama.cpp")
            .unwrap()
            .is_empty());
        assert_eq!(registry.inspect("broken").unwrap().family, "unknown");
        std::fs::remove_dir_all(directory).unwrap();
    }

    /// Run explicitly with a caller-supplied local model; never acquires one.
    #[test]
    #[ignore = "requires TALVIK_GGUF_INSPECT_PATH pointing to a local GGUF"]
    fn inspect_local_artifact_descriptors() {
        let path = std::env::var_os("TALVIK_GGUF_INSPECT_PATH")
            .expect("set TALVIK_GGUF_INSPECT_PATH to a local GGUF");
        let path = std::path::Path::new(&path);
        let facts = inspect(path).expect("inspect metadata");
        let tensors = crate::models::inspect_artifact_tensors(path).expect("inspect tensors");
        assert!(!tensors.is_empty(), "expected tensor descriptors");
        println!(
            "storage={:?}",
            crate::models::inspect_artifact_storage(path).expect("inspect storage")
        );
        let markers: Vec<_> = tensors
            .iter()
            .filter(|tensor| {
                ["expert", "exps", "router", "ffn_gate_inp"]
                    .iter()
                    .any(|marker| tensor.name.contains(marker))
            })
            .map(|tensor| tensor.name.as_str())
            .collect();
        println!(
            "architecture={:?} quantization={:?} experts={:?} tensors={} marker_names={markers:?}",
            facts.architecture,
            facts.quantization,
            crate::models::inspect_artifact_expert_evidence(path).expect("classify evidence"),
            tensors.len()
        );
    }
}
