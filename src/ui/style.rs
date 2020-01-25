use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::Widget;

pub fn default() -> Style {
    Style::default()
        .bg(tui::style::Color::Indexed(231))
        .fg(tui::style::Color::Black)
}

// a not-so-smart way to clear screen
pub struct EmptyBox {}

impl EmptyBox {
    pub fn fill(area: Rect, buf: &mut Buffer) {
        let s = " ".repeat((area.right() - area.left()) as usize + 1);
        let style = default();
        for y in area.top()..area.bottom() {
            buf.set_string(area.left(), y, &s, style);
        }
    }
}

impl Widget for EmptyBox {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        EmptyBox::fill(area, buf);
    }
}
