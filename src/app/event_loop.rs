use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use failure::Error;
use termion::{
    async_stdin,
    event::{Event, Key, MouseButton, MouseEvent},
    input::TermRead,
};

use crate::{
    app::{settings::Settings, window::WindowAdjust},
    data::{
        fetcher_loop::{FetcherError, FetcherLoop},
        series::{SeriesSet, Slice},
        state::State,
    },
    ui::{
        surface::{Surface, TermSurface},
        ui_init,
    },
};

// Unit of information we pass through the main queue
pub enum Message {
    KeyPress(Key),
    MousePress((MouseButton, u16)), // button and x
    DataSlice(Slice),
    ExtendDataSet(SeriesSet),
    FetchError(FetcherError),
}

// Main event loop.
pub struct EventLoop {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    state: State,
    fetcher_loop: FetcherLoop,
}

impl EventLoop {
    fn on_mouse_press(&mut self, b: MouseButton, x: i64, w: i64, h: i64) -> bool {
        let mut xm = WindowAdjust::new(self.state.data.series_size(), w, &mut self.state.x);
        let mut ym = WindowAdjust::new(self.state.data.series_count(), h, &mut self.state.y);
        match b {
            MouseButton::WheelDown => ym.move_offset(1),
            MouseButton::WheelUp => ym.move_offset(-1),
            MouseButton::Left => xm.set_cursor(x - 1),
            _ => false,
        }
    }

    fn on_key_press(&mut self, input: Key, w: i64, h: i64) -> bool {
        let mut x = WindowAdjust::new(self.state.data.series_size(), w, &mut self.state.x);
        let mut y = WindowAdjust::new(self.state.data.series_count(), h, &mut self.state.y);

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

            Key::Char('c') => self.state.hide_cursor(),

            Key::Char('p') => {
                // TODO: this can race. We shall update UI after fetcher 'acknowledged' pause
                self.fetcher_loop.pause();
                self.state.pause()
            }
            _ => false,
        }
    }

    // Entry point to main event loop.
    pub fn start(settings: Settings) -> Result<(), Error> {
        let mut terminal = ui_init::init()?;
        // surface is an entity which is aware of layout, thus,
        // surface can provide information on the screen capacity.
        let mut surface = TermSurface::new(&mut terminal);

        // queue for main event loop
        let (sender, receiver) = mpsc::channel();

        let fetcher_loop = FetcherLoop::new(sender.clone(), &settings);

        let mut event_loop = EventLoop {
            sender,
            receiver,
            state: State::from_settings(&settings),
            fetcher_loop,
        };

        event_loop.add(move |sender: mpsc::Sender<Message>| -> Result<(), Error> {
            let mut events = async_stdin().events();
            loop {
                match events.next() {
                    Some(Ok(Event::Key(k))) => sender.send(Message::KeyPress(k))?,
                    Some(Ok(Event::Mouse(MouseEvent::Press(button, a, _)))) => {
                        sender.send(Message::MousePress((button, a)))?
                    }
                    _ => {}
                }
                thread::sleep(Duration::from_millis(50));
            }
        });

        event_loop.fetcher_loop.fetch();

        // main event loop
        loop {
            match event_loop.receiver.recv()? {
                // Append new column, with potentially new column names.
                // series might be reordered as a result of this operation.
                Message::ExtendDataSet(d) => {
                    event_loop.state.extend_dataset(d, surface.width()?);
                    surface.render(&event_loop.state)?;
                }
                // Append new slice to the existing set of columns.
                Message::DataSlice(s) => {
                    event_loop.state.append_slice(s, surface.width()?);
                    surface.render(&event_loop.state)?;
                }
                // Handle fetching error.
                Message::FetchError(e) => {
                    // error will be cleared on next successful data fetch
                    event_loop.state.on_error(format!("{}", e));
                    // we need to render to show 'error' to user.
                    surface.render(&event_loop.state)?;
                }
                // mouse event; includes both press/scroll.
                Message::MousePress((b, x)) => {
                    if event_loop.on_mouse_press(b, x as i64, surface.width()?, surface.height()?) {
                        surface.render(&event_loop.state)?;
                    }
                }
                // key events; 'exit' handled right here, everything else - in dedicated handler.
                Message::KeyPress(input) => {
                    // TODO: handle signals (including ctrl+c) properly
                    if input == Key::Char('q') || input == Key::Esc || input == Key::Ctrl('c') {
                        break;
                    }
                    if event_loop.on_key_press(input, surface.width()?, surface.height()?) {
                        surface.render(&event_loop.state)?;
                    }
                }
            }
        }

        Ok(())
    }

    // add new 'event producer'
    fn add<F>(&self, producer: F) -> &EventLoop
    where
        F: Fn(mpsc::Sender<Message>) -> Result<(), Error>,
        F: std::marker::Send,
        F: 'static,
    {
        let sender = self.sender.clone();
        thread::spawn(move || match producer(sender.clone()) {
            Ok(_) => (),
            Err(e) => eprintln!("error: {}", e),
        });
        self
    }
}
