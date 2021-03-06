use crate::app::event_loop::Message;
use crate::app::settings::{Column, Settings};
use crate::data::fetcher::Fetcher;

use std::sync::mpsc;

pub enum FetcherEvent {
    Tick,
    Pause,
}

/// FetcherLoop is responsbile for setting up and maintaining
/// communication channel between main loop and data reading routines
/// It spawns a new thread where data reading will happen.
pub struct FetcherLoop {
    sender_to_fetcher: mpsc::Sender<FetcherEvent>,
}

pub struct FetcherSettings {
    pub input_file: Option<String>,
    pub x: Column,
    pub paired: bool,
}

impl FetcherLoop {
    pub fn new(
        to_main_loop: mpsc::Sender<Message>, // where to send fetched data
        settings: &Settings,
    ) -> FetcherLoop {
        let (to_fetcher, from_main_loop) = mpsc::channel();
        let fetcher = Fetcher::new();
        let fetcher_settings = FetcherSettings {
            input_file: settings.input_file.clone(),
            x: settings.x.clone(),
            paired: settings.paired,
        };
        fetcher.fetcher_loop(fetcher_settings, from_main_loop, to_main_loop.clone());
        FetcherLoop {
            sender_to_fetcher: to_fetcher,
        }
    }
    pub fn fetch(&mut self) {
        if let Err(_) = self.sender_to_fetcher.send(FetcherEvent::Tick) {
            // TODO: fetching done. Update status to done
        }
    }

    pub fn pause(&mut self) {
        if let Err(_) = self.sender_to_fetcher.send(FetcherEvent::Pause) {
            // TODO: fetching done. Update status to done
        }
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
