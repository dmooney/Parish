/// Curated catalog of model identifiers used to power the `/model`
/// autocomplete dropdown. Names match what the backend forwards to each
/// provider verbatim (Anthropic Messages API model IDs, OpenAI model
/// names, Ollama tags, OpenRouter `vendor/model` slugs, etc.).
///
/// Keep this list focused on currently-shipping, well-known models —
/// it is a navigation aid, not an authoritative registry. Providers
/// continue to accept any string the user types directly.

export interface ModelSuggestion {
	/// The model identifier to send (e.g. `claude-opus-4-7`, `qwen3:14b`).
	name: string;
	/// Human-readable provider label shown in the dropdown.
	provider: string;
}

export const MODEL_CATALOG: ModelSuggestion[] = [
	// Anthropic — native Messages API
	{ name: 'claude-opus-4-7', provider: 'Anthropic' },
	{ name: 'claude-sonnet-4-6', provider: 'Anthropic' },
	{ name: 'claude-haiku-4-5-20251001', provider: 'Anthropic' },
	{ name: 'claude-sonnet-4-20250514', provider: 'Anthropic' },
	{ name: 'claude-opus-4-20250514', provider: 'Anthropic' },

	// OpenAI
	{ name: 'gpt-4.1', provider: 'OpenAI' },
	{ name: 'gpt-4.1-mini', provider: 'OpenAI' },
	{ name: 'gpt-4o', provider: 'OpenAI' },
	{ name: 'gpt-4o-mini', provider: 'OpenAI' },
	{ name: 'o3-mini', provider: 'OpenAI' },
	{ name: 'o1', provider: 'OpenAI' },

	// Google Gemini
	{ name: 'gemini-2.5-pro', provider: 'Google' },
	{ name: 'gemini-2.5-flash', provider: 'Google' },
	{ name: 'gemini-2.0-flash', provider: 'Google' },

	// Groq (hosted open-source models)
	{ name: 'llama-3.3-70b-versatile', provider: 'Groq' },
	{ name: 'llama-3.1-8b-instant', provider: 'Groq' },
	{ name: 'mixtral-8x7b-32768', provider: 'Groq' },

	// xAI Grok
	{ name: 'grok-2-1212', provider: 'xAI' },
	{ name: 'grok-2-vision-1212', provider: 'xAI' },

	// Mistral
	{ name: 'mistral-large-latest', provider: 'Mistral' },
	{ name: 'mistral-small-latest', provider: 'Mistral' },

	// DeepSeek
	{ name: 'deepseek-chat', provider: 'DeepSeek' },
	{ name: 'deepseek-reasoner', provider: 'DeepSeek' },

	// Together AI
	{ name: 'meta-llama/Llama-3.3-70B-Instruct-Turbo', provider: 'Together' },
	{ name: 'Qwen/Qwen2.5-72B-Instruct-Turbo', provider: 'Together' },

	// OpenRouter (vendor-prefixed slugs)
	{ name: 'openrouter/auto', provider: 'OpenRouter' },
	{ name: 'anthropic/claude-sonnet-4-20250514', provider: 'OpenRouter' },
	{ name: 'openai/gpt-4o', provider: 'OpenRouter' },
	{ name: 'google/gemini-2.5-flash', provider: 'OpenRouter' },
	{ name: 'meta-llama/llama-3.3-70b-instruct', provider: 'OpenRouter' },

	// Ollama (local tags) — Rundale's recommended tiers
	{ name: 'qwen3:14b', provider: 'Ollama' },
	{ name: 'qwen3:8b', provider: 'Ollama' },
	{ name: 'qwen3:4b', provider: 'Ollama' },
	{ name: 'llama3.2:3b', provider: 'Ollama' },
	{ name: 'gemma2:9b', provider: 'Ollama' },
	{ name: 'phi4', provider: 'Ollama' },
	{ name: 'mistral-nemo', provider: 'Ollama' }
];

/// Filter the catalog by a free-text query. Matches a substring against
/// either the model name or the provider label (case-insensitive).
/// Empty query returns the full catalog.
export function filterModels(query: string): ModelSuggestion[] {
	const trimmed = query.trim();
	if (trimmed === '') return MODEL_CATALOG;
	const q = trimmed.toLowerCase();
	return MODEL_CATALOG.filter(
		(m) => m.name.toLowerCase().includes(q) || m.provider.toLowerCase().includes(q)
	);
}

/// Per-category subcommand suffixes accepted after `/model.` (matches
/// `parish_config::InferenceCategory::from_name`).
export const MODEL_CATEGORIES = ['dialogue', 'simulation', 'intent', 'reaction'] as const;

/// If `text` matches `/model ` or `/model.<category> ` (with trailing
/// space), returns the leading `/model[.cat]` prefix and the remainder
/// the user has typed after the space. Otherwise returns `null`.
export function detectModelTrigger(
	text: string
): { prefix: string; query: string } | null {
	const match = /^\/model(\.[a-z]+)?\s(.*)$/i.exec(text);
	if (!match) return null;
	const dotted = match[1];
	if (dotted) {
		const cat = dotted.slice(1).toLowerCase();
		if (!(MODEL_CATEGORIES as readonly string[]).includes(cat)) return null;
		return { prefix: `/model.${cat}`, query: match[2] };
	}
	return { prefix: '/model', query: match[2] };
}
