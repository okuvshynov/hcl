use crate::app::settings::Settings;
use crate::app::window::{Window, WindowAdjust};
use crate::data::scale_config::ScalesConfig;
use crate::data::series::{SeriesSet, Slice};

#[derive(Debug)]
pub struct State {
    pub data: SeriesSet,
    pub error_message: Option<String>,
    pub x: Window,
    pub y: Window,
    pub scales: Option<ScalesConfig>,
    auto: bool,
    show_cursor: bool,
}

impl State {
    pub fn from_settings(settings: &Settings) -> State {
        State {
            data: SeriesSet::default(),
            error_message: None,
            x: Window::default(),
            y: Window::default(),
            scales: settings
                .scales
                .as_ref()
                .map(String::as_str)
                .map(|s| ScalesConfig::new(s).unwrap()),
            auto: true,
            show_cursor: false,
        }
    }

    // TODO: rename to paused
    pub fn is_auto(&self) -> bool {
        self.auto
    }

    pub fn pause(&mut self) -> bool {
        self.auto = !self.auto;
        true
    }

    pub fn hide_cursor(&mut self) -> bool {
        self.show_cursor = !self.show_cursor;
        true
    }

    pub fn cursor_allowed(&self) -> bool {
        self.show_cursor
    }

    pub fn append_slice(&mut self, slice: Slice, width: i64) {
        self.error_message = None;
        self.data.append_slice(slice);
        let mut xm = WindowAdjust::new(self.data.series_size(), width, &mut self.x);
        xm.on_data();
        if self.auto {
            xm.end();
        }
    }

    pub fn extend_dataset(&mut self, d: SeriesSet, width: i64) {
        self.error_message = None;
        self.data.append_set(d);
        let mut xm = WindowAdjust::new(self.data.series_size(), width, &mut self.x);
        xm.on_data();
        if self.auto {
            xm.end();
        }
    }

    pub fn on_error(&mut self, e: String) {
        self.error_message = Some(e);
    }
}
