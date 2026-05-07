use foundry_tui_app::{AppModel, LogLine, LogStream, LogTextMode, SectionFocus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use crate::{
    layout_utils::{bottom_window, centered_window, top_window},
    log_text::{horizontal_slice, max_horizontal_offset, wrap_text},
    theme::UiTheme,
};

use super::common::{key_for, panel_title};
use foundry_tui_config::ActionId;

pub(super) fn render_custom_dashboard(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &AppModel,
    theme: UiTheme,
    block: Block<'static>,
) {
    let mut lines = vec![
        Line::from(Span::styled(
            "Forge Command Builder",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Templates loaded: {}    Selected: {}",
            model.custom_templates.len(),
            model.custom_template_index.saturating_add(1)
        )),
        Line::from(format!(
            "Global templates: {}",
            if model.custom_templates_global_path.as_os_str().is_empty() {
                "(not loaded)".to_string()
            } else {
                model.custom_templates_global_path.display().to_string()
            }
        )),
        Line::from(format!(
            "Project templates: {}",
            if model.custom_templates_project_path.as_os_str().is_empty() {
                "(not loaded)".to_string()
            } else {
                model.custom_templates_project_path.display().to_string()
            }
        )),
        Line::from(""),
        Line::from(format!(
            "Open builder: {} or Ctrl+P → {}",
            key_for(model, ActionId::RunCustomCommand),
            ActionId::RunCustomCommand.label()
        )),
        Line::from("Use ↑/↓ while focused here to select template."),
        Line::from(""),
    ];

    if model.custom_templates.is_empty() {
        lines.push(Line::from(Span::styled(
            "No forge presets available. Press builder key to paste a forge command.",
            Style::default().fg(theme.warning),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Templates",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));

        for (index, template) in model.custom_templates.iter().enumerate().take(10) {
            let marker = if index == model.custom_template_index {
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

        let selected_index = model
            .custom_template_index
            .min(model.custom_templates.len().saturating_sub(1));
        if let Some(selected) = model.custom_templates.get(selected_index) {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Selected Template",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(format!("ID: {}", selected.id)));
            lines.push(Line::from(format!("Tool: {}", selected.tool.binary())));
            lines.push(Line::from(format!(
                "Args: {}",
                selected.args_template.join(" ")
            )));
            if let Some(description) = &selected.description {
                lines.push(Line::from(format!("Description: {description}")));
            }
        }
    }

    let visible_rows = area.height.saturating_sub(2) as usize;
    let (start, end) = top_window(lines.len(), model.main_scroll, visible_rows);
    let panel = Paragraph::new(lines[start..end].to_vec())
        .block(block)
        .wrap(Wrap { trim: true });
    frame.render_widget(panel, area);
}

pub(super) fn render_anvil_dashboard(
    frame: &mut Frame<'_>,
    area: Rect,
    model: &AppModel,
    theme: UiTheme,
) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(area);

    let mut all_instance_items = Vec::new();
    for (index, instance) in model.anvil_instances.iter().enumerate() {
        let selected = index == model.selected_anvil_index;
        let marker = if selected { ">" } else { " " };
        let status_color = theme.instances_text;

        all_instance_items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!("{marker} "),
                Style::default()
                    .fg(theme.instances_text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                instance.name.clone(),
                Style::default().fg(theme.instances_text),
            ),
            Span::styled(
                format!(" :{}", instance.port),
                Style::default().fg(theme.instances_text),
            ),
            Span::raw(" "),
            Span::styled(instance.status.label(), Style::default().fg(status_color)),
        ])));
    }

    if all_instance_items.is_empty() {
        all_instance_items.push(ListItem::new(Line::from(Span::styled(
            "No Anvil instances. Press `a` to start one.",
            Style::default().fg(theme.instances_text),
        ))));
    }

    let instance_visible_rows = split[0].height.saturating_sub(2) as usize;
    let (instance_start, instance_end) = centered_window(
        all_instance_items.len(),
        model
            .selected_anvil_index
            .min(all_instance_items.len().saturating_sub(1)),
        instance_visible_rows,
    );
    let instance_title = panel_title(
        "Instances (↑/↓ select)".to_string(),
        matches!(model.focused_section, SectionFocus::AnvilInstancesPanel),
    );

    let instance_panel = List::new(all_instance_items[instance_start..instance_end].to_vec())
        .block(
            Block::default()
                .title(instance_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.instances_border))
                .style(
                    Style::default()
                        .bg(theme.background)
                        .fg(theme.instances_text),
                )
                .padding(Padding::new(1, 1, 0, 0)),
        );
    frame.render_widget(instance_panel, split[0]);

    let mut log_lines = Vec::new();
    let log_content_width = split[1].width.saturating_sub(4) as usize;
    if let Some(instance) = model.anvil_instances.get(model.selected_anvil_index) {
        let visible = split[1].height.saturating_sub(3) as usize;
        if model.log_text_mode == LogTextMode::Horizontal {
            let (start, end) = bottom_window(instance.logs.len(), model.anvil_logs_scroll, visible);
            for entry in &instance.logs[start..end] {
                log_lines.push(anvil_log_line_horizontal(
                    entry,
                    model.anvil_logs_hscroll,
                    log_content_width,
                    theme,
                ));
            }
        } else {
            let mut visual_lines = Vec::new();
            for entry in &instance.logs {
                visual_lines.extend(anvil_log_lines_wrapped(entry, log_content_width, theme));
            }
            let (start, end) = bottom_window(visual_lines.len(), model.anvil_logs_scroll, visible);
            for line in visual_lines
                .into_iter()
                .skip(start)
                .take(end.saturating_sub(start))
            {
                log_lines.push(line);
            }
        }
    } else {
        log_lines.push(ListItem::new(Line::from(Span::styled(
            "Start an Anvil instance to see blockchain logs here.",
            Style::default().fg(theme.muted),
        ))));
    }

    let mode_badge = format!("[{}]", model.log_text_mode.short_label());
    let logs_title = if let Some(instance) = model.anvil_instances.get(model.selected_anvil_index) {
        if let Some(fork) = &instance.fork_url {
            format!(
                "Instance Logs: {} :{} (fork: {}) {}",
                instance.name, instance.port, fork, mode_badge
            )
        } else {
            format!(
                "Instance Logs: {} :{} {}",
                instance.name, instance.port, mode_badge
            )
        }
    } else {
        format!("Instance Logs {}", mode_badge)
    };

    let logs_panel = List::new(log_lines).block(
        Block::default()
            .title(panel_title(
                logs_title,
                matches!(model.focused_section, SectionFocus::AnvilInstanceLogsPanel),
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.instance_logs_border))
            .style(Style::default().bg(theme.background).fg(theme.foreground))
            .padding(Padding::new(1, 1, 0, 0)),
    );
    frame.render_widget(logs_panel, split[1]);
}

pub(super) fn render_jobs(frame: &mut Frame<'_>, area: Rect, model: &AppModel, theme: UiTheme) {
    let all_items = model
        .jobs
        .values()
        .rev()
        .map(|job| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("#{:03}", job.id),
                    Style::default().fg(theme.jobs_text),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:9}", job.status.label()),
                    Style::default().fg(theme.jobs_text),
                ),
                Span::raw(" "),
                Span::styled(job.name.clone(), Style::default().fg(theme.jobs_text)),
            ]))
        })
        .collect::<Vec<_>>();

    let visible_rows = area.height.saturating_sub(2) as usize;
    let (start, end) = top_window(all_items.len(), model.jobs_scroll, visible_rows);
    let title = panel_title(
        "Job Queue".to_string(),
        matches!(model.focused_section, SectionFocus::JobsPanel),
    );

    let widget = List::new(all_items[start..end].to_vec()).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.jobs_border))
            .style(Style::default().bg(theme.background).fg(theme.jobs_text))
            .padding(Padding::new(1, 1, 0, 0)),
    );

    frame.render_widget(widget, area);
}

pub(super) fn render_logs(frame: &mut Frame<'_>, area: Rect, model: &AppModel, theme: UiTheme) {
    let visible_rows = area.height.saturating_sub(2) as usize;
    let log_content_width = area.width.saturating_sub(4) as usize;

    let items = if model.log_text_mode == LogTextMode::Horizontal {
        let (start, end) = bottom_window(model.logs.len(), model.logs_scroll, visible_rows);
        model.logs[start..end]
            .iter()
            .map(|entry| {
                global_log_line_horizontal(entry, model.logs_hscroll, log_content_width, theme)
            })
            .collect::<Vec<_>>()
    } else {
        let mut visual_lines = Vec::new();
        for entry in &model.logs {
            visual_lines.extend(global_log_lines_wrapped(entry, log_content_width, theme));
        }
        let (start, end) = bottom_window(visual_lines.len(), model.logs_scroll, visible_rows);
        visual_lines
            .into_iter()
            .skip(start)
            .take(end.saturating_sub(start))
            .collect::<Vec<_>>()
    };

    let title = panel_title(
        format!("Logs [{}]", model.log_text_mode.short_label()),
        matches!(model.focused_section, SectionFocus::LogsPanel),
    );

    let widget = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.logs_border))
            .style(Style::default().bg(theme.background).fg(theme.foreground))
            .padding(Padding::new(1, 1, 0, 0)),
    );

    frame.render_widget(widget, area);
}

fn anvil_log_line_horizontal(
    entry: &LogLine,
    horizontal_offset: usize,
    content_width: usize,
    theme: UiTheme,
) -> ListItem<'static> {
    let stream_label = match entry.stream {
        LogStream::System => "[sys] ",
        LogStream::Stdout => "[out] ",
        LogStream::Stderr => "[in] ",
    };
    let stream_color = if matches!(entry.stream, LogStream::System) {
        theme.muted
    } else {
        theme.instance_logs_prefix
    };
    let timestamp = format!("{} ", entry.ts.format("%H:%M:%S"));
    let prefix_chars = timestamp.chars().count() + stream_label.chars().count();
    let message_width = content_width.saturating_sub(prefix_chars).max(1);
    let clamped_offset =
        horizontal_offset.min(max_horizontal_offset(&entry.message, message_width));
    let visible_message = horizontal_slice(&entry.message, clamped_offset, message_width);

    ListItem::new(Line::from(vec![
        Span::styled(timestamp, Style::default().fg(theme.instance_logs_prefix)),
        Span::styled(stream_label.to_string(), Style::default().fg(stream_color)),
        Span::styled(visible_message, Style::default().fg(theme.foreground)),
    ]))
}

fn anvil_log_lines_wrapped(
    entry: &LogLine,
    content_width: usize,
    theme: UiTheme,
) -> Vec<ListItem<'static>> {
    let stream_label = match entry.stream {
        LogStream::System => "[sys] ",
        LogStream::Stdout => "[out] ",
        LogStream::Stderr => "[in] ",
    };
    let stream_color = if matches!(entry.stream, LogStream::System) {
        theme.muted
    } else {
        theme.instance_logs_prefix
    };
    let timestamp = format!("{} ", entry.ts.format("%H:%M:%S"));
    let continuation_prefix = " ".repeat(timestamp.chars().count() + stream_label.chars().count());
    let message_width = content_width
        .saturating_sub(timestamp.chars().count() + stream_label.chars().count())
        .max(1);
    let chunks = wrap_text(&entry.message, message_width);

    let mut lines = Vec::new();
    for (index, chunk) in chunks.into_iter().enumerate() {
        if index == 0 {
            lines.push(ListItem::new(Line::from(vec![
                Span::styled(
                    timestamp.clone(),
                    Style::default().fg(theme.instance_logs_prefix),
                ),
                Span::styled(stream_label.to_string(), Style::default().fg(stream_color)),
                Span::styled(chunk, Style::default().fg(theme.foreground)),
            ])));
        } else {
            lines.push(ListItem::new(Line::from(vec![
                Span::styled(
                    continuation_prefix.clone(),
                    Style::default().fg(theme.instance_logs_prefix),
                ),
                Span::styled(chunk, Style::default().fg(theme.foreground)),
            ])));
        }
    }

    lines
}

fn global_log_line_horizontal(
    entry: &LogLine,
    horizontal_offset: usize,
    content_width: usize,
    theme: UiTheme,
) -> ListItem<'static> {
    let timestamp = format!("{} ", entry.ts.format("%H:%M:%S"));
    let message_width = content_width
        .saturating_sub(timestamp.chars().count())
        .max(1);
    let clamped_offset =
        horizontal_offset.min(max_horizontal_offset(&entry.message, message_width));
    let visible_message = horizontal_slice(&entry.message, clamped_offset, message_width);

    ListItem::new(Line::from(vec![
        Span::styled(timestamp, Style::default().fg(theme.muted)),
        Span::styled(visible_message, Style::default().fg(theme.foreground)),
    ]))
}

fn global_log_lines_wrapped(
    entry: &LogLine,
    content_width: usize,
    theme: UiTheme,
) -> Vec<ListItem<'static>> {
    let timestamp = format!("{} ", entry.ts.format("%H:%M:%S"));
    let continuation_prefix = " ".repeat(timestamp.chars().count());
    let message_width = content_width
        .saturating_sub(timestamp.chars().count())
        .max(1);
    let chunks = wrap_text(&entry.message, message_width);

    let mut lines = Vec::new();
    for (index, chunk) in chunks.into_iter().enumerate() {
        if index == 0 {
            lines.push(ListItem::new(Line::from(vec![
                Span::styled(timestamp.clone(), Style::default().fg(theme.muted)),
                Span::styled(chunk, Style::default().fg(theme.foreground)),
            ])));
        } else {
            lines.push(ListItem::new(Line::from(vec![
                Span::styled(
                    continuation_prefix.clone(),
                    Style::default().fg(theme.muted),
                ),
                Span::styled(chunk, Style::default().fg(theme.foreground)),
            ])));
        }
    }

    lines
}
