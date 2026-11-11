//! Backend-neutral facts and supported output formats for a loaded model.

/// Safe context reuse supported by the selected backend and architecture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheStrategy {
    /// Context reuse is unavailable.
    Disabled,
    /// Attention state supports verified sequence trimming.
    Attention,
    /// Recurrent state requires bounded snapshots and verified restoration.
    Recurrent,
}

/// Tool output syntax that the serving edge can validate and parse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolFormat {
    /// JSON function calls enclosed in XML-style `tool_call` markers.
    XmlJson,
    /// Function/parameter XML blocks inside `tool_call` markers.
    XmlFunction,
}

/// Reasoning output syntax that can be separated from answer text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasoningFormat {
    /// A leading think section with an explicit closing marker.
    ThinkTags,
}

/// Expert routing facts reported by the model artifact. Missing or inconsistent
/// metadata must not be interpreted as evidence that a model is dense.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpertInfo {
    /// Classification: `dense`, `moe`, or `unknown`.
    pub kind: &'static str,
    /// Number of routed experts, when explicitly declared and valid.
    pub expert_count: Option<u32>,
    /// Number of routed experts selected per token, when explicitly declared and valid.
    pub experts_per_token: Option<u32>,
    /// Explanation for unknown or incomplete information.
    pub explanation: Option<&'static str>,
}

impl ExpertInfo {
    /// Interpret GGUF expert metadata without guessing from parameter counts.
    #[must_use]
    pub fn from_metadata(count: Option<&str>, used: Option<&str>) -> Self {
        let parsed_count = count.and_then(|value| value.parse::<u32>().ok());
        let parsed_used = used.and_then(|value| value.parse::<u32>().ok());
        let invalid = (count.is_some() && parsed_count.is_none())
            || (used.is_some() && parsed_used.is_none())
            || matches!((parsed_count, parsed_used), (Some(0), Some(1..)))
            || matches!((parsed_count, parsed_used), (Some(1..), Some(0)))
            || matches!((parsed_count, parsed_used), (Some(n), Some(k)) if k > n);
        if invalid {
            return Self {
                kind: "unknown",
                expert_count: None,
                experts_per_token: None,
                explanation: Some("expert metadata is invalid or inconsistent"),
            };
        }
        match parsed_count {
            Some(0) => Self {
                kind: "dense",
                expert_count: Some(0),
                experts_per_token: parsed_used,
                explanation: None,
            },
            Some(n) => Self {
                kind: "moe",
                expert_count: Some(n),
                experts_per_token: parsed_used,
                explanation: parsed_used
                    .is_none()
                    .then_some("experts selected per token are not declared"),
            },
            None => Self {
                kind: "unknown",
                expert_count: None,
                experts_per_token: None,
                explanation: Some("expert count is not declared"),
            },
        }
    }
}

/// Immutable, observed model facts plus implemented serving capabilities.
///
/// Template declarations alone do not establish arbitrary parser support.
/// This is backend support information, not a claim of model answer quality.
// Capability facts are independent, not mutually exclusive lifecycle states.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelProfile {
    /// Architecture read from the artifact, independent of its filename.
    pub architecture: String,
    /// Expert routing metadata, without inferred active parameter counts.
    pub experts: ExpertInfo,
    /// Training context limit reported by the engine (not available RAM).
    pub context_length: u32,
    /// Whether the loaded engine has a decoder for text generation.
    pub completion: bool,
    /// Whether its selected template can render a user conversation.
    pub chat: bool,
    /// Whether this adapter implements the artifact's embedding mode.
    pub embedding: bool,
    /// Whether the artifact declares embedding pooling, including unavailable modes.
    pub embedding_declared: bool,
    /// Whether an initialized matching vision projector supports images.
    pub vision: bool,
    /// Context strategy selected from the actual architecture/backend.
    pub cache: CacheStrategy,
    /// Whether the tokenizer declares automatic BOS insertion.
    pub tokenizer_adds_bos: bool,
    /// Whether the template declares tool handling (which may be unsupported).
    pub tools_declared: bool,
    /// Implemented compatible tool syntax, or no usable tool parser.
    pub tool_format: Option<ToolFormat>,
    /// Implemented compatible reasoning syntax, if recognized.
    pub reasoning_format: Option<ReasoningFormat>,
    /// Whether the current template adapter implements a thinking switch.
    pub thinking_control: bool,
    /// Whether the template already opens reasoning before generation starts.
    pub reasoning_prefilled: bool,
}

#[cfg(test)]
mod tests {
    use super::ExpertInfo;

    #[test]
    fn expert_metadata_is_conservative() {
        assert_eq!(ExpertInfo::from_metadata(Some("0"), None).kind, "dense");
        let moe = ExpertInfo::from_metadata(Some("8"), Some("2"));
        assert_eq!(moe.kind, "moe");
        assert_eq!(moe.experts_per_token, Some(2));
        assert_eq!(ExpertInfo::from_metadata(Some("8"), None).kind, "moe");
        for (count, used) in [
            (None, None),
            (Some("bad"), None),
            (Some("8"), Some("9")),
            (Some("0"), Some("1")),
            (Some("8"), Some("0")),
            (None, Some("2")),
        ] {
            assert_eq!(ExpertInfo::from_metadata(count, used).kind, "unknown");
        }
    }
}
