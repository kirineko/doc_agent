#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleViolation {
    pub rule_id: &'static str,
    pub xsd_ref: &'static str,
    pub message: String,
    pub line: Option<usize>,
    pub part: Option<String>,
}

impl RuleViolation {
    pub fn new(
        rule_id: &'static str,
        xsd_ref: &'static str,
        message: impl Into<String>,
        line: Option<usize>,
    ) -> Self {
        Self {
            rule_id,
            xsd_ref,
            message: message.into(),
            line,
            part: None,
        }
    }

    pub fn with_part(mut self, part: impl Into<String>) -> Self {
        self.part = Some(part.into());
        self
    }

    pub fn format(&self, default_part: &str) -> String {
        let part = self.part.as_deref().unwrap_or(default_part);
        match self.line {
            Some(line) => format!(
                "{part}:{line} [{} {}] {}",
                self.rule_id, self.xsd_ref, self.message
            ),
            None => format!(
                "{part} [{} {}] {}",
                self.rule_id, self.xsd_ref, self.message
            ),
        }
    }
}

pub fn violations_to_error(default_part: &str, violations: Vec<RuleViolation>) -> String {
    violations
        .into_iter()
        .map(|v| v.format(default_part))
        .collect::<Vec<_>>()
        .join("; ")
}
