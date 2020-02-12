use crate::app::event_loop::Message;
use crate::app::settings::{Column, FetchMode, Settings};
use crate::data::schema::Schema;
use crate::platform::exec::spawned_stdout;

use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

/// FetcherLoop is responsbile for setting up and maintaining
/// communication channel between main loop and data reading routines
/// It spawns a new thread where data reading will happen.
pub struct FetcherLoop {
    sender_to_fetcher: mpsc::Sender<()>,
}

impl FetcherLoop {
    pub fn new(
        sender_to_main_loop: mpsc::Sender<Message>, // where to send fetched ata
        settings: &Settings,
    ) -> FetcherLoop {
        let (sender_to_fetcher, receiver_from_main_loop) = mpsc::channel();
        let fetcher = Fetcher::new(settings, sender_to_main_loop.clone());
        thread::spawn(move || loop {
            if receiver_from_main_loop.recv().is_ok() {
                match fetcher.read() {
                    Ok(_) => {}
                    // In this case, we observe the error running the fetch command;
                    // This needs to be represented in UI, so we send 'Error' event
                    // if send itself fails, we consider that not recoverable;
                    Err(e) => sender_to_main_loop.send(Message::FetchError(e)).unwrap(),
                }
            }
        });
        FetcherLoop {
            sender_to_fetcher: sender_to_fetcher,
        }
    }
    pub fn fetch(&mut self) {
        self.sender_to_fetcher.send(()).unwrap();
    }
}

struct Fetcher {
    cmd: Option<String>,
    x: Column,
    sender_to_main_loop: mpsc::Sender<Message>,
    mode: FetchMode,
}

impl Fetcher {
    pub fn new(settings: &Settings, sender_to_main_loop: mpsc::Sender<Message>) -> Fetcher {
        Fetcher {
            cmd: settings.cmd.as_ref().map(|v| v.join(" ")),
            x: settings.x.clone(),
            sender_to_main_loop: sender_to_main_loop.clone(),
            mode: settings.fetch_mode(),
        }
    }

    fn read_from(&self, reader: impl Read) -> Result<(), FetcherError> {
        match self.mode {
            FetchMode::Incremental => self.read_lines(reader),
            FetchMode::Autorefresh(_) => self.read_all(reader),
        }
    }

    pub fn read(&self) -> Result<(), FetcherError> {
        if let Some(cmd) = self.cmd.as_ref() {
            self.read_from(spawned_stdout(&cmd)?)
        } else {
            let stdin = stdin();
            self.read_from(stdin.lock())
        }
    }

    /// Reading lines one by one, sending over as we go.
    /// We read titles first, then, after empty line, read column names again.
    fn read_lines(&self, reader: impl Read) -> Result<(), FetcherError> {
        let reader = BufReader::new(reader);

        // each iteration of a loop is a new batch/epoch
        let mut lines = reader.lines();
        while let Some(l) = lines.next() {
            // TODO: no clone
            // schema here needs to represent OLD schema + current schema
            let schema = Schema::new(self.x.clone(), l?.split(','));
            self.sender_to_main_loop
                .send(Message::ExtendDataSet(schema.empty_set()))
                .unwrap();

            loop {
                match lines.next() {
                    // This arm is 'regular data'
                    Some(Ok(l)) if l != "" => self
                        .sender_to_main_loop
                        .send(Message::DataSlice(schema.slice(l.split(','))))
                        .unwrap(),
                    // This arm is EOF or empty line
                    _ => {
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    // reads until EOF, sends single update
    fn read_all(&self, reader: impl Read) -> Result<(), FetcherError> {
        let reader = BufReader::new(reader);

        // each iteration of a loop is a new batch/epoch
        let mut lines = reader.lines();
        if let Some(l) = lines.next() {
            let schema = Schema::new(self.x.clone(), l?.split(','));
            let mut data = schema.empty_set();

            for l in lines {
                data.append_slice(schema.slice(l?.split(',')));
            }
            self.sender_to_main_loop.send(Message::Data(data)).unwrap();
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum FetcherError {
    IO(std::io::Error),
    CSV(csv::Error),
}

impl From<std::io::Error> for FetcherError {
    fn from(err: std::io::Error) -> FetcherError {
        FetcherError::IO(err)
    }
}

impl From<csv::Error> for FetcherError {
    fn from(err: csv::Error) -> FetcherError {
        FetcherError::CSV(err)
    }
}

impl std::fmt::Display for FetcherError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            FetcherError::IO(ref err) => write!(f, "IO error: {}", err),
            FetcherError::CSV(ref err) => write!(f, "CSV parse error: {}", err),
        }
    }
}
