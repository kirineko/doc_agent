use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Deepseek,
    Kimi,
    Mimo,
    Mock,
}

impl ProviderKind {
    pub fn secrets_key(self) -> &'static str {
        match self {
            Self::Deepseek => "deepseek",
            Self::Kimi => "kimi",
            Self::Mimo => "mimo",
            Self::Mock => "mock",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: &'static str,
    pub label: &'static str,
    pub provider: ProviderKind,
    pub api_model: &'static str,
    pub supports_vision: bool,
    pub supports_effort: bool,
    pub max_context: u32,
}

pub struct ModelCatalog;

impl ModelCatalog {
    pub const ALL: &'static [ModelInfo] = &[
        ModelInfo {
            id: "deepseek-v4-flash",
            label: "DeepSeek V4 Flash",
            provider: ProviderKind::Deepseek,
            api_model: "deepseek-v4-flash",
            supports_vision: false,
            supports_effort: true,
            max_context: 1_000_000,
        },
        ModelInfo {
            id: "deepseek-v4-pro",
            label: "DeepSeek V4 Pro",
            provider: ProviderKind::Deepseek,
            api_model: "deepseek-v4-pro",
            supports_vision: false,
            supports_effort: true,
            max_context: 1_000_000,
        },
        ModelInfo {
            id: "kimi-k2.6",
            label: "Kimi K2.6",
            provider: ProviderKind::Kimi,
            api_model: "kimi-k2.6",
            supports_vision: true,
            supports_effort: false,
            max_context: 256_000,
        },
        ModelInfo {
            id: "mimo-v2.5",
            label: "MiMo v2.5",
            provider: ProviderKind::Mimo,
            api_model: "mimo-v2.5",
            supports_vision: true,
            supports_effort: false,
            max_context: 1_000_000,
        },
        ModelInfo {
            id: "mimo-v2.5-pro",
            label: "MiMo v2.5 Pro",
            provider: ProviderKind::Mimo,
            api_model: "mimo-v2.5-pro",
            supports_vision: false,
            supports_effort: false,
            max_context: 1_000_000,
        },
        ModelInfo {
            id: "mimo-v2.5-pro-ultraspeed",
            label: "MiMo v2.5 Pro Ultraspeed",
            provider: ProviderKind::Mimo,
            api_model: "mimo-v2.5-pro-ultraspeed",
            supports_vision: false,
            supports_effort: false,
            max_context: 1_000_000,
        },
    ];

    const MOCK: ModelInfo = ModelInfo {
        id: "mock",
        label: "Mock",
        provider: ProviderKind::Mock,
        api_model: "mock",
        supports_vision: false,
        supports_effort: false,
        max_context: 100_000,
    };

    pub fn list_public() -> impl Iterator<Item = &'static ModelInfo> {
        Self::ALL.iter()
    }

    pub fn find(id: &str) -> Option<&'static ModelInfo> {
        if id == Self::MOCK.id {
            Some(&Self::MOCK)
        } else {
            Self::ALL.iter().find(|m| m.id == id)
        }
    }

    pub fn list() -> &'static [ModelInfo] {
        Self::ALL
    }
}
