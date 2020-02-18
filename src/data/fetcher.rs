use crate::app::event_loop::Message;
use crate::app::settings::{Column, FetchMode, Settings};
use crate::data::continuous_fetcher::ContinuousFetcher;
use crate::data::on_demand_fetcher::OnDemandFetcher;

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
    pub cmd: Option<String>,
    pub x: Column,
    pub paired: bool,
}

impl FetcherLoop {
    pub fn new(
        to_main_loop: mpsc::Sender<Message>, // where to send fetched data
        settings: &Settings,
    ) -> FetcherLoop {
        let (to_fetcher, from_main_loop) = mpsc::channel();
        let fetcher = fetcher_factory(settings);
        let fetcher_settings = FetcherSettings {
            cmd: settings.cmd.as_ref().map(|v| v.join(" ")),
            x: settings.x.clone(),
            paired: settings.paired,
        };
        fetcher.fetcher_loop(fetcher_settings, from_main_loop, to_main_loop.clone());
        FetcherLoop {
            sender_to_fetcher: to_fetcher,
        }
    }
    pub fn fetch(&mut self) {
        self.sender_to_fetcher.send(FetcherEvent::Tick).unwrap();
    }

    pub fn pause(&mut self) {
        self.sender_to_fetcher.send(FetcherEvent::Pause).unwrap();
    }
}

pub trait Fetcher {
    fn fetcher_loop(
        &self,
        fetcher_settings: FetcherSettings,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: mpsc::Sender<Message>,
    );
}

fn fetcher_factory(settings: &Settings) -> Box<dyn Fetcher> {
    if settings.fetch_mode() == FetchMode::Incremental {
        return Box::new(ContinuousFetcher::new());
    } else {
        return Box::new(OnDemandFetcher::new());
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
