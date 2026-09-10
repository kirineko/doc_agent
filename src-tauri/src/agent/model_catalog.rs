use crate::agent::types::ThinkingEffort;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Deepseek,
    Kimi,
    Mimo,
    Google,
    Zhipu,
    Mock,
}

impl ProviderKind {
    pub fn secrets_key(self) -> &'static str {
        match self {
            Self::Deepseek => "deepseek",
            Self::Kimi => "kimi",
            Self::Mimo => "mimo",
            Self::Google => "google",
            Self::Zhipu => "zhipu",
            Self::Mock => "mock",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModelAvailability {
    Available,
    Retired,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub provider: ProviderKind,
    pub api_model: &'static str,
    pub supports_vision: bool,
    pub supports_effort: bool,
    pub selectable: bool,
    pub availability: ModelAvailability,
    pub supports_thinking_toggle: bool,
    pub thinking_efforts: &'static [ThinkingEffort],
    pub default_thinking_enabled: bool,
    pub default_thinking_effort: ThinkingEffort,
    pub max_context: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
}

const DEEPSEEK_FLASH_EFFORTS: &[ThinkingEffort] = &[
    ThinkingEffort::Low,
    ThinkingEffort::High,
    ThinkingEffort::Max,
];
const DEEPSEEK_PRO_EFFORTS: &[ThinkingEffort] = &[ThinkingEffort::High, ThinkingEffort::Max];
const LOW_HIGH_MAX: &[ThinkingEffort] = &[
    ThinkingEffort::Low,
    ThinkingEffort::High,
    ThinkingEffort::Max,
];
const GEMINI_EFFORTS: &[ThinkingEffort] = &[
    ThinkingEffort::Low,
    ThinkingEffort::Medium,
    ThinkingEffort::High,
];
const NO_EFFORTS: &[ThinkingEffort] = &[];

/// DeepSeek official 384K output cap: 384 * 1024.
pub const DEEPSEEK_MAX_OUTPUT_TOKENS: u32 = 393_216;
/// MiMo / GLM official 128K output cap: 128 * 1024.
pub const MIMO_GLM_MAX_OUTPUT_TOKENS: u32 = 131_072;
/// Gemini 3.8 Flash official output cap.
pub const GEMINI_MAX_OUTPUT_TOKENS: u32 = 65_536;
/// Kimi K3 Chat Completions parameter upper bound (not a guaranteed generation length).
pub const KIMI_K3_MAX_OUTPUT_TOKENS: u32 = 1_048_576;

#[allow(clippy::too_many_arguments)]
const fn model(
    id: &'static str,
    label: &'static str,
    provider: ProviderKind,
    api_model: &'static str,
    supports_vision: bool,
    selectable: bool,
    availability: ModelAvailability,
    supports_thinking_toggle: bool,
    thinking_efforts: &'static [ThinkingEffort],
    default_thinking_enabled: bool,
    default_thinking_effort: ThinkingEffort,
    max_context: u32,
    max_output_tokens: Option<u32>,
) -> ModelInfo {
    ModelInfo {
        id,
        label,
        provider,
        api_model,
        supports_vision,
        supports_effort: !thinking_efforts.is_empty(),
        selectable,
        availability,
        supports_thinking_toggle,
        thinking_efforts,
        default_thinking_enabled,
        default_thinking_effort,
        max_context,
        max_output_tokens,
    }
}

pub fn canonicalize_model_id(id: &str) -> &str {
    match id {
        "deepseek-v4-flash" => "deepseek-flash",
        other => other,
    }
}

pub struct ModelCatalog;

impl ModelCatalog {
    pub const ALL: &'static [ModelInfo] = &[
        model(
            "deepseek-flash",
            "DeepSeek Flash",
            ProviderKind::Deepseek,
            "deepseek-flash",
            true,
            true,
            ModelAvailability::Available,
            true,
            DEEPSEEK_FLASH_EFFORTS,
            true,
            ThinkingEffort::High,
            1_000_000,
            Some(DEEPSEEK_MAX_OUTPUT_TOKENS),
        ),
        model(
            "mimo-v2.5",
            "MiMo v2.5",
            ProviderKind::Mimo,
            "mimo-v2.5",
            true,
            true,
            ModelAvailability::Available,
            true,
            NO_EFFORTS,
            true,
            ThinkingEffort::High,
            1_000_000,
            Some(MIMO_GLM_MAX_OUTPUT_TOKENS),
        ),
        model(
            "mimo-v2.5-pro",
            "MiMo v2.5 Pro",
            ProviderKind::Mimo,
            "mimo-v2.5-pro",
            false,
            true,
            ModelAvailability::Available,
            true,
            NO_EFFORTS,
            true,
            ThinkingEffort::High,
            1_000_000,
            Some(MIMO_GLM_MAX_OUTPUT_TOKENS),
        ),
        model(
            "kimi-k3",
            "Kimi K3",
            ProviderKind::Kimi,
            "kimi-k3",
            true,
            true,
            ModelAvailability::Available,
            false,
            LOW_HIGH_MAX,
            true,
            ThinkingEffort::Max,
            1_000_000,
            Some(KIMI_K3_MAX_OUTPUT_TOKENS),
        ),
        model(
            "gemini-3.8-flash",
            "Gemini 3.8 Flash",
            ProviderKind::Google,
            "gemini-3.8-flash",
            true,
            true,
            ModelAvailability::Available,
            false,
            GEMINI_EFFORTS,
            true,
            ThinkingEffort::Medium,
            1_048_576,
            Some(GEMINI_MAX_OUTPUT_TOKENS),
        ),
        model(
            "glm-5.3-flash",
            "GLM-5.3-Flash",
            ProviderKind::Zhipu,
            "glm-5.3-flash",
            true,
            true,
            ModelAvailability::Available,
            false,
            LOW_HIGH_MAX,
            true,
            ThinkingEffort::Max,
            1_000_000,
            Some(MIMO_GLM_MAX_OUTPUT_TOKENS),
        ),
        model(
            "deepseek-v4-pro",
            "DeepSeek V4 Pro",
            ProviderKind::Deepseek,
            "deepseek-v4-pro",
            false,
            false,
            ModelAvailability::Available,
            true,
            DEEPSEEK_PRO_EFFORTS,
            true,
            ThinkingEffort::High,
            1_000_000,
            Some(DEEPSEEK_MAX_OUTPUT_TOKENS),
        ),
        model(
            "kimi-k2.6",
            "Kimi K2.6",
            ProviderKind::Kimi,
            "kimi-k2.6",
            true,
            false,
            ModelAvailability::Available,
            true,
            NO_EFFORTS,
            true,
            ThinkingEffort::High,
            256_000,
            None,
        ),
        model(
            "mimo-v2.5-pro-ultraspeed",
            "MiMo v2.5 Pro Ultraspeed",
            ProviderKind::Mimo,
            "mimo-v2.5-pro-ultraspeed",
            false,
            false,
            ModelAvailability::Retired,
            true,
            NO_EFFORTS,
            true,
            ThinkingEffort::High,
            1_000_000,
            Some(MIMO_GLM_MAX_OUTPUT_TOKENS),
        ),
    ];

    pub const MOCK: ModelInfo = model(
        "mock",
        "Mock",
        ProviderKind::Mock,
        "mock",
        false,
        false,
        ModelAvailability::Available,
        false,
        NO_EFFORTS,
        false,
        ThinkingEffort::High,
        100_000,
        None,
    );

    pub fn list_public() -> impl Iterator<Item = &'static ModelInfo> {
        Self::ALL.iter()
    }

    pub fn find(id: &str) -> Option<&'static ModelInfo> {
        let id = canonicalize_model_id(id);
        if id == Self::MOCK.id {
            Some(&Self::MOCK)
        } else {
            Self::ALL.iter().find(|m| m.id == id)
        }
    }

    pub fn list() -> &'static [ModelInfo] {
        Self::ALL
    }

    pub fn selectable() -> impl Iterator<Item = &'static ModelInfo> {
        Self::ALL.iter().filter(|m| m.selectable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn by_id(id: &str) -> &'static ModelInfo {
        ModelCatalog::find(id).expect(id)
    }

    #[test]
    fn public_catalog_order_and_selectable_set() {
        let ids: Vec<_> = ModelCatalog::list_public().map(|m| m.id).collect();
        assert_eq!(
            ids,
            vec![
                "deepseek-flash",
                "mimo-v2.5",
                "mimo-v2.5-pro",
                "kimi-k3",
                "gemini-3.8-flash",
                "glm-5.3-flash",
                "deepseek-v4-pro",
                "kimi-k2.6",
                "mimo-v2.5-pro-ultraspeed",
            ]
        );
        let selectable: Vec<_> = ModelCatalog::selectable().map(|m| m.id).collect();
        assert_eq!(
            selectable,
            vec![
                "deepseek-flash",
                "mimo-v2.5",
                "mimo-v2.5-pro",
                "kimi-k3",
                "gemini-3.8-flash",
                "glm-5.3-flash",
            ]
        );
        assert!(ModelCatalog::find("mock").is_some());
        assert!(!ModelCatalog::list_public().any(|m| m.id == "mock"));
    }

    #[test]
    fn catalog_capabilities_match_design() {
        let flash = by_id("deepseek-flash");
        assert_eq!(flash.provider, ProviderKind::Deepseek);
        assert_eq!(flash.label, "DeepSeek Flash");
        assert_eq!(flash.api_model, "deepseek-flash");
        assert!(flash.supports_vision);
        assert_eq!(by_id("deepseek-v4-flash").id, "deepseek-flash");
        assert!(flash.supports_thinking_toggle);
        assert_eq!(flash.thinking_efforts, DEEPSEEK_FLASH_EFFORTS);
        assert_eq!(flash.default_thinking_effort, ThinkingEffort::High);
        assert_eq!(flash.max_context, 1_000_000);
        assert_eq!(flash.max_output_tokens, Some(DEEPSEEK_MAX_OUTPUT_TOKENS));

        let mimo = by_id("mimo-v2.5");
        assert!(mimo.supports_vision);
        assert!(mimo.supports_thinking_toggle);
        assert!(mimo.thinking_efforts.is_empty());
        assert!(!mimo.supports_effort);

        let pro = by_id("mimo-v2.5-pro");
        assert!(!pro.supports_vision);
        assert!(pro.selectable);

        let k3 = by_id("kimi-k3");
        assert!(k3.supports_vision);
        assert!(!k3.supports_thinking_toggle);
        assert_eq!(k3.thinking_efforts, LOW_HIGH_MAX);
        assert_eq!(k3.default_thinking_effort, ThinkingEffort::Max);
        assert_eq!(k3.max_context, 1_000_000);

        let gemini = by_id("gemini-3.8-flash");
        assert_eq!(gemini.provider, ProviderKind::Google);
        assert!(!gemini.supports_thinking_toggle);
        assert_eq!(gemini.thinking_efforts, GEMINI_EFFORTS);
        assert_eq!(gemini.default_thinking_effort, ThinkingEffort::Medium);
        assert_eq!(gemini.max_context, 1_048_576);
        assert_eq!(gemini.max_output_tokens, Some(GEMINI_MAX_OUTPUT_TOKENS));

        let glm = by_id("glm-5.3-flash");
        assert_eq!(glm.provider, ProviderKind::Zhipu);
        assert_eq!(glm.default_thinking_effort, ThinkingEffort::Max);
        assert_eq!(glm.max_output_tokens, Some(MIMO_GLM_MAX_OUTPUT_TOKENS));

        let deepseek_pro = by_id("deepseek-v4-pro");
        assert!(!deepseek_pro.selectable);
        assert_eq!(deepseek_pro.availability, ModelAvailability::Available);
        assert_eq!(deepseek_pro.thinking_efforts, DEEPSEEK_PRO_EFFORTS);

        let k26 = by_id("kimi-k2.6");
        assert!(!k26.selectable);
        assert!(k26.supports_vision);
        assert!(k26.supports_thinking_toggle);
        assert_eq!(k26.max_context, 256_000);

        let ultra = by_id("mimo-v2.5-pro-ultraspeed");
        assert!(!ultra.selectable);
        assert_eq!(ultra.availability, ModelAvailability::Retired);
        assert!(!ultra.supports_vision);
    }

    #[test]
    fn supports_effort_matches_nonempty_efforts() {
        for info in ModelCatalog::ALL
            .iter()
            .chain(std::iter::once(&ModelCatalog::MOCK))
        {
            assert_eq!(
                info.supports_effort,
                !info.thinking_efforts.is_empty(),
                "{}",
                info.id
            );
        }
    }
}
