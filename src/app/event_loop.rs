use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use failure::Error;
use termion::async_stdin;
use termion::event::{Event, Key, MouseButton, MouseEvent};
use termion::input::TermRead;

use crate::app::settings::Settings;
use crate::data::fetcher::{Fetcher, FetcherError};
use crate::data::series::SeriesSet;
use crate::data::state::State;
use crate::ui;
use crate::ui::surface::{Surface, TermSurface};

// Unit of information we pass through the main queue
pub enum Message {
    KeyPress(Key),
    MousePress((MouseButton, u16)), // button and x
    Data(SeriesSet),
    AppendData(SeriesSet),
    Tick,
    FetchError(FetcherError),
}

// Main event loop.
pub struct EventLoop {
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    state: State,
    fetcher: Fetcher,
}

impl EventLoop {
    // Entry point to main event loop.
    pub fn start(settings: Settings) -> Result<(), Error> {
        let mut terminal = ui::ui_init::init()?;
        // surface is an entity which is aware of layout, thus,
        // surface can provide information on the screen capacity.
        let mut surface = TermSurface::new(&mut terminal);

        // queue for main event loop
        let (sender, receiver) = mpsc::channel();

        let fetcher = Fetcher::new(sender.clone(), &settings);

        let mut event_loop = EventLoop {
            sender,
            receiver,
            state: State::from_settings(&settings),
            fetcher,
        };

        // keyboard and mouse listener
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

        // ticker, right now used to send 'fetch' messages to fetcher.
        let rate = settings.refresh_rate;
        event_loop.add(move |sender: mpsc::Sender<Message>| -> Result<(), Error> {
            // Fetch once anyway
            sender.send(Message::Tick)?;
            // 0 means 'never refresh', just keep tailing
            if rate.as_nanos() == 0 {
                return Ok(());
            }

            loop {
                std::thread::sleep(rate);
                sender.send(Message::Tick)?;
            }
        });

        // main event loop
        loop {
            match event_loop.receiver.recv()? {
                Message::AppendData(d) => {
                    event_loop.state.append_data(d, surface.width()?);
                    surface.render(&event_loop.state)?;
                }
                Message::Data(d) => {
                    event_loop.state.replace_data(d, surface.width()?);
                    surface.render(&event_loop.state)?;
                }
                Message::FetchError(e) => {
                    // error will be cleared on next successful data fetch
                    event_loop.state.on_error(format!("{}", e));
                    // we need to render to show 'error' to user.
                    surface.render(&event_loop.state)?;
                }
                Message::Tick => event_loop.fetcher.fetch(),
                Message::MousePress((b, x)) => {
                    if event_loop.state.on_mouse_press(
                        b,
                        x as i64,
                        surface.width()?,
                        surface.height()?,
                    ) {
                        surface.render(&event_loop.state)?;
                    }
                }
                Message::KeyPress(input) => {
                    if input == Key::Char('q') || input == Key::Esc || input == Key::Ctrl('c') {
                        break;
                    }
                    if event_loop
                        .state
                        .on_key_press(input, surface.width()?, surface.height()?)
                    {
                        surface.render(&event_loop.state)?;
                    }
                }
            }
        }

        Ok(())
    }

    // add new 'event producer'
    pub fn add<F>(&self, producer: F) -> &EventLoop
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
