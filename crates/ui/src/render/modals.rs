use foundry_tui_app::{AnvilInstanceStatus, AnvilPromptField, AppModel, CustomModalStep};
use foundry_tui_config::TemplateParamKind;
use foundry_tui_foundry::redact_cli_args;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

use crate::{
    layout_utils::centered_rect,
    style_utils::{themed_block, BoxTone},
    theme::UiTheme,
};

use super::common::{form_placeholders, is_secret_meta, param_kind_label};

pub(super) fn render_anvil_prompt(frame: &mut Frame<'_>, model: &AppModel, theme: UiTheme) {
    let Some(prompt) = &model.anvil_prompt else {
        return;
    };

    let screen = frame.size();
    let overlay = Block::default().style(Style::default().bg(theme.modal_background));
    frame.render_widget(overlay, screen);

    let area = centered_rect(72, 55, frame.size());
    frame.render_widget(Clear, area);

    let field = |label: &str, value: &str, focused: bool| -> Line<'static> {
        let label_style = if focused {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.muted)
        };

        Line::from(vec![
            Span::styled(format!("{label}: "), label_style),
            Span::styled(value.to_string(), Style::default().fg(theme.foreground)),
        ])
    };

    let mut lines = vec![
        Line::from(Span::styled(
            "New Anvil Instance",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "A node is already running. Set what this new instance needs.",
            Style::default().fg(theme.muted),
        )),
        Line::from(""),
        field("Name", &prompt.name, prompt.focus == AnvilPromptField::Name),
        field("Port", &prompt.port, prompt.focus == AnvilPromptField::Port),
        field(
            "Fork URL (optional)",
            &prompt.fork_url,
            prompt.focus == AnvilPromptField::ForkUrl,
        ),
        field(
            "Extra Flags",
            &prompt.extra_flags,
            prompt.focus == AnvilPromptField::ExtraFlags,
        ),
    ];

    if let Some(error) = &prompt.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            error.clone(),
            Style::default().fg(theme.danger),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Examples: --chain-id 31337 --block-time 1 --steps-tracing",
        Style::default().fg(theme.muted),
    )));
    lines.push(Line::from(Span::styled(
        "Enter: launch  |  Tab/Shift+Tab: move fields  |  Esc: cancel",
        Style::default().fg(theme.muted),
    )));

    let prompt_tone = if prompt.error.is_some() {
        BoxTone::Error
    } else if model.anvil_instances.iter().any(|instance| {
        matches!(
            instance.status,
            AnvilInstanceStatus::Starting | AnvilInstanceStatus::Running
        )
    }) {
        BoxTone::Warning
    } else {
        BoxTone::Modal
    };

    let panel = Paragraph::new(lines).block(
        themed_block(theme, prompt_tone, theme.modal_background).title("Anvil Launch Prompt"),
    );
    frame.render_widget(panel, area);
}

pub(super) fn render_custom_modal(frame: &mut Frame<'_>, model: &AppModel, theme: UiTheme) {
    let Some(modal) = &model.custom_modal else {
        return;
    };

    let screen = frame.size();
    let overlay = Block::default().style(Style::default().bg(theme.muted));
    frame.render_widget(overlay, screen);

    let area = centered_rect(78, 76, frame.size());
    frame.render_widget(Clear, area);

    let mut lines = Vec::new();

    let title = match modal.step {
        CustomModalStep::TemplatePicker => {
            let title = "Forge Builder: Preset Picker";
            if modal.paste_mode {
                lines.push(Line::from(Span::styled(
                    "Paste full forge command",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(
                    "Supports multiline commands with trailing \\ continuations.",
                ));
                lines.push(Line::from(""));
                lines.push(Line::from("Input:"));
                lines.push(Line::from(Span::styled(
                    if modal.paste_input.is_empty() {
                        "<paste command here>"
                    } else {
                        modal.paste_input.as_str()
                    },
                    Style::default().fg(theme.foreground),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Enter: parse forge command  |  Esc: back to preset list",
                    Style::default().fg(theme.muted),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "Select a forge preset",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));

                let paste_marker = if modal.picker_index == 0 { ">" } else { " " };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{paste_marker} "),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "Paste full forge command",
                        Style::default().fg(theme.foreground),
                    ),
                ]));

                for (index, template) in model.custom_templates.iter().enumerate().take(12) {
                    let marker = if modal.picker_index == index + 1 {
                        ">"
                    } else {
                        " "
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("{marker} "),
                            Style::default()
                                .fg(theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            template.label.clone(),
                            Style::default().fg(theme.foreground),
                        ),
                        Span::styled(
                            format!("  [{}]", template.tool.binary()),
                            Style::default().fg(theme.muted),
                        ),
                    ]));
                }

                if model.custom_templates.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "No saved forge presets found yet.",
                        Style::default().fg(theme.warning),
                    )));
                }

                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "↑/↓: select  |  Enter: continue  |  p: paste mode  |  Esc: close builder",
                    Style::default().fg(theme.muted),
                )));
            }
            title
        }
        CustomModalStep::Editor => {
            let title = "Forge Builder: Fill Form";
            let Some(draft) = modal.draft.as_ref() else {
                lines.push(Line::from("Missing draft state."));
                lines.push(Line::from("Press Esc to return."));
                return;
            };
            let placeholders = form_placeholders(draft);
            let field_count = 2 + placeholders.len();
            let editor_index = if field_count == 0 {
                0
            } else {
                modal.editor_index.min(field_count.saturating_sub(1))
            };

            lines.push(Line::from(vec![
                Span::styled(
                    "Template: ",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    draft.template.label.clone(),
                    Style::default().fg(theme.foreground),
                ),
                Span::styled(
                    format!("  [{}]", draft.template.tool.binary()),
                    Style::default().fg(theme.muted),
                ),
            ]));
            lines.push(Line::from(""));

            let rpc_style = if editor_index == 0 {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };
            let raw_style = if editor_index == 1 {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.muted)
            };

            lines.push(Line::from(vec![
                Span::styled(
                    if editor_index == 0 { "> " } else { "  " },
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("RPC Preset: ", rpc_style),
                Span::styled(
                    draft.rpc_preset.clone(),
                    Style::default().fg(theme.foreground),
                ),
                Span::styled(
                    "  [required]  (←/→ cycle)",
                    Style::default().fg(theme.muted),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    if editor_index == 1 { "> " } else { "  " },
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Raw Args Tail: ", raw_style),
                Span::styled(
                    if draft.raw_args.is_empty() {
                        "<none>".to_string()
                    } else {
                        redact_cli_args(
                            &draft
                                .raw_args
                                .split_whitespace()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>(),
                        )
                        .join(" ")
                    },
                    Style::default().fg(theme.foreground),
                ),
                Span::styled("  [optional]", Style::default().fg(theme.muted)),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Template Fields",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));

            if placeholders.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No template placeholders. You can still set raw args.",
                    Style::default().fg(theme.muted),
                )));
            } else {
                for (offset, placeholder) in placeholders.iter().enumerate() {
                    let focused = editor_index == offset + 2;
                    let meta = draft.template.params.get(placeholder);
                    let label = meta
                        .and_then(|meta| meta.label.clone())
                        .unwrap_or_else(|| placeholder.clone());
                    let optional = meta.map(|meta| meta.optional).unwrap_or(false);
                    let kind = meta
                        .map(|meta| meta.kind)
                        .unwrap_or(TemplateParamKind::String);
                    let secret = is_secret_meta(placeholder, meta);
                    let value = draft
                        .param_values
                        .get(placeholder)
                        .cloned()
                        .or_else(|| meta.and_then(|meta| meta.default.clone()))
                        .unwrap_or_default();
                    let value_text = if value.trim().is_empty() {
                        if optional { "<optional>" } else { "<required>" }.to_string()
                    } else if secret {
                        "*".repeat(value.chars().count().max(6))
                    } else {
                        value
                    };

                    let field_style = if focused {
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.muted)
                    };

                    lines.push(Line::from(vec![
                        Span::styled(
                            if focused { "> " } else { "  " },
                            Style::default()
                                .fg(theme.accent)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!("{label}: "), field_style),
                        Span::styled(value_text, Style::default().fg(theme.foreground)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(
                            format!(
                                "[{}] [{}]{}",
                                if optional { "optional" } else { "required" },
                                param_kind_label(kind),
                                if secret { " [secret]" } else { "" }
                            ),
                            Style::default().fg(theme.muted),
                        ),
                    ]));
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Tab/Shift+Tab/↑/↓: move field  |  ←/→: cycle RPC preset",
                Style::default().fg(theme.muted),
            )));
            lines.push(Line::from(Span::styled(
                "Type/Backspace: edit focused value  |  Enter: preview  |  Esc: back",
                Style::default().fg(theme.muted),
            )));
            title
        }
        CustomModalStep::Preview => {
            let title = "Forge Builder: Confirm Execution";
            let Some(draft) = modal.draft.as_ref() else {
                lines.push(Line::from("Missing draft state."));
                lines.push(Line::from("Press Esc to return."));
                return;
            };

            lines.push(Line::from(Span::styled(
                "Resolved command (masked):",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                if draft.display_commandline.is_empty() {
                    "<empty preview>"
                } else {
                    draft.display_commandline.as_str()
                },
                Style::default().fg(theme.foreground),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(format!(
                "RPC preset: {}",
                if draft.rpc_preset.is_empty() {
                    "(none)"
                } else {
                    draft.rpc_preset.as_str()
                }
            )));
            lines.push(Line::from(format!(
                "RPC URL: {}",
                draft
                    .rpc_url
                    .clone()
                    .unwrap_or_else(|| "(none)".to_string())
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Enter: run command  |  Esc: back to editor",
                Style::default().fg(theme.muted),
            )));
            title
        }
    };

    if let Some(error) = &modal.error {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Error: {error}"),
            Style::default().fg(theme.danger),
        )));
    }

    let tone = if modal.error.is_some() {
        BoxTone::Error
    } else {
        BoxTone::Modal
    };

    let widget =
        Paragraph::new(lines).block(themed_block(theme, tone, theme.modal_background).title(title));
    frame.render_widget(widget, area);
}
