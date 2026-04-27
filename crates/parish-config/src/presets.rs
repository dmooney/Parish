//! Recommended model presets per provider, indexed by inference category.
//!
//! Each entry in [`PresetModels`] is a model id chosen as a sensible default
//! for that role, e.g. Anthropic's preset uses Opus for player-facing
//! dialogue, Sonnet for background simulation and arrival reactions, and
//! Haiku for low-latency intent parsing.
//!
//! Local providers reference Ollama-style tags (`gemma4:e4b`, `qwen3:8b`,
//! `ministral3:3b`) per the recommendations in
//! `docs/design/inference-pipeline.md`. `Custom` and `Simulator` declare no
//! preset — `Custom` because the endpoint shape is unknown, `Simulator`
//! because it ignores the model name entirely.

use crate::provider::{InferenceCategory, Provider};

/// Recommended model id per [`InferenceCategory`], in canonical
/// [`InferenceCategory::ALL`] order: `[Dialogue, Simulation, Intent, Reaction]`.
///
/// `None` in any slot means "no preset available for this provider/role".
pub type PresetModels = [Option<&'static str>; 4];

impl InferenceCategory {
    /// Array index matching [`InferenceCategory::ALL`] order.
    pub fn idx(self) -> usize {
        match self {
            InferenceCategory::Dialogue => 0,
            InferenceCategory::Simulation => 1,
            InferenceCategory::Intent => 2,
            InferenceCategory::Reaction => 3,
        }
    }
}

impl Provider {
    /// Returns the recommended model id for each [`InferenceCategory`].
    ///
    /// `Custom` and `Simulator` return `[None; 4]`: `Custom` is opaque
    /// (the user must know their own endpoint's model ids) and `Simulator`
    /// runs offline without a real model.
    pub fn preset_models(&self) -> PresetModels {
        match self {
            // Dialogue: Opus (highest quality), Sim/Reaction: Sonnet (balanced),
            // Intent: Haiku (cheap + low-latency JSON).
            Provider::Anthropic => [
                Some("claude-opus-4-7"),
                Some("claude-sonnet-4-6"),
                Some("claude-haiku-4-5"),
                Some("claude-sonnet-4-6"),
            ],
            Provider::OpenAi => [
                Some("gpt-4o"),
                Some("gpt-4o-mini"),
                Some("gpt-4o-mini"),
                Some("gpt-4o-mini"),
            ],
            Provider::Google => [
                Some("gemini-2.5-pro"),
                Some("gemini-2.5-flash"),
                Some("gemini-2.5-flash-lite"),
                Some("gemini-2.5-flash"),
            ],
            Provider::Groq => [
                Some("llama-3.3-70b-versatile"),
                Some("llama-3.1-8b-instant"),
                Some("llama-3.1-8b-instant"),
                Some("llama-3.1-8b-instant"),
            ],
            Provider::Xai => [
                Some("grok-4"),
                Some("grok-4-fast"),
                Some("grok-4-fast"),
                Some("grok-4-fast"),
            ],
            Provider::Mistral => [
                Some("mistral-large-latest"),
                Some("mistral-small-latest"),
                Some("ministral-3b-latest"),
                Some("mistral-small-latest"),
            ],
            Provider::DeepSeek => [
                Some("deepseek-chat"),
                Some("deepseek-chat"),
                Some("deepseek-chat"),
                Some("deepseek-chat"),
            ],
            Provider::Together => [
                Some("meta-llama/Llama-3.3-70B-Instruct-Turbo"),
                Some("meta-llama/Llama-3.1-8B-Instruct-Turbo"),
                Some("meta-llama/Llama-3.1-8B-Instruct-Turbo"),
                Some("meta-llama/Llama-3.1-8B-Instruct-Turbo"),
            ],
            // OpenRouter: cross-provider IDs mirror the Anthropic preset.
            Provider::OpenRouter => [
                Some("anthropic/claude-opus-4-7"),
                Some("anthropic/claude-sonnet-4-6"),
                Some("anthropic/claude-haiku-4-5"),
                Some("anthropic/claude-sonnet-4-6"),
            ],
            // Local providers: 9-14B for dialogue, 7-8B for sim/reaction,
            // 3B for intent (per docs/design/inference-pipeline.md).
            Provider::Ollama => [
                Some("gemma4:e4b"),
                Some("qwen3:8b"),
                Some("ministral3:3b"),
                Some("qwen3:8b"),
            ],
            Provider::LmStudio => [
                Some("qwen3:14b"),
                Some("qwen3:8b"),
                Some("ministral3:3b"),
                Some("qwen3:8b"),
            ],
            Provider::Vllm => [
                Some("Qwen/Qwen3-14B"),
                Some("Qwen/Qwen3-8B"),
                Some("mistralai/Ministral-3B-Instruct"),
                Some("Qwen/Qwen3-8B"),
            ],
            Provider::Custom | Provider::Simulator => [None, None, None, None],
        }
    }

    /// Returns the recommended model id for a single [`InferenceCategory`],
    /// or `None` if no preset is available for that role.
    pub fn preset_model(&self, cat: InferenceCategory) -> Option<&'static str> {
        self.preset_models()[cat.idx()]
    }

    /// Returns true if this provider declares any preset models.
    pub fn has_preset(&self) -> bool {
        self.preset_models().iter().any(Option::is_some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inference_category_idx_matches_all_order() {
        for (i, cat) in InferenceCategory::ALL.iter().enumerate() {
            assert_eq!(cat.idx(), i, "idx() must match position in ALL");
        }
    }

    #[test]
    fn cloud_providers_have_complete_presets() {
        for provider in [
            Provider::Anthropic,
            Provider::OpenAi,
            Provider::Google,
            Provider::Groq,
            Provider::Xai,
            Provider::Mistral,
            Provider::DeepSeek,
            Provider::Together,
            Provider::OpenRouter,
        ] {
            let presets = provider.preset_models();
            for (i, slot) in presets.iter().enumerate() {
                let model =
                    slot.unwrap_or_else(|| panic!("{:?} missing preset for slot {}", provider, i));
                assert!(
                    !model.is_empty(),
                    "{:?} has empty preset for slot {}",
                    provider,
                    i
                );
            }
            assert!(provider.has_preset());
        }
    }

    #[test]
    fn local_providers_have_complete_presets() {
        for provider in [Provider::Ollama, Provider::LmStudio, Provider::Vllm] {
            let presets = provider.preset_models();
            for (i, slot) in presets.iter().enumerate() {
                let model =
                    slot.unwrap_or_else(|| panic!("{:?} missing preset for slot {}", provider, i));
                assert!(!model.is_empty());
            }
            assert!(provider.has_preset());
        }
    }

    #[test]
    fn custom_and_simulator_have_no_presets() {
        assert_eq!(Provider::Custom.preset_models(), [None, None, None, None]);
        assert_eq!(
            Provider::Simulator.preset_models(),
            [None, None, None, None]
        );
        assert!(!Provider::Custom.has_preset());
        assert!(!Provider::Simulator.has_preset());
    }

    #[test]
    fn anthropic_preset_matches_user_intent() {
        let p = Provider::Anthropic;
        assert_eq!(
            p.preset_model(InferenceCategory::Dialogue),
            Some("claude-opus-4-7")
        );
        assert_eq!(
            p.preset_model(InferenceCategory::Simulation),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(
            p.preset_model(InferenceCategory::Intent),
            Some("claude-haiku-4-5")
        );
        assert_eq!(
            p.preset_model(InferenceCategory::Reaction),
            Some("claude-sonnet-4-6")
        );
    }

    #[test]
    fn preset_model_indexes_correctly() {
        let p = Provider::Ollama;
        assert_eq!(
            p.preset_model(InferenceCategory::Dialogue),
            Some("gemma4:e4b")
        );
        assert_eq!(
            p.preset_model(InferenceCategory::Intent),
            Some("ministral3:3b")
        );
    }
}
