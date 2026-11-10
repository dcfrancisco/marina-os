//! Bounded, non-loading GGUF metadata inspection for discovery.

use std::collections::{HashMap, HashSet};
use std::io::{self, Read};
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
            || (key == "general.file_type" && kind == 4)
            || (key.ends_with(".context_length") && matches!(kind, 0 | 2 | 4 | 10))
            || (key.ends_with(".block_count") && matches!(kind, 0 | 2 | 4 | 10))
            || (key.ends_with(".embedding_length") && matches!(kind, 0 | 2 | 4 | 10))
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
    Ok((
        DiscoveryFacts {
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
    Ok((facts, tensors))
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
