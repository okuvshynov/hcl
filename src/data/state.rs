use crate::app::settings::Settings;
use crate::app::window::{Window, WindowAdjust};
use crate::data::history::History;
use crate::data::scale_config::ScalesConfig;
use crate::data::series::{SeriesSet, Slice};

use termion::event::{Key, MouseButton};

#[derive(Debug)]
pub struct State {
    pub history: History,
    pub error_message: Option<String>,
    pub x: Window,
    pub y: Window,
    pub scales: Option<ScalesConfig>,
    auto: bool,
}

impl State {
    pub fn from_settings(settings: &Settings) -> State {
        State {
            history: History::new(),
            error_message: None,
            x: Window::default(),
            y: Window::default(),
            scales: settings
                .scales
                .as_ref()
                .map(String::as_str)
                .map(|s| ScalesConfig::new(s).unwrap()),
            auto: true,
        }
    }

    pub fn is_auto(&self) -> bool {
        self.auto
    }

    pub fn pause(&mut self) -> bool {
        self.auto = !self.auto;
        true
    }

    pub fn append_slice(&mut self, slice: Slice, width: i64) {
        self.error_message = None;
        self.history.append_slice(slice);
        let mut xm = WindowAdjust::new(self.history.current().series_size(), width, &mut self.x);
        xm.on_data();
        if self.auto {
            xm.end();
        }
    }

    pub fn append_dataset(&mut self, d: SeriesSet, width: i64) {
        self.error_message = None;
        self.history.append(d);
        if self.is_auto() {
            self.history.last();
            let mut xm =
                WindowAdjust::new(self.history.current().series_size(), width, &mut self.x);
            xm.on_data();
            xm.end();
        }
    }

    pub fn replace_data(&mut self, d: SeriesSet, width: i64) {
        self.error_message = None;
        self.history.replace_current(d);
        let mut xm = WindowAdjust::new(self.history.current().series_size(), width, &mut self.x);
        xm.on_data();
        if self.auto {
            xm.end();
        }
    }

    pub fn on_error(&mut self, e: String) {
        self.error_message = Some(e);
    }

    pub fn on_mouse_press(&mut self, b: MouseButton, x: i64, w: i64, h: i64) -> bool {
        let data = self.history.current();
        let mut xm = WindowAdjust::new(data.series_size(), w, &mut self.x);
        let mut ym = WindowAdjust::new(data.series_count(), h, &mut self.y);
        match b {
            MouseButton::WheelDown => ym.move_offset(1),
            MouseButton::WheelUp => ym.move_offset(-1),
            MouseButton::Left => xm.set_cursor(x - 1),
            _ => false,
        }
    }

    pub fn on_key_press(&mut self, input: Key, w: i64, h: i64) -> bool {
        let data = self.history.current();
        let mut x = WindowAdjust::new(data.series_size(), w, &mut self.x);
        let mut y = WindowAdjust::new(data.series_count(), h, &mut self.y);

        match input {
            // vertical navigation
            Key::Down | Key::Char('j') => y.move_offset(1),
            Key::Up | Key::Char('k') => y.move_offset(-1),
            Key::Char('g') => y.begin(),
            Key::Char('G') => y.end(),
            Key::Ctrl('b') => y.pageup(),
            Key::Ctrl('f') => y.pagedown(),
            Key::Ctrl('u') => y.halfpageup(),
            Key::Ctrl('d') => y.halfpagedown(),

            // horizontal navigation
            Key::Right | Key::Char('l') => x.move_cursor(1),
            Key::Left | Key::Char('h') => x.move_cursor(-1),
            Key::Ctrl('l') => x.move_offset(1),
            Key::Ctrl('h') => x.move_offset(-1),

            Key::Char('H') => x.cursor_begin(),
            Key::Char('L') => x.cursor_end(),

            Key::Char('$') => (x.end() || x.cursor_end()),
            Key::Char('0') => (x.begin() || x.cursor_begin()),

            // time/epoch navigation
            Key::Char('t') => self.history.forward(),
            Key::Char('T') => self.history.backward(),

            // controls
            Key::Char('p') => self.pause(),
            _ => false,
        }
    }
}
