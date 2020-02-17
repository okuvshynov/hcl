use crate::app::event_loop::Message;
use crate::app::settings::{Column, FetchMode};
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
    Replace(SeriesSet),
    EOF,
}

pub struct Reader<R: Read> {
    lines: Lines<BufReader<R>>,
    mode: FetchMode,
    schema: Option<Schema>,
    x: Column,
}

impl<R: Read> Reader<R> {
    pub fn new(reader: R, mode: FetchMode, x: Column) -> Self {
        let br = BufReader::new(reader);
        Reader::<R> {
            lines: br.lines(),
            mode,
            schema: None,
            x,
        }
    }

    fn next_incremental(&mut self) -> Result<ReaderMessage, FetcherError> {
        loop {
            match self.lines.next() {
                Some(Ok(l)) => {
                    if l == "" {
                        self.schema = None;
                        continue;
                    }
                    if self.schema.is_none() {
                        self.schema = Some(Schema::new(self.x.clone(), l.split(',')));
                        return Ok(ReaderMessage::Extend(
                            self.schema.as_ref().unwrap().empty_set(),
                        ));
                    } else {
                        return Ok(ReaderMessage::Append(
                            self.schema.as_ref().unwrap().slice(l.split(',')),
                        ));
                    }
                }
                _ => return Ok(ReaderMessage::EOF),
            }
        }
    }

    fn next_full(&mut self) -> Result<ReaderMessage, FetcherError> {
        if let Some(l) = self.lines.next() {
            let schema = Schema::new(self.x.clone(), l?.split(','));
            let mut data = schema.empty_set();

            loop {
                match self.lines.next() {
                    Some(l) => data.append_slice(schema.slice(l?.split(','))),
                    None => break,
                }
            }
            Ok(ReaderMessage::Replace(data))
        } else {
            Ok(ReaderMessage::EOF)
        }
    }

    pub fn next(&mut self) -> Result<ReaderMessage, FetcherError> {
        match self.mode {
            FetchMode::Incremental => self.next_incremental(),
            FetchMode::Autorefresh(_) => self.next_full(),
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
        let mut r = Reader::new(reader, FetchMode::Incremental, settings.x.clone());
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
                ReaderMessage::Replace(set) => to_main_loop.send(Message::Data(set)).unwrap(),
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
}

impl Fetcher for ContinuousFetcher {
    fn fetcher_loop(
        &self,
        settings: FetcherSettings,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: mpsc::Sender<Message>,
    ) {
        std::thread::spawn(move || {
            let res = if let Some(cmd) = settings.cmd.as_ref() {
                ContinuousFetcher::read_from(
                    &settings,
                    spawned_stdout(&cmd).unwrap(),
                    from_main_loop,
                    &to_main_loop,
                )
            } else {
                let stdin = stdin();
                ContinuousFetcher::read_from(&settings, stdin.lock(), from_main_loop, &to_main_loop)
            };
            if let Err(e) = res {
                to_main_loop.send(Message::FetchError(e)).unwrap();
            }
        });
    }
}
