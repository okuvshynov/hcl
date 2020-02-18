use crate::app::event_loop::Message;
use crate::data::fetcher::{Fetcher, FetcherError, FetcherEvent, FetcherSettings};
use crate::data::schema::Schema;
use crate::data::series::SeriesSet;
use crate::platform::exec::spawned_stdout;

use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::sync::mpsc;

// waits for 'fetch' event to re-read the data;
// reads data until EOF and sends complete dataset
// to main loop.
pub struct OnDemandFetcher {}

impl OnDemandFetcher {
    pub fn new() -> OnDemandFetcher {
        OnDemandFetcher {}
    }
    fn read(settings: &FetcherSettings) -> Result<SeriesSet, FetcherError> {
        if let Some(cmd) = &settings.cmd.as_ref() {
            let mut lines = BufReader::new(spawned_stdout(cmd)?).lines();
            if let Some(l) = lines.next() {
                let schema = Schema::from_titles(settings.x.clone(), &l?);
                let mut data = schema.empty_set();

                for l in lines {
                    data.append_slice(schema.slice(&l?));
                }
                return Ok(data);
            }
            // return empty data vs no data?
            return Ok(SeriesSet::default());
        }
        return Err(FetcherError::IO(Error::new(
            ErrorKind::NotFound,
            "command not found",
        )));
    }
}

impl Fetcher for OnDemandFetcher {
    fn fetcher_loop(
        &self,
        settings: FetcherSettings,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: mpsc::Sender<Message>,
    ) {
        std::thread::spawn(move || {
            let mut paused = false;
            loop {
                match from_main_loop.recv().unwrap() {
                    FetcherEvent::Pause => paused = !paused,
                    FetcherEvent::Tick => {
                        if !paused {
                            match OnDemandFetcher::read(&settings) {
                                Ok(set) => to_main_loop.send(Message::Data(set)).unwrap(),
                                Err(e) => to_main_loop.send(Message::FetchError(e)).unwrap(),
                            }
                        }
                    }
                }
            }
        });
    }
}
