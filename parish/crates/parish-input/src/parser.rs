//! System command parsing and input classification.
//!
//! Translates raw `/`-prefixed input strings into [`Command`] values
//! and routes anything else to free-form game input.

use parish_config::InferenceCategory;
use parish_types::GameSpeed;

use crate::commands::{Command, FlagSubcommand, validate_branch_name, validate_flag_name};
use crate::intent_types::InputResult;

const SPINNER_DEFAULT_SECS: u64 = 30;
const SPINNER_MAX_SECS: u64 = 300;

/// Attempts to parse a system command from raw input.
///
/// Returns `Some(Command)` if the input matches a known `/` command,
/// `None` otherwise.
pub fn parse_system_command(input: &str) -> Option<Command> {
    let trimmed = input.trim();
    let lower = trimmed.to_lowercase();

    // Split into the command keyword and the remainder argument string.
    // e.g. "/map clonalis" → keyword="/map", rest="clonalis"
    //      "/map"          → keyword="/map", rest=""
    let (keyword, rest_trimmed) = match lower.find(' ') {
        Some(pos) => (&lower[..pos], trimmed[pos..].trim()),
        None => (lower.as_str(), ""),
    };

    match keyword {
        "/pause" if rest_trimmed.is_empty() => Some(Command::Pause),
        "/resume" if rest_trimmed.is_empty() => Some(Command::Resume),
        "/quit" if rest_trimmed.is_empty() => Some(Command::Quit),
        "/save" if rest_trimmed.is_empty() => Some(Command::Save),
        "/branches" if rest_trimmed.is_empty() => Some(Command::Branches),
        "/log" if rest_trimmed.is_empty() => Some(Command::Log),
        "/status" | "/where" if rest_trimmed.is_empty() => Some(Command::Status),
        "/help" if rest_trimmed.is_empty() => Some(Command::Help),
        "/irish" if rest_trimmed.is_empty() => Some(Command::ToggleSidebar),
        "/improv" if rest_trimmed.is_empty() => Some(Command::ToggleImprov),
        "/about" if rest_trimmed.is_empty() => Some(Command::About),
        "/designer" if rest_trimmed.is_empty() => Some(Command::Designer),
        "/npcs" if rest_trimmed.is_empty() => Some(Command::NpcsHere),
        "/time" if rest_trimmed.is_empty() => Some(Command::Time),
        "/new" if rest_trimmed.is_empty() => Some(Command::NewGame),
        "/tick" if rest_trimmed.is_empty() => Some(Command::Tick),
        "/flags" if rest_trimmed.is_empty() => Some(Command::Flags),
        "/session" | "/tune" | "/music" | "/fiddle" | "/seisiun" if rest_trimmed.is_empty() => {
            Some(Command::Session)
        }

        "/fork" => {
            if rest_trimmed.is_empty() {
                Some(Command::Help) // bare /fork → show help
            } else {
                match validate_branch_name(rest_trimmed) {
                    Ok(valid) => Some(Command::Fork(valid)),
                    Err(msg) => Some(Command::InvalidBranchName(msg)),
                }
            }
        }

        "/load" => {
            if rest_trimmed.is_empty() {
                Some(Command::Load(String::new())) // empty string = show save picker
            } else {
                match validate_branch_name(rest_trimmed) {
                    Ok(valid) => Some(Command::Load(valid)),
                    Err(msg) => Some(Command::InvalidBranchName(msg)),
                }
            }
        }

        "/map" => {
            if rest_trimmed.is_empty() {
                Some(Command::Map(None))
            } else {
                Some(Command::Map(Some(rest_trimmed.to_string())))
            }
        }

        "/wait" => {
            let mins = rest_trimmed.parse::<u32>().unwrap_or(15);
            Some(Command::Wait(mins))
        }

        "/theme" => {
            if rest_trimmed.is_empty() {
                Some(Command::Theme(None))
            } else {
                Some(Command::Theme(Some(rest_trimmed.to_string())))
            }
        }

        "/unexplored" => {
            let arg = rest_trimmed.to_lowercase();
            match arg.as_str() {
                "reveal" | "show" | "on" => Some(Command::Unexplored(Some(true))),
                "hide" | "off" => Some(Command::Unexplored(Some(false))),
                _ => Some(Command::Unexplored(None)),
            }
        }

        "/preset" => {
            if rest_trimmed.is_empty() {
                Some(Command::ShowPreset)
            } else {
                Some(Command::ApplyPreset(rest_trimmed.to_string()))
            }
        }

        "/provider" => {
            if rest_trimmed.is_empty() {
                Some(Command::ShowProvider)
            } else {
                Some(Command::SetProvider(rest_trimmed.to_string()))
            }
        }

        "/model" => {
            if rest_trimmed.is_empty() {
                Some(Command::ShowModel)
            } else {
                Some(Command::SetModel(rest_trimmed.to_string()))
            }
        }

        "/key" => {
            if rest_trimmed.is_empty() {
                Some(Command::ShowKey)
            } else {
                Some(Command::SetKey(rest_trimmed.to_string()))
            }
        }

        "/spinner" => {
            let secs = rest_trimmed
                .parse::<u64>()
                .unwrap_or(SPINNER_DEFAULT_SECS)
                .min(SPINNER_MAX_SECS);
            Some(Command::Spinner(secs))
        }

        "/debug" => {
            if rest_trimmed.is_empty() {
                Some(Command::Debug(None))
            } else {
                Some(Command::Debug(Some(rest_trimmed.to_string())))
            }
        }

        "/speed" => {
            if rest_trimmed.is_empty() {
                Some(Command::ShowSpeed)
            } else {
                match GameSpeed::from_name(rest_trimmed) {
                    Some(speed) => Some(Command::SetSpeed(speed)),
                    None => Some(Command::InvalidSpeed(rest_trimmed.to_string())),
                }
            }
        }

        "/cloud" => {
            if rest_trimmed.is_empty() {
                Some(Command::ShowCloud)
            } else {
                parse_cloud_subcommand(rest_trimmed)
            }
        }

        "/weather" => {
            if rest_trimmed.is_empty() {
                Some(Command::Weather(None))
            } else {
                Some(Command::Weather(Some(rest_trimmed.to_string())))
            }
        }

        "/flag" => {
            if rest_trimmed.is_empty() || rest_trimmed.to_lowercase() == "list" {
                Some(Command::Flag(FlagSubcommand::List))
            } else {
                parse_flag_subcommand(rest_trimmed)
            }
        }

        // Dot-notation per-category commands: /model.<cat>, /provider.<cat>, /key.<cat>
        kw if kw.starts_with("/model.")
            || kw.starts_with("/provider.")
            || kw.starts_with("/key.") =>
        {
            // Re-assemble the full trimmed string for parse_category_command since it
            // expects the original (potentially mixed-case) trimmed input alongside the
            // lowercase version for prefix-stripping.
            parse_category_command(trimmed, &lower)
        }

        _ => None,
    }
}

/// Parses `/cloud <subcommand>` arguments.
fn parse_cloud_subcommand(rest: &str) -> Option<Command> {
    let rest_lower = rest.to_lowercase();

    // Split subcommand keyword from its argument.
    let (sub_kw, sub_arg) = match rest_lower.find(' ') {
        Some(pos) => (&rest_lower[..pos], rest[pos..].trim()),
        None => (rest_lower.as_str(), ""),
    };

    match sub_kw {
        "provider" => {
            if sub_arg.is_empty() {
                Some(Command::ShowCloud)
            } else {
                Some(Command::SetCloudProvider(sub_arg.to_string()))
            }
        }
        "model" => {
            if sub_arg.is_empty() {
                Some(Command::ShowCloudModel)
            } else {
                Some(Command::SetCloudModel(sub_arg.to_string()))
            }
        }
        "key" => {
            if sub_arg.is_empty() {
                Some(Command::ShowCloudKey)
            } else {
                Some(Command::SetCloudKey(sub_arg.to_string()))
            }
        }
        _ => Some(Command::ShowCloud),
    }
}

/// Parses `/flag <subcommand>` arguments (enable/disable/list).
fn parse_flag_subcommand(rest: &str) -> Option<Command> {
    let rest_lower = rest.to_lowercase();

    let (sub_kw, sub_arg) = match rest_lower.find(' ') {
        Some(pos) => (&rest_lower[..pos], rest[pos..].trim()),
        None => (rest_lower.as_str(), ""),
    };

    match sub_kw {
        "enable" => {
            if sub_arg.is_empty() {
                // `/flag enable` with no name → show list
                Some(Command::Flag(FlagSubcommand::List))
            } else {
                match validate_flag_name(sub_arg) {
                    Ok(valid) => Some(Command::Flag(FlagSubcommand::Enable(valid))),
                    Err(msg) => Some(Command::InvalidFlagName(msg)),
                }
            }
        }
        "disable" => {
            if sub_arg.is_empty() {
                Some(Command::Flag(FlagSubcommand::List))
            } else {
                match validate_flag_name(sub_arg) {
                    Ok(valid) => Some(Command::Flag(FlagSubcommand::Disable(valid))),
                    Err(msg) => Some(Command::InvalidFlagName(msg)),
                }
            }
        }
        "list" => Some(Command::Flag(FlagSubcommand::List)),
        _ => Some(Command::InvalidFlagName(format!(
            "Unknown flag sub-command '{}'. Use: /flag enable <name>, /flag disable <name>, /flag list",
            rest
        ))),
    }
}

/// Parses dot-notation per-category commands like `/model.dialogue`, `/provider.intent`.
///
/// Returns `Some(Command)` if the input matches a `/<base>.<category>` pattern
/// where base is `model`, `provider`, or `key`, and category is `dialogue`,
/// `simulation`, or `intent`.
fn parse_category_command(trimmed: &str, lower: &str) -> Option<Command> {
    for (prefix, show_fn, set_fn) in &[
        (
            "/model.",
            Command::ShowCategoryModel as fn(InferenceCategory) -> Command,
            Command::SetCategoryModel as fn(InferenceCategory, String) -> Command,
        ),
        (
            "/provider.",
            Command::ShowCategoryProvider as fn(InferenceCategory) -> Command,
            Command::SetCategoryProvider as fn(InferenceCategory, String) -> Command,
        ),
        (
            "/key.",
            Command::ShowCategoryKey as fn(InferenceCategory) -> Command,
            Command::SetCategoryKey as fn(InferenceCategory, String) -> Command,
        ),
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let (cat_str, arg) = match rest.find(' ') {
                Some(pos) => (&rest[..pos], trimmed[prefix.len() + pos..].trim()),
                None => (rest, ""),
            };
            let category = InferenceCategory::from_name(cat_str)?;
            if arg.is_empty() {
                return Some(show_fn(category));
            } else {
                return Some(set_fn(category, arg.to_string()));
            }
        }
    }
    None
}

/// Classifies raw input as either a system command or game input.
///
/// If the input starts with `/` and matches a known command, returns
/// `InputResult::SystemCommand`. Otherwise returns `InputResult::GameInput`.
pub fn classify_input(raw: &str) -> InputResult {
    let trimmed = raw.trim();
    if let Some(cmd) = parse_system_command(trimmed) {
        InputResult::SystemCommand(cmd)
    } else {
        InputResult::GameInput(trimmed.to_string())
    }
}
