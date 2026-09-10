use crate::agent::model_catalog::{ModelAvailability, ModelInfo};
use crate::agent::types::{ModelId, ThinkingConfig, ThinkingEffort};

pub const DEFAULT_CREATE_MODEL: &str = "deepseek-flash";
pub const TITLE_OUTPUT_BUDGET: u32 = 1024;
pub const COMPACTION_OUTPUT_BUDGET: u32 = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedSessionConfig {
    pub model: ModelId,
    pub thinking_enabled: bool,
    pub thinking_effort: ThinkingEffort,
}

impl ResolvedSessionConfig {
    pub fn thinking(self) -> ThinkingConfig {
        ThinkingConfig {
            enabled: self.thinking_enabled,
            effort: self.thinking_effort,
        }
    }
}

pub fn parse_model_id(value: &str) -> Result<ModelId, String> {
    value.parse()
}

pub fn parse_effort(value: &str) -> Result<ThinkingEffort, String> {
    value.parse()
}

pub fn require_callable_model(model: ModelId) -> Result<ModelId, String> {
    if model == ModelId::Mock {
        return Ok(model);
    }
    let info = model.info();
    match info.availability {
        ModelAvailability::Available => Ok(model),
        ModelAvailability::Retired => Err(retired_message(info)),
    }
}

pub fn require_selectable_model(model: ModelId) -> Result<ModelId, String> {
    if model.info().selectable {
        Ok(model)
    } else {
        Err(format!(
            "{} 已不在新建目录中，请选择当前可选模型",
            model.info().label
        ))
    }
}

pub fn resolve_create_config(
    model: Option<&str>,
    thinking_enabled: Option<bool>,
    thinking_effort: Option<&str>,
) -> Result<ResolvedSessionConfig, String> {
    let model_id = parse_model_id(model.unwrap_or(DEFAULT_CREATE_MODEL))?;
    require_selectable_model(model_id)?;
    resolve_thinking(model_id.info(), thinking_enabled, thinking_effort, true).map(|thinking| {
        ResolvedSessionConfig {
            model: model_id,
            thinking_enabled: thinking.enabled,
            thinking_effort: thinking.effort,
        }
    })
}

pub fn resolve_update_config(
    current_model: &str,
    current_enabled: bool,
    current_effort: &str,
    new_model: Option<&str>,
    new_enabled: Option<bool>,
    new_effort: Option<&str>,
) -> Result<ResolvedSessionConfig, String> {
    let current_id = parse_model_id(current_model)?;
    let model_id = if let Some(raw) = new_model {
        let next = parse_model_id(raw)?;
        if next != current_id {
            require_selectable_model(next)?;
        }
        next
    } else {
        current_id
    };

    let model_changed = model_id != current_id;
    if model_changed {
        return resolve_thinking(model_id.info(), new_enabled, new_effort, true).map(|thinking| {
            ResolvedSessionConfig {
                model: model_id,
                thinking_enabled: thinking.enabled,
                thinking_effort: thinking.effort,
            }
        });
    }

    let enabled = new_enabled.or(Some(current_enabled));
    let effort = new_effort.or(Some(current_effort));
    resolve_thinking(model_id.info(), enabled, effort, false).map(|thinking| {
        ResolvedSessionConfig {
            model: model_id,
            thinking_enabled: thinking.enabled,
            thinking_effort: thinking.effort,
        }
    })
}

pub fn auxiliary_thinking(model: ModelId) -> ThinkingConfig {
    if model.info().supports_thinking_toggle {
        ThinkingConfig {
            enabled: false,
            effort: ThinkingEffort::High,
        }
    } else {
        ThinkingConfig {
            enabled: true,
            effort: ThinkingEffort::Low,
        }
    }
}

pub fn capped_output_budget(model: ModelId, requested: u32) -> Result<u32, String> {
    match model.info().max_output_tokens {
        Some(max) if requested > max => Err(format!(
            "{} 输出上限为 {max} tokens，不能申请 {requested}",
            model.info().label
        )),
        _ => Ok(requested),
    }
}

pub fn title_output_budget(model: ModelId) -> Result<u32, String> {
    capped_output_budget(model, TITLE_OUTPUT_BUDGET)
}

pub fn compaction_output_budget(model: ModelId) -> Result<u32, String> {
    capped_output_budget(model, COMPACTION_OUTPUT_BUDGET)
}

fn resolve_thinking(
    info: &ModelInfo,
    thinking_enabled: Option<bool>,
    thinking_effort: Option<&str>,
    fill_defaults: bool,
) -> Result<ThinkingConfig, String> {
    let enabled = match thinking_enabled {
        Some(value) => value,
        None if fill_defaults => info.default_thinking_enabled,
        None => {
            return Err(format!("{} 缺少思考开关配置", info.label));
        }
    };
    let effort = match thinking_effort {
        Some(raw) => parse_effort(raw)?,
        None if fill_defaults => info.default_thinking_effort,
        None => {
            return Err(format!("{} 缺少思考强度配置", info.label));
        }
    };
    validate_thinking(info, enabled, effort)?;
    Ok(ThinkingConfig { enabled, effort })
}

pub(crate) fn validate_thinking(
    info: &ModelInfo,
    enabled: bool,
    effort: ThinkingEffort,
) -> Result<(), String> {
    if !enabled && !info.supports_thinking_toggle {
        return Err(format!("{} 始终启用思考，不能关闭", info.label));
    }
    if !info.thinking_efforts.is_empty() && !info.thinking_efforts.contains(&effort) {
        return Err(format!("{} 不支持思考强度 {}", info.label, effort.as_str()));
    }
    Ok(())
}

fn retired_message(info: &ModelInfo) -> String {
    if info.id == "mimo-v2.5-pro-ultraspeed" {
        "该模型已不可调用。请新建会话选择 MiMo v2.5 Pro。".into()
    } else {
        format!("{} 已不可调用，请新建会话选择当前可用模型", info.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_model_does_not_fallback_to_mock() {
        let err = parse_model_id("not-a-model").unwrap_err();
        assert!(err.contains("unknown model"));
        assert_eq!(parse_model_id("mock").unwrap(), ModelId::Mock);
        assert_eq!(
            parse_model_id("gemini-3.8-flash").unwrap(),
            ModelId::Gemini38Flash
        );
    }

    #[test]
    fn unknown_effort_does_not_fallback_to_high() {
        let err = parse_effort("ultra").unwrap_err();
        assert!(err.contains("unknown thinking effort"));
        assert_eq!(parse_effort("low").unwrap(), ThinkingEffort::Low);
        assert_eq!(parse_effort("medium").unwrap(), ThinkingEffort::Medium);
    }

    #[test]
    fn create_rejects_historical_and_illegal_values() {
        assert!(resolve_create_config(Some("kimi-k2.6"), None, None).is_err());
        assert!(resolve_create_config(Some("deepseek-v4-pro"), None, None).is_err());
        assert!(resolve_create_config(Some("mimo-v2.5-pro-ultraspeed"), None, None).is_err());
        assert!(resolve_create_config(Some("mock"), None, None).is_err());
        assert!(resolve_create_config(Some("gemini-3.8-flash"), Some(false), None).is_err());
        assert!(resolve_create_config(Some("kimi-k3"), Some(false), None).is_err());
        assert!(resolve_create_config(Some("gemini-3.8-flash"), None, Some("max")).is_err());
        assert!(resolve_create_config(Some("deepseek-v4-flash"), None, Some("medium")).is_err());
    }

    #[test]
    fn create_fills_model_defaults_and_keeps_legal_low_medium() {
        let flash = resolve_create_config(None, None, None).unwrap();
        assert_eq!(flash.model, ModelId::DeepSeekV4Flash);
        assert_eq!(flash.model.as_str(), "deepseek-flash");
        assert!(flash.thinking_enabled);
        assert_eq!(flash.thinking_effort, ThinkingEffort::High);

        let k3 = resolve_create_config(Some("kimi-k3"), None, None).unwrap();
        assert_eq!(k3.thinking_effort, ThinkingEffort::Max);

        let glm = resolve_create_config(Some("glm-5.3-flash"), None, None).unwrap();
        assert_eq!(glm.thinking_effort, ThinkingEffort::Max);

        let gemini = resolve_create_config(Some("gemini-3.8-flash"), None, None).unwrap();
        assert_eq!(gemini.thinking_effort, ThinkingEffort::Medium);

        let low =
            resolve_create_config(Some("deepseek-v4-flash"), Some(true), Some("low")).unwrap();
        assert_eq!(low.thinking_effort, ThinkingEffort::Low);

        let gemini_low =
            resolve_create_config(Some("gemini-3.8-flash"), None, Some("low")).unwrap();
        assert_eq!(gemini_low.thinking_effort, ThinkingEffort::Low);
    }

    #[test]
    fn update_applies_target_defaults_when_model_changes() {
        let switched =
            resolve_update_config("kimi-k3", true, "max", Some("gemini-3.8-flash"), None, None)
                .unwrap();
        assert_eq!(switched.model, ModelId::Gemini38Flash);
        assert_eq!(switched.thinking_effort, ThinkingEffort::Medium);

        let kept = resolve_update_config("kimi-k2.6", false, "high", None, None, None).unwrap();
        assert_eq!(kept.model, ModelId::KimiK26);
        assert!(!kept.thinking_enabled);

        assert!(resolve_update_config(
            "deepseek-v4-flash",
            true,
            "high",
            Some("kimi-k2.6"),
            None,
            None
        )
        .is_err());
    }

    #[test]
    fn retired_and_unknown_are_not_callable() {
        let ultra = parse_model_id("mimo-v2.5-pro-ultraspeed").unwrap();
        let err = require_callable_model(ultra).unwrap_err();
        assert!(err.contains("MiMo v2.5 Pro"));
        assert!(require_callable_model(ModelId::Mock).is_ok());
        assert!(require_callable_model(ModelId::DeepSeekV4Pro).is_ok());
        assert!(require_callable_model(ModelId::KimiK26).is_ok());
    }

    #[test]
    fn auxiliary_thinking_and_output_caps() {
        let flash = auxiliary_thinking(ModelId::DeepSeekV4Flash);
        assert!(!flash.enabled);
        let gemini = auxiliary_thinking(ModelId::Gemini38Flash);
        assert!(gemini.enabled);
        assert_eq!(gemini.effort, ThinkingEffort::Low);
        let k3 = auxiliary_thinking(ModelId::KimiK3);
        assert!(k3.enabled);
        assert_eq!(k3.effort, ThinkingEffort::Low);

        assert_eq!(title_output_budget(ModelId::Gemini38Flash).unwrap(), 1024);
        assert_eq!(
            compaction_output_budget(ModelId::DeepSeekV4Flash).unwrap(),
            8192
        );
        assert!(capped_output_budget(ModelId::Gemini38Flash, 65_537).is_err());
        assert_eq!(
            capped_output_budget(ModelId::Gemini38Flash, 65_536).unwrap(),
            65_536
        );
    }
}
