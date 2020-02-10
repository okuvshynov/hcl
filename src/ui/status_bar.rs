use crate::app::settings::{FetchMode, Settings};
use crate::data::state::State;
use crate::ui::style::{default, EmptyBox};

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Modifier;
use tui::widgets::Widget;

/// Status bar is showing information like mode/visible subset of data
/// And cuurrent 'epoch'.
pub struct StatusBar<'a, 'b> {
    state: &'a State,
    settings: &'b Settings,
    series_displayed: (usize, usize),
}

impl<'a, 'b> StatusBar<'a, 'b> {
    pub fn new(
        state: &'a State,
        settings: &'b Settings,
        series_displayed: (usize, usize),
    ) -> StatusBar<'a, 'b> {
        StatusBar {
            state,
            settings,
            series_displayed,
        }
    }
}

impl<'a, 'b> Widget for StatusBar<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        EmptyBox::fill(area, buf);

        // format of status is:
        // refresh_mode [epoch|frequency] | paused/autoscroll
        let mode = match self.settings.fetch_mode() {
            FetchMode::Autorefresh(dur) => format!("refresh every {}ms", dur.as_millis()),
            FetchMode::Incremental => format!("inremental"),
        };

        let message = match (self.state.error_message.as_ref(), self.state.is_auto()) {
            (Some(err), _) => format!("error: {}", err),
            (None, false) => format!("{}, paused", mode),
            (None, true) => mode,
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
