use crate::app::event_loop::Message;
use crate::app::settings::Column;
use crate::data::fetcher::{Fetcher, FetcherError, FetcherEvent, FetcherSettings};
use crate::data::schema::Schema;
use crate::data::series::{SeriesSet, Slice};
use crate::platform::exec::spawned_stdout;

use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::io::Read;
use std::sync::mpsc;

pub enum ReaderMessage {
    Extend(SeriesSet),
    Append(Slice),
    EOF,
}

pub struct Reader<R: Read> {
    lines: Lines<BufReader<R>>,
    schema: Option<Schema>,
    x: Column,
}

impl<R: Read> Reader<R> {
    pub fn new(reader: R, x: Column) -> Self {
        let br = BufReader::new(reader);
        Reader::<R> {
            lines: br.lines(),
            schema: None,
            x,
        }
    }

    pub fn next(&mut self) -> Result<ReaderMessage, FetcherError> {
        loop {
            match self.lines.next() {
                Some(Ok(l)) => {
                    if l == "" {
                        self.schema = None;
                        continue;
                    }
                    match self.schema.as_ref() {
                        None => {
                            self.schema = Some(Schema::from_titles(self.x.clone(), &l));
                            return Ok(ReaderMessage::Extend(
                                self.schema.as_ref().unwrap().empty_set(),
                            ));
                        }
                        Some(schema) => return Ok(ReaderMessage::Append(schema.slice(&l))),
                    }
                }
                _ => return Ok(ReaderMessage::EOF),
            }
        }
    }
}

pub struct ContinuousFetcher {}

impl ContinuousFetcher {
    pub fn new() -> ContinuousFetcher {
        ContinuousFetcher {}
    }
    fn read_from(
        settings: &FetcherSettings,
        reader: impl Read,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: &mpsc::Sender<Message>,
    ) -> Result<(), FetcherError> {
        let mut r = Reader::new(reader, settings.x.clone());
        loop {
            ContinuousFetcher::check_pause(&from_main_loop);
            match r.next()? {
                ReaderMessage::EOF => return Ok(()),
                ReaderMessage::Append(slice) => {
                    to_main_loop.send(Message::DataSlice(slice)).unwrap()
                }
                ReaderMessage::Extend(set) => {
                    to_main_loop.send(Message::ExtendDataSet(set)).unwrap()
                }
            }
        }
    }

    // checks if pause was received and blocks until unpaused
    fn check_pause(from_main_loop: &mpsc::Receiver<FetcherEvent>) {
        if let Ok(FetcherEvent::Pause) = from_main_loop.try_recv() {
            loop {
                if let Ok(FetcherEvent::Pause) = from_main_loop.recv() {
                    break;
                }
            }
        }
    }

    fn read(
        settings: FetcherSettings,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: &mpsc::Sender<Message>,
    ) -> Result<(), FetcherError> {
        if let Some(cmd) = settings.cmd.as_ref() {
            ContinuousFetcher::read_from(
                &settings,
                spawned_stdout(&cmd)?,
                from_main_loop,
                &to_main_loop,
            )
        } else {
            let stdin = stdin();
            ContinuousFetcher::read_from(&settings, stdin.lock(), from_main_loop, &to_main_loop)
        }
    }
}

impl Fetcher for ContinuousFetcher {
    fn fetcher_loop(
        &self,
        settings: FetcherSettings,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: mpsc::Sender<Message>,
    ) {
        std::thread::spawn(move || {
            if let Err(e) = ContinuousFetcher::read(settings, from_main_loop, &to_main_loop) {
                to_main_loop.send(Message::FetchError(e)).unwrap();
            }
        });
    }
}
