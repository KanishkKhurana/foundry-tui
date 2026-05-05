use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind};
use foundry_tui_app::AppController;
use foundry_tui_config::{load_or_create, load_templates, ConfigLoadState};
use foundry_tui_foundry::ToolEvent;
use foundry_tui_ui::{
    draw, init_terminal, restore_terminal, section_at_position, set_mouse_capture, UiTheme,
};
use tokio::sync::mpsc;
use tracing_subscriber::{fmt, EnvFilter};

enum InputEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
}

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("foundry_tui=info".parse()?))
        .without_time()
        .init();

    let (config, state) = load_or_create()?;
    let config_path = match state {
        ConfigLoadState::Loaded(path) | ConfigLoadState::Created(path) => path,
    };

    let project_root = config
        .foundry
        .project_root
        .clone()
        .unwrap_or(std::env::current_dir()?);
    let template_state = load_templates(&project_root)?;

    let theme = UiTheme::from_config(&config.theme);
    let frame_rate_ms = config.ui.frame_rate_ms;
    let tick_rate_ms = config.ui.tick_rate_ms;

    let mut controller =
        AppController::new_with_templates(config, project_root, config_path, template_state);

    let mut terminal = init_terminal()?;
    let mut mouse_capture_enabled = true;

    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<InputEvent>();
    let (tool_tx, mut tool_rx) = mpsc::unbounded_channel::<ToolEvent>();

    let _input_worker = spawn_input_worker(input_tx);

    let mut frame_interval = tokio::time::interval(Duration::from_millis(frame_rate_ms));
    let mut tick_interval = tokio::time::interval(Duration::from_millis(tick_rate_ms));

    loop {
        tokio::select! {
            _ = frame_interval.tick() => {
                terminal.draw(|frame| draw(frame, &controller.model, theme))?;
            }
            Some(input) = input_rx.recv() => {
                match input {
                    InputEvent::Key(key) => {
                        if key.kind == KeyEventKind::Press && key.code == KeyCode::F(2) {
                            mouse_capture_enabled = !mouse_capture_enabled;
                            set_mouse_capture(mouse_capture_enabled)?;
                            controller.model.notification = Some(if mouse_capture_enabled {
                                "mouse mode enabled (hover + wheel). press F2 for text selection mode".to_string()
                            } else {
                                "text selection mode enabled. press F2 to re-enable mouse interactions".to_string()
                            });
                            continue;
                        }

                        controller.handle_key(key, &tool_tx)
                    }
                    InputEvent::Mouse(mouse) => {
                        if mouse_capture_enabled {
                            if should_focus_from_mouse(mouse.kind) {
                                let viewport = terminal.size()?;
                                if let Some(section) = section_at_position(
                                    &controller.model,
                                    viewport,
                                    mouse.column,
                                    mouse.row,
                                ) {
                                    controller.set_focused_section(section);
                                }
                            }
                            controller.handle_mouse(mouse)
                        }
                    }
                }
            }
            Some(tool_event) = tool_rx.recv() => {
                controller.handle_tool_event(tool_event);
            }
            _ = tick_interval.tick() => {
                controller.on_tick();
            }
        }

        if controller.should_quit() {
            break;
        }
    }

    restore_terminal()?;
    Ok(())
}

fn should_focus_from_mouse(kind: MouseEventKind) -> bool {
    matches!(
        kind,
        MouseEventKind::Moved
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollUp
            | MouseEventKind::Down(_)
            | MouseEventKind::Drag(_)
    )
}

fn spawn_input_worker(input_tx: mpsc::UnboundedSender<InputEvent>) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || loop {
        let has_event = event::poll(Duration::from_millis(50)).unwrap_or(false);
        if !has_event {
            continue;
        }

        match event::read() {
            Ok(Event::Key(key)) => {
                if input_tx.send(InputEvent::Key(key)).is_err() {
                    break;
                }
            }
            Ok(Event::Mouse(mouse)) => {
                if input_tx.send(InputEvent::Mouse(mouse)).is_err() {
                    break;
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    })
}
