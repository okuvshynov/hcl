use std::cmp::{max, min};

// Window is a current 'view' of a data
// it is part of a state, thus it won't be updated immediatedly on
// new data arrival or screen resize.
#[derive(Debug)]
pub struct Window {
    // offset represents the index of data item at screen leftmost character
    pub offset: i64,

    // cursor is the coordinate relative to screen left border
    pub cursor: i64,
}

impl Default for Window {
    fn default() -> Window {
        Window {
            offset: 0,
            // some large number to put cursor to the right
            cursor: 1024 * 1024,
        }
    }
}

// WindowAdjust is a short-lived object which combines
// infomation on 'app settings' (= window) and externals -
// screen size and new data. It is responsible for eventually
// updating the window and handles navigation requests.
pub struct WindowAdjust<'a> {
    data_size: i64, // data length
    view_size: i64, // how large is the view window
    window: &'a mut Window,
}

impl<'a> WindowAdjust<'a> {
    pub fn halfpageup(&mut self) -> bool {
        self.move_offset(-self.view_size / 2)
    }

    pub fn halfpagedown(&mut self) -> bool {
        self.move_offset(self.view_size / 2)
    }

    pub fn pageup(&mut self) -> bool {
        self.move_offset(-self.view_size)
    }

    pub fn pagedown(&mut self) -> bool {
        self.move_offset(self.view_size)
    }

    pub fn begin(&mut self) -> bool {
        self.set_offset(0)
    }

    pub fn end(&mut self) -> bool {
        self.set_offset(self.data_size - self.view_size)
    }

    pub fn set_offset(&mut self, o: i64) -> bool {
        // check that cursor 'make sense'
        let cursor = self.move_cursor(0);
        let o = max(min(self.data_size - self.view_size, o), 0);
        if o != self.window.offset {
            self.window.offset = o;
            true
        } else {
            cursor
        }
    }

    pub fn move_offset(&mut self, delta: i64) -> bool {
        self.set_offset(self.window.offset + delta)
    }

    // last column on a screen.
    pub fn cursor_end(&mut self) -> bool {
        self.set_cursor(self.view_size - 1)
    }

    pub fn cursor_begin(&mut self) -> bool {
        self.set_cursor(0)
    }

    pub fn set_cursor(&mut self, c: i64) -> bool {
        // valid range for cursor is
        // [- offset; data_size - offset - 1]
        let c = max(
            -self.window.offset,
            min(c, self.data_size - self.window.offset - 1),
        );
        if c < 0 {
            // moving window as well;
            self.window.cursor = 0;
            self.window.offset += c;
            return true;
        }
        if c >= self.view_size {
            self.window.cursor = self.view_size - 1;
            self.window.offset += c - self.window.cursor;
            return true;
        }
        if c != self.window.cursor {
            self.window.cursor = c;
            true
        } else {
            false
        }
    }

    pub fn move_cursor(&mut self, delta: i64) -> bool {
        self.set_cursor(self.window.cursor + delta)
    }

    // this runs on new data arrival;
    // if new data is significantly different (or even empty)
    // offset and cursor might not be valid anymore
    pub fn on_data(&mut self) {
        self.move_offset(0);
        self.move_cursor(0);
    }

    pub fn new(data_size: i64, view_size: i64, window: &'a mut Window) -> Self {
        WindowAdjust {
            data_size,
            view_size,
            window,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_move() {
        let mut w = Window::default();
        let mut m = WindowAdjust::new(100, 10, &mut w);
        assert_eq!(m.move_cursor(-1), true);
        assert_eq!(m.move_cursor(1), false);
        assert_eq!(m.window.cursor, 9);
        assert_eq!(m.move_cursor(10000), false);
        assert_eq!(m.window.cursor, 9);
        assert_eq!(m.move_cursor(-100), true);
        assert_eq!(m.window.cursor, 0);
    }

    #[test]
    fn cursor_move_small_data() {
        let mut w = Window::default();
        let mut m = WindowAdjust::new(1, 10, &mut w);
        assert_eq!(m.move_cursor(-1), true);
        assert_eq!(m.move_cursor(1), false);
        assert_eq!(m.window.cursor, 0);
        assert_eq!(m.move_cursor(-1), false);
        assert_eq!(m.window.cursor, 0);
    }

    #[test]
    fn window_move() {
        let mut w = Window::default();
        let mut m = WindowAdjust::new(20, 10, &mut w);
        assert_eq!(m.move_offset(-1), true);
        assert_eq!(m.move_offset(1), true);
        assert_eq!(m.window.cursor, 9);
        assert_eq!(m.window.offset, 1);
        assert_eq!(m.move_offset(100), true);
        assert_eq!(m.window.cursor, 9);
        assert_eq!(m.window.offset, 10);
        assert_eq!(m.move_offset(-1), true);
        assert_eq!(m.window.cursor, 9);
        assert_eq!(m.window.offset, 9);
        assert_eq!(m.move_offset(-10), true);
        assert_eq!(m.window.cursor, 9);
        assert_eq!(m.window.offset, 0);
    }
}
