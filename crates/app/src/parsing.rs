use std::collections::BTreeSet;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use foundry_tui_config::{TemplateParamKind, TemplateParamMeta, TemplateTool};
use foundry_tui_foundry::ToolKind;

use crate::model::{AnvilLaunchPrompt, AnvilPromptField, CustomCommandDraft};

pub(crate) fn rpc_chain_label(preset: &str) -> String {
    match preset {
        "local" => "Local Anvil (127.0.0.1:8545)".to_string(),
        "across-ethereum-1" => "Ethereum (1)".to_string(),
        "across-megaeth-4326" => "MegaETH (4326)".to_string(),
        "across-optimism-10" => "Optimism (10)".to_string(),
        "across-polygon-137" => "Polygon (137)".to_string(),
        "across-arbitrum-42161" => "Arbitrum (42161)".to_string(),
        "across-zksync-324" => "zkSync (324)".to_string(),
        "across-base-8453" => "Base (8453)".to_string(),
        "across-linea-59144" => "Linea (59144)".to_string(),
        "across-mode-34443" => "Mode (34443)".to_string(),
        "across-blast-81457" => "Blast (81457)".to_string(),
        "across-lisk-1135" => "Lisk (1135)".to_string(),
        "across-zora-7777777" => "Zora (7777777)".to_string(),
        "across-world-chain-480" => "World Chain (480)".to_string(),
        "across-ink-57073" => "Ink (57073)".to_string(),
        "across-soneium-1868" => "Soneium (1868)".to_string(),
        "across-unichain-130" => "Unichain (130)".to_string(),
        "across-lens-232" => "Lens (232)".to_string(),
        "across-bnb-smart-chain-56" => "BNB Smart Chain (56)".to_string(),
        "across-solana-34268394551451" => "Solana (34268394551451)".to_string(),
        "across-hyperevm-999" => "HyperEVM (999)".to_string(),
        "across-plasma-9745" => "Plasma (9745)".to_string(),
        "across-monad-143" => "Monad (143)".to_string(),
        "across-tempo-4217" => "Tempo (4217)".to_string(),
        "across-hypercore-1337" => "HyperCore (1337)".to_string(),
        "across-lighter-2337" => "Lighter (2337)".to_string(),
        _ => preset.to_string(),
    }
}

pub(crate) fn is_non_evm_preset(preset: &str) -> bool {
    matches!(
        preset,
        "across-solana-34268394551451" | "across-hypercore-1337"
    )
}

pub(crate) fn parse_flag_value(args: &[String], flag: &str) -> Option<String> {
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == flag {
            return args.get(index + 1).cloned();
        }
        index += 1;
    }
    None
}

pub(crate) fn remove_flag_and_value(args: &mut Vec<String>, flag: &str) {
    let mut index = 0usize;
    while index < args.len() {
        if args[index] == flag {
            args.remove(index);
            if index < args.len() {
                args.remove(index);
            }
        } else {
            index += 1;
        }
    }
}

pub(crate) fn merge_template_and_raw_args(
    template_args: &[String],
    raw_args: &[String],
) -> Vec<String> {
    let overrides = raw_override_flags(raw_args);
    let mut merged = Vec::new();
    let mut index = 0usize;

    while index < template_args.len() {
        let token = &template_args[index];
        if let Some(flag) = normalized_long_flag(token) {
            if overrides.contains(&flag) {
                if token == &flag
                    && template_args
                        .get(index + 1)
                        .is_some_and(|next| !looks_like_flag(next))
                {
                    index += 2;
                } else {
                    index += 1;
                }
                continue;
            }
        }

        merged.push(token.clone());
        index += 1;
    }

    merged.extend(raw_args.iter().cloned());
    merged
}

fn raw_override_flags(raw_args: &[String]) -> BTreeSet<String> {
    raw_args
        .iter()
        .filter_map(|token| normalized_long_flag(token))
        .collect()
}

fn normalized_long_flag(token: &str) -> Option<String> {
    if !token.starts_with("--") {
        return None;
    }

    if let Some((left, _)) = token.split_once('=') {
        return Some(left.to_string());
    }

    Some(token.to_string())
}

pub(crate) fn looks_like_flag(token: &str) -> bool {
    token.starts_with('-')
}

pub(crate) fn parse_template_tool(binary: &str) -> Option<TemplateTool> {
    match binary {
        "forge" => Some(TemplateTool::Forge),
        "cast" => Some(TemplateTool::Cast),
        "anvil" => Some(TemplateTool::Anvil),
        "chisel" => Some(TemplateTool::Chisel),
        "foundryup" => Some(TemplateTool::Foundryup),
        _ => None,
    }
}

pub(crate) fn template_tool_to_tool_kind(tool: TemplateTool) -> ToolKind {
    match tool {
        TemplateTool::Forge => ToolKind::Forge,
        TemplateTool::Cast => ToolKind::Cast,
        TemplateTool::Anvil => ToolKind::Anvil,
        TemplateTool::Chisel => ToolKind::Chisel,
        TemplateTool::Foundryup => ToolKind::Foundryup,
    }
}

pub(crate) fn normalize_pasted_command(input: &str) -> String {
    let mut merged = String::new();
    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(prefix) = line.strip_suffix('\\') {
            merged.push_str(prefix.trim_end());
            merged.push(' ');
        } else {
            merged.push_str(line);
            merged.push(' ');
        }
    }
    merged.trim().to_string()
}

pub(crate) fn convert_angle_placeholder_token(token: &str) -> String {
    let Some(inner) = token
        .strip_prefix('<')
        .and_then(|value| value.strip_suffix('>'))
    else {
        return token.to_string();
    };

    let normalized = sanitize_placeholder_name(inner);
    if normalized.is_empty() {
        return token.to_string();
    }

    format!("{{{{{normalized}}}}}")
}

fn sanitize_placeholder_name(value: &str) -> String {
    let mut output = String::new();
    let mut previous_underscore = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                output.push(ch.to_ascii_lowercase());
            } else {
                output.push(ch);
            }
            previous_underscore = false;
        } else if !previous_underscore {
            output.push('_');
            previous_underscore = true;
        }
    }

    while output.starts_with('_') {
        output.remove(0);
    }
    while output.ends_with('_') {
        output.pop();
    }
    if output.is_empty() {
        return String::new();
    }
    if output.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        return format!("value_{output}");
    }

    output
}

pub(crate) fn infer_param_meta(name: &str) -> TemplateParamMeta {
    let lower = name.to_lowercase();
    let kind = if lower.contains("addr") || lower.contains("address") {
        TemplateParamKind::Address
    } else if lower.contains("amount") || lower.contains("nonce") || lower.contains("id") {
        TemplateParamKind::Uint
    } else if lower.contains("hex") || lower.contains("key") || lower.contains("secret") {
        TemplateParamKind::Hex
    } else {
        TemplateParamKind::String
    };

    let secret = lower.contains("private")
        || lower.contains("secret")
        || lower.contains("key")
        || lower.contains("password");

    let label = Some(
        name.split('_')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                let first = chars.next().unwrap_or_default().to_ascii_uppercase();
                format!("{first}{}", chars.as_str())
            })
            .collect::<Vec<_>>()
            .join(" "),
    );

    TemplateParamMeta {
        label,
        default: None,
        secret,
        optional: false,
        kind,
    }
}

pub(crate) fn extract_placeholders(args: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut placeholders = Vec::new();
    for arg in args {
        for placeholder in extract_placeholders_from_token(arg) {
            if seen.insert(placeholder.clone()) {
                placeholders.push(placeholder);
            }
        }
    }
    placeholders
}

pub(crate) fn custom_form_placeholders(draft: &CustomCommandDraft) -> Vec<String> {
    extract_placeholders(&draft.template.args_template)
}

pub(crate) fn extract_placeholders_from_token(token: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut cursor = 0usize;

    while let Some(start_offset) = token[cursor..].find("{{") {
        let start = cursor + start_offset;
        let rest = &token[start + 2..];
        let Some(end_offset) = rest.find("}}") else {
            break;
        };
        let end = start + 2 + end_offset;
        let name = token[start + 2..end].trim();
        if !name.is_empty() {
            placeholders.push(name.to_string());
        }
        cursor = end + 2;
    }

    placeholders
}

pub(crate) fn has_placeholder_tokens(token: &str) -> bool {
    token.contains("{{") && token.contains("}}")
}

pub(crate) fn is_exact_placeholder_token(token: &str) -> bool {
    let trimmed = token.trim();
    let placeholders = extract_placeholders_from_token(trimmed);
    if placeholders.len() != 1 {
        return false;
    }

    trimmed == format!("{{{{{}}}}}", placeholders[0])
}

pub(crate) fn is_secret_placeholder(
    template: &foundry_tui_config::CustomTemplate,
    name: &str,
) -> bool {
    template
        .params
        .get(name)
        .map(|meta| meta.secret)
        .unwrap_or_else(|| infer_param_meta(name).secret)
}

pub(crate) fn initial_placeholder_value(
    template: &foundry_tui_config::CustomTemplate,
    param_values: &std::collections::BTreeMap<String, String>,
    placeholder: &str,
    rpc_url: Option<String>,
) -> String {
    if let Some(value) = param_values.get(placeholder) {
        return value.clone();
    }
    if placeholder.eq_ignore_ascii_case("rpc_url") {
        if let Some(url) = rpc_url {
            return url;
        }
    }
    template
        .params
        .get(placeholder)
        .and_then(|meta| meta.default.clone())
        .unwrap_or_default()
}

pub(crate) fn validate_custom_param_value(
    placeholder: &str,
    value: &str,
    meta: Option<&TemplateParamMeta>,
) -> Option<String> {
    let kind = meta
        .map(|meta| meta.kind)
        .unwrap_or_else(|| infer_param_meta(placeholder).kind);

    match kind {
        TemplateParamKind::String => None,
        TemplateParamKind::Address => {
            let value = value.trim();
            let valid = value.len() == 42
                && value.starts_with("0x")
                && value[2..].chars().all(|ch| ch.is_ascii_hexdigit());
            if valid {
                None
            } else {
                Some(format!(
                    "`{placeholder}` must be a valid 0x-prefixed address"
                ))
            }
        }
        TemplateParamKind::Hex => {
            let value = value.trim();
            let valid = value.len() > 2
                && value.starts_with("0x")
                && value[2..].chars().all(|ch| ch.is_ascii_hexdigit());
            if valid {
                None
            } else {
                Some(format!(
                    "`{placeholder}` must be a valid 0x-prefixed hex value"
                ))
            }
        }
        TemplateParamKind::Uint => {
            if value.chars().all(|ch| ch.is_ascii_digit()) {
                None
            } else {
                Some(format!("`{placeholder}` must be an unsigned integer"))
            }
        }
    }
}

pub(crate) fn anvil_prompt_field_mut(prompt: &mut AnvilLaunchPrompt) -> &mut String {
    match prompt.focus {
        AnvilPromptField::Name => &mut prompt.name,
        AnvilPromptField::Port => &mut prompt.port,
        AnvilPromptField::ForkUrl => &mut prompt.fork_url,
        AnvilPromptField::ExtraFlags => &mut prompt.extra_flags,
    }
}

pub(crate) fn parse_cli_args(input: &str) -> std::result::Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        match ch {
            '\\' if !in_single => {
                escaped = true;
            }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            ch if ch.is_whitespace() && !in_single && !in_double => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if escaped {
        return Err("invalid flags: trailing escape sequence".to_string());
    }
    if in_single || in_double {
        return Err("invalid flags: unclosed quote in extra flags".to_string());
    }
    if !current.is_empty() {
        args.push(current);
    }

    Ok(args)
}

pub(crate) fn contains_placeholders(args: &[String]) -> bool {
    args.iter().any(|arg| {
        (arg.contains('<') && arg.contains('>')) || (arg.contains("{{") && arg.contains("}}"))
    })
}

pub(crate) fn key_matches(binding: &str, key: KeyEvent) -> bool {
    let mut required_modifiers = KeyModifiers::NONE;
    let mut key_token = "";
    let normalized = binding.trim().to_lowercase();

    for part in normalized.split('+') {
        match part {
            "ctrl" | "control" => required_modifiers |= KeyModifiers::CONTROL,
            "alt" => required_modifiers |= KeyModifiers::ALT,
            "shift" => required_modifiers |= KeyModifiers::SHIFT,
            other => key_token = other,
        }
    }

    let actual_modifiers =
        key.modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
    if actual_modifiers != required_modifiers {
        return false;
    }

    match key_token {
        "tab" => matches!(key.code, KeyCode::Tab),
        "backtab" => matches!(key.code, KeyCode::BackTab),
        "esc" => matches!(key.code, KeyCode::Esc),
        "enter" => matches!(key.code, KeyCode::Enter),
        "up" => matches!(key.code, KeyCode::Up),
        "down" => matches!(key.code, KeyCode::Down),
        "left" => matches!(key.code, KeyCode::Left),
        "right" => matches!(key.code, KeyCode::Right),
        "space" => matches!(key.code, KeyCode::Char(' ')),
        token if token.len() == 1 => match key.code {
            KeyCode::Char(ch) => {
                ch.to_ascii_lowercase() == token.chars().next().unwrap_or_default()
            }
            _ => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_detection_works() {
        let args = vec!["verify-check".to_string(), "<GUID>".to_string()];
        assert!(contains_placeholders(&args));
    }

    #[test]
    fn remove_flag_and_value_drops_overridden_options() {
        let mut args = vec![
            "--port".to_string(),
            "8545".to_string(),
            "--fork-url".to_string(),
            "http://localhost:8545".to_string(),
            "--steps-tracing".to_string(),
        ];

        remove_flag_and_value(&mut args, "--port");
        remove_flag_and_value(&mut args, "--fork-url");

        assert_eq!(args, vec!["--steps-tracing".to_string()]);
    }

    #[test]
    fn chain_label_and_non_evm_detection_work() {
        assert_eq!(
            rpc_chain_label("across-ethereum-1"),
            "Ethereum (1)".to_string()
        );
        assert!(is_non_evm_preset("across-solana-34268394551451"));
        assert!(!is_non_evm_preset("across-base-8453"));
    }

    #[test]
    fn parse_cli_args_handles_quotes() {
        let parsed = parse_cli_args("--chain-id 31337 --fork-url \"https://rpc.example\"")
            .expect("expected valid flags");
        assert_eq!(
            parsed,
            vec![
                "--chain-id".to_string(),
                "31337".to_string(),
                "--fork-url".to_string(),
                "https://rpc.example".to_string()
            ]
        );
    }

    #[test]
    fn parse_cli_args_rejects_unclosed_quote() {
        let error = parse_cli_args("--fork-url \"https://rpc").expect_err("expected parse error");
        assert!(error.contains("unclosed quote"));
    }

    #[test]
    fn merge_template_and_raw_args_overrides_flags() {
        let template = vec![
            "script".to_string(),
            "script/Increment.s.sol:IncrementScript".to_string(),
            "--rpc-url".to_string(),
            "{{rpc_url}}".to_string(),
            "--sig".to_string(),
            "run()".to_string(),
        ];
        let raw = vec![
            "--rpc-url".to_string(),
            "http://127.0.0.1:9555".to_string(),
            "-vv".to_string(),
        ];

        let merged = merge_template_and_raw_args(&template, &raw);
        assert!(!merged.contains(&"{{rpc_url}}".to_string()));
        assert_eq!(
            merged,
            vec![
                "script".to_string(),
                "script/Increment.s.sol:IncrementScript".to_string(),
                "--sig".to_string(),
                "run()".to_string(),
                "--rpc-url".to_string(),
                "http://127.0.0.1:9555".to_string(),
                "-vv".to_string()
            ]
        );
    }
}
