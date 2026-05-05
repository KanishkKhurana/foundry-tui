use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(crate) fn centered_rect(width_percent: u16, height_percent: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}

pub(crate) fn inset_rect(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(horizontal),
        y: area.y.saturating_add(vertical),
        width: area.width.saturating_sub(horizontal.saturating_mul(2)),
        height: area.height.saturating_sub(vertical.saturating_mul(2)),
    }
}

pub(crate) fn rect_contains(area: Rect, column: u16, row: u16) -> bool {
    if area.width == 0 || area.height == 0 {
        return false;
    }

    let right = area.x.saturating_add(area.width);
    let bottom = area.y.saturating_add(area.height);
    column >= area.x && column < right && row >= area.y && row < bottom
}

pub(crate) fn top_window(total: usize, scroll: usize, visible: usize) -> (usize, usize) {
    if total == 0 || visible == 0 {
        return (0, 0);
    }

    let max_start = total.saturating_sub(visible);
    let start = scroll.min(max_start);
    let end = (start + visible).min(total);
    (start, end)
}

pub(crate) fn bottom_window(
    total: usize,
    scroll_from_bottom: usize,
    visible: usize,
) -> (usize, usize) {
    if total == 0 || visible == 0 {
        return (0, 0);
    }

    let end = total.saturating_sub(scroll_from_bottom.min(total));
    let start = end.saturating_sub(visible);
    (start, end)
}

pub(crate) fn centered_window(total: usize, selected: usize, visible: usize) -> (usize, usize) {
    if total == 0 || visible == 0 {
        return (0, 0);
    }

    if visible >= total {
        return (0, total);
    }

    let half = visible / 2;
    let mut start = selected.saturating_sub(half);
    let max_start = total.saturating_sub(visible);
    if start > max_start {
        start = max_start;
    }
    (start, start + visible)
}
