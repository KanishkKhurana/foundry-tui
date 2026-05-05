use foundry_tui_config::ThemeConfig;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct UiTheme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub panel: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub muted: Color,
    pub app_frame_border: Color,
    pub panel_border: Color,
    pub focus_border: Color,
    pub modal_border: Color,
    pub modal_background: Color,
    pub instances_text: Color,
    pub instances_border: Color,
    pub instance_logs_prefix: Color,
    pub instance_logs_border: Color,
    pub jobs_text: Color,
    pub jobs_border: Color,
    pub logs_border: Color,
}

impl UiTheme {
    pub fn from_config(_config: &ThemeConfig) -> Self {
        let background = Color::Black;
        let foreground = Color::White;
        let accent = Color::Cyan;
        let panel = background;
        let success = Color::Green;
        let warning = Color::Yellow;
        let danger = Color::Red;
        let muted = Color::DarkGray;

        Self {
            background,
            foreground,
            accent,
            panel,
            success,
            warning,
            danger,
            muted,
            app_frame_border: Color::DarkGray,
            panel_border: Color::DarkGray,
            focus_border: Color::White,
            modal_border: Color::White,
            modal_background: background,
            instances_text: Color::Cyan,
            instances_border: Color::White,
            instance_logs_prefix: Color::Green,
            instance_logs_border: Color::DarkGray,
            jobs_text: Color::Yellow,
            jobs_border: Color::Magenta,
            logs_border: Color::DarkGray,
        }
    }
}
