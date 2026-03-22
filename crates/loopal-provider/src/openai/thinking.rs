use loopal_provider_api::{EffortLevel, ThinkingConfig};

/// Translate a resolved `ThinkingConfig` into OpenAI's `reasoning_effort` value.
pub fn to_openai_reasoning_effort(config: &ThinkingConfig) -> &'static str {
    match config {
        ThinkingConfig::Effort { level } => match level {
            EffortLevel::Low => "low",
            EffortLevel::Medium => "medium",
            EffortLevel::High | EffortLevel::Max => "high",
        },
        // Budget and other modes degrade to medium
        _ => "medium",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effort_max_maps_to_high() {
        assert_eq!(
            to_openai_reasoning_effort(&ThinkingConfig::Effort {
                level: EffortLevel::Max
            }),
            "high"
        );
    }

    #[test]
    fn budget_degrades_to_medium() {
        assert_eq!(
            to_openai_reasoning_effort(&ThinkingConfig::Budget { tokens: 5000 }),
            "medium"
        );
    }
}
