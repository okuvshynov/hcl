use crate::data::state::State;
use crate::ui::style::{default, EmptyBox};

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Modifier;
use tui::widgets::Widget;

/// Status bar is showing information like mode/visible subset of data
/// And cuurrent 'epoch'.
pub struct StatusBar<'a> {
    state: &'a State,
    series_displayed: (usize, usize),
}

impl<'a> StatusBar<'a> {
    pub fn new(
        state: &'a State,
        series_displayed: (usize, usize),
    ) -> StatusBar<'a> {
        StatusBar {
            state,
            series_displayed,
        }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        EmptyBox::fill(area, buf);

        let message = match (self.state.error_message.as_ref(), self.state.is_auto()) {
            (Some(err), _) => format!("error: {}", err),
            (None, false) => format!("paused"),
            (None, true) => format!("reading"),
        };

        buf.set_string(
            area.left(),
            area.top(),
            &message,
            default().modifier(Modifier::REVERSED),
        );

        // series format on the right
        let y = if self.series_displayed.1 > self.series_displayed.0 {
            format!(
                "series {}..{} out of {}",
                self.series_displayed.0 + 1,
                self.series_displayed.1,
                self.state.data.y.len(),
            )
        } else {
            "no data".to_string()
        };
        buf.set_string(area.right() - y.len() as u16, area.top(), &y, default());
    }
}
