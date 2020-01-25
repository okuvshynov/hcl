use crate::app::event_loop::Message;
use crate::app::settings::{Settings, XColumn};
use crate::data::series_collector::SeriesCollector;
use crate::platform::exec::spawned_stdout;

use std::io::stdin;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

/// Fetcher is responsbile for setting up and maintaining
/// communication channel between main loop and data reading routines
/// It spawns a new thread where data reading will happen.
pub struct Fetcher {
    sender_to_fetcher: mpsc::Sender<()>,
}

impl Fetcher {
    pub fn new(
        sender_to_main_loop: mpsc::Sender<Message>, // where to send fetched ata
        settings: &Settings,
    ) -> Fetcher {
        let (sender_to_fetcher, receiver_from_main_loop) = mpsc::channel();
        let mut reader = Reader::new(settings, sender_to_main_loop.clone());
        thread::spawn(move || loop {
            if receiver_from_main_loop.recv().is_ok() {
                match reader.read() {
                    Ok(_) => {}
                    // In this case, we observe the error running the fetch command;
                    // This needs to be represented in UI, so we send 'Error' event
                    // if send itself fails, we consider that not recoverable;
                    Err(e) => sender_to_main_loop.send(Message::FetchError(e)).unwrap(),
                }
            }
        });
        Fetcher {
            sender_to_fetcher: sender_to_fetcher,
        }
    }
    pub fn fetch(&mut self) {
        self.sender_to_fetcher.send(()).unwrap();
    }
}

struct Reader {
    cmd: Option<String>,
    x: XColumn,
    sender_to_main_loop: mpsc::Sender<Message>,
    batch_mode: bool,
}

impl Reader {
    pub fn new(settings: &Settings, sender_to_main_loop: mpsc::Sender<Message>) -> Reader {
        Reader {
            cmd: settings.cmd.as_ref().map(|v| v.join(" ")),
            x: settings.x.clone(),
            sender_to_main_loop: sender_to_main_loop.clone(),
            batch_mode: settings.refresh_rate.as_nanos() > 0,
        }
    }

    pub fn read(&mut self) -> Result<(), FetcherError> {
        match (self.cmd.as_ref(), self.batch_mode) {
            (Some(cmd), true) => self.read_once(spawned_stdout(&cmd)?),
            (Some(cmd), false) => self.keep_reading(spawned_stdout(&cmd)?),
            (None, _) => {
                let stdin = stdin();
                self.keep_reading(stdin.lock())
            }
        }
    }

    // reads and sends updates after each line read
    fn keep_reading(&self, reader: impl Read) -> Result<(), FetcherError> {
        let mut reader = csv::Reader::from_reader(reader);
        let mut collector = SeriesCollector::from_titles(reader.headers()?.iter(), &self.x);

        let mut x = 0;

        for res in reader.records() {
            collector.one_row(res?.iter());
            let mut data = collector.current();

            if data.x == None {
                data.x = Some(("index".to_owned(), vec![format!("{}", x)]));
                x += 1;
            }
            self.sender_to_main_loop
                .send(Message::AppendData(data))
                .unwrap();
        }

        Ok(())
    }

    // reads until EOF, collects to internal series set and sends whole thing.
    fn read_once(&self, reader: impl Read) -> Result<(), FetcherError> {
        let mut reader = csv::Reader::from_reader(reader);
        let mut collector = SeriesCollector::from_titles(reader.headers()?.iter(), &self.x);

        for res in reader.records() {
            collector.append(res?.iter());
        }

        let mut data = collector.current();

        if data.x == None {
            data.x = Some((
                "index".to_owned(),
                (0..data.series_size()).map(|v| format!("{}", v)).collect(),
            ));
        }

        self.sender_to_main_loop.send(Message::Data(data)).unwrap();

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
