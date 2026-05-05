use foundry_tui_app::{AppModel, JobStatus};
use foundry_tui_config::{ActionId, TemplateParamKind, TemplateParamMeta};
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::theme::UiTheme;

pub(super) fn panel_title(base: String, focused: bool) -> Line<'static> {
    if focused {
        Line::from(vec![
            Span::raw(base),
            Span::styled(
                " [focused]",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else {
        Line::from(base)
    }
}

pub(super) fn key_for(model: &AppModel, action: ActionId) -> String {
    model
        .key_hints
        .get(&action)
        .cloned()
        .unwrap_or_else(|| "-".to_string())
}

pub(super) fn status_color(status: JobStatus, theme: UiTheme) -> Color {
    match status {
        JobStatus::Running => theme.accent,
        JobStatus::Success => theme.success,
        JobStatus::Failed => theme.danger,
        JobStatus::Cancelled => theme.muted,
    }
}

pub(super) fn form_placeholders(draft: &foundry_tui_app::CustomCommandDraft) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut seen = std::collections::BTreeSet::new();

    for token in &draft.template.args_template {
        for placeholder in extract_placeholders_from_token(token) {
            if seen.insert(placeholder.clone()) {
                placeholders.push(placeholder);
            }
        }
    }

    placeholders
}

fn extract_placeholders_from_token(token: &str) -> Vec<String> {
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

pub(super) fn is_secret_meta(name: &str, meta: Option<&TemplateParamMeta>) -> bool {
    if let Some(meta) = meta {
        return meta.secret;
    }

    let lower = name.to_lowercase();
    lower.contains("private")
        || lower.contains("secret")
        || lower.contains("key")
        || lower.contains("password")
}

pub(super) fn param_kind_label(kind: TemplateParamKind) -> String {
    match kind {
        TemplateParamKind::String => "string".to_string(),
        TemplateParamKind::Address => "address".to_string(),
        TemplateParamKind::Hex => "hex".to_string(),
        TemplateParamKind::Uint => "uint".to_string(),
    }
}
