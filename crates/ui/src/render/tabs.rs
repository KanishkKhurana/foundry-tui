use foundry_tui_app::{AppModel, LogStream, SectionFocus, Tab};
use foundry_tui_config::ActionId;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::{
    layout_utils::top_window,
    style_utils::{themed_block, BoxTone},
    theme::UiTheme,
};

use super::{
    common::{key_for, panel_title, status_color},
    panels::{render_anvil_dashboard, render_custom_dashboard},
};

pub(super) fn render_active_tab(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &AppModel,
    theme: UiTheme,
) {
    let main_focused = matches!(model.focused_section, SectionFocus::MainPanel);
    let title = panel_title(
        format!("{} Dashboard", model.active_tab.title()),
        main_focused,
    );
    let tone = if main_focused {
        BoxTone::Focused
    } else {
        BoxTone::Normal
    };

    let block = themed_block(theme, tone, theme.panel).title(title);

    match model.active_tab {
        Tab::Anvil => render_anvil_dashboard(frame, area, model, theme),
        Tab::Custom => render_custom_dashboard(frame, area, model, theme, block),
        Tab::History => {
            let all_lines = model
                .history
                .iter()
                .map(|job| {
                    let color = status_color(job.status, theme);
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("#{:03} ", job.id), Style::default().fg(theme.muted)),
                        Span::styled(
                            format!("{:10}", job.status.label()),
                            Style::default().fg(color),
                        ),
                        Span::styled(
                            format!(" {}", job.name),
                            Style::default().fg(theme.foreground),
                        ),
                    ]))
                })
                .collect::<Vec<_>>();
            let visible_rows = area.height.saturating_sub(2) as usize;
            let (start, end) = top_window(all_lines.len(), model.main_scroll, visible_rows);
            frame.render_widget(List::new(all_lines[start..end].to_vec()).block(block), area);
        }
        Tab::Logs => {
            let all_lines = model
                .logs
                .iter()
                .rev()
                .map(|entry| {
                    let stream_label = match entry.stream {
                        LogStream::System => "SYS",
                        LogStream::Stdout => "OUT",
                        LogStream::Stderr => "ERR",
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{} ", entry.ts.format("%H:%M:%S")),
                            Style::default().fg(theme.muted),
                        ),
                        Span::styled(
                            format!("[{stream_label}] "),
                            Style::default().fg(match entry.stream {
                                LogStream::System => theme.muted,
                                LogStream::Stdout => theme.foreground,
                                LogStream::Stderr => theme.danger,
                            }),
                        ),
                        Span::raw(entry.message.clone()),
                    ]))
                })
                .collect::<Vec<_>>();
            let visible_rows = area.height.saturating_sub(2) as usize;
            let (start, end) = top_window(all_lines.len(), model.main_scroll, visible_rows);
            frame.render_widget(List::new(all_lines[start..end].to_vec()).block(block), area);
        }
        _ => {
            let rpc_line = match (
                model.active_rpc_chain.as_ref(),
                model.active_rpc_preset.as_ref(),
                model.active_rpc_url.as_ref(),
            ) {
                (Some(chain), Some(preset), Some(url)) => {
                    format!("RPC: {chain} | {preset} | {url}")
                }
                _ => "RPC: not configured".to_string(),
            };

            let mut lines = vec![
                Line::from(Span::styled(
                    "Workspace Snapshot",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!(
                    "Foundry config: {}    Remappings: {}    Indexed: {}",
                    if model.project_has_foundry_toml {
                        "yes"
                    } else {
                        "no"
                    },
                    if model.project_has_remappings {
                        "yes"
                    } else {
                        "no"
                    },
                    model.project_indexed_at.format("%H:%M:%S")
                )),
                Line::from(format!(
                    "Loaded Solidity files: {}",
                    model.project_sol_files.len()
                )),
                Line::from(rpc_line),
                Line::from(""),
                Line::from(Span::styled(
                    "Fast actions",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunCustomCommand),
                    ActionId::RunCustomCommand.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunBuild),
                    ActionId::RunBuild.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunTest),
                    ActionId::RunTest.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunScript),
                    ActionId::RunScript.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunCastBlockNumber),
                    ActionId::RunCastBlockNumber.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunVerifyCheck),
                    ActionId::RunVerifyCheck.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunChiselList),
                    ActionId::RunChiselList.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::RunFoundryupUpdate),
                    ActionId::RunFoundryupUpdate.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::StartAnvil),
                    ActionId::StartAnvil.label()
                )),
                Line::from(format!(
                    "- {}: {}",
                    key_for(model, ActionId::StopAnvil),
                    ActionId::StopAnvil.label()
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Loaded files (first 8)",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )),
            ];

            if model.project_sol_files.is_empty() {
                lines.push(Line::from(Span::styled(
                    "No .sol files discovered under src/script/test/contracts",
                    Style::default().fg(theme.muted),
                )));
            } else {
                for file in model.project_sol_files.iter().take(8) {
                    lines.push(Line::from(vec![
                        Span::styled("- ", Style::default().fg(theme.muted)),
                        Span::styled(file.clone(), Style::default().fg(theme.foreground)),
                    ]));
                }
            }

            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Latest jobs",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));

            for job in model.jobs.values().rev().take(6) {
                let color = status_color(job.status, theme);
                lines.push(Line::from(vec![
                    Span::styled(format!("#{:03} ", job.id), Style::default().fg(theme.muted)),
                    Span::styled(
                        format!("{:10}", job.status.label()),
                        Style::default().fg(color),
                    ),
                    Span::raw(format!(" {}", job.name)),
                ]));
            }

            let visible_rows = area.height.saturating_sub(2) as usize;
            let (start, end) = top_window(lines.len(), model.main_scroll, visible_rows);
            let panel = Paragraph::new(lines[start..end].to_vec())
                .block(block)
                .wrap(Wrap { trim: true });
            frame.render_widget(panel, area);
        }
    }
}
