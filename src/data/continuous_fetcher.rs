use crate::app::event_loop::Message;
use crate::app::settings::Column;
use crate::data::fetcher::{Fetcher, FetcherError, FetcherEvent, FetcherSettings};
use crate::data::schema::Schema;
use crate::data::series::{SeriesSet, Slice};

use std::fs::File;
use std::io::stdin;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Lines;
use std::io::Read;
use std::sync::mpsc;

trait Reader {
    fn next(&mut self) -> Result<ReaderMessage, FetcherError>;
}

pub enum ReaderMessage {
    Extend(SeriesSet),
    Append(Slice),
    EOF,
}

// Right now reader and schema both maintain 'state' in reading FSM
// This needs to be improved, as we start loading supporting other input
// formats.
// First, let's build a 'pair reader'. Format will be
// title: value\ntitle: value\n, with empty line being a 'end of column' signal.

pub struct PairReader<R: Read> {
    lines: Lines<BufReader<R>>,
    x: Column,
}

impl<R: Read> PairReader<R> {
    pub fn new(reader: R, x: Column) -> Self {
        let br = BufReader::new(reader);
        PairReader::<R> {
            lines: br.lines(),
            x,
        }
    }
}

impl<R: Read> Reader for PairReader<R> {
    fn next(&mut self) -> Result<ReaderMessage, FetcherError> {
        let mut titles = vec![];
        let mut values = vec![];
        loop {
            match self.lines.next() {
                Some(Ok(l)) => {
                    if l != "" {
                        let mut parts = l.split(':').take(2);
                        match (parts.next(), parts.next()) {
                            (Some(title), Some(value)) => {
                                titles.push(title.to_owned());
                                values.push(value.to_owned());
                            }
                            _ => {}
                        };
                    } else {
                        // TODO: cleanup
                        // empty line, flush
                        if titles.len() > 0 {
                            let schema = Schema::from_title_range(self.x.clone(), &titles);
                            let mut data = schema.empty_set();
                            data.append_slice(schema.slice_from_range(&values));
                            return Ok(ReaderMessage::Extend(data));
                        } else {
                            return Ok(ReaderMessage::Extend(SeriesSet::default()));
                        }
                    }
                }
                _ => {
                    if titles.len() > 0 {
                        let schema = Schema::from_title_range(self.x.clone(), &titles);
                        let mut data = schema.empty_set();
                        data.append_slice(schema.slice_from_range(&values));
                        return Ok(ReaderMessage::Extend(data));
                    } else {
                        return Ok(ReaderMessage::EOF);
                    }
                }
            }
        }
    }
}

pub struct LineReader<R: Read> {
    lines: Lines<BufReader<R>>,
    schema: Option<Schema>,
    x: Column,
}

impl<R: Read> LineReader<R> {
    pub fn new(reader: R, x: Column) -> Self {
        let br = BufReader::new(reader);
        LineReader::<R> {
            lines: br.lines(),
            schema: None,
            x,
        }
    }
}

impl<R: Read> Reader for LineReader<R> {
    fn next(&mut self) -> Result<ReaderMessage, FetcherError> {
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

    fn loop_with_reader(
        mut reader: impl Reader,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: &mpsc::Sender<Message>,
    ) -> Result<(), FetcherError> {
        loop {
            ContinuousFetcher::check_pause(&from_main_loop);
            match reader.next()? {
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

    fn read_from(
        settings: &FetcherSettings,
        reader: impl Read,
        from_main_loop: mpsc::Receiver<FetcherEvent>,
        to_main_loop: &mpsc::Sender<Message>,
    ) -> Result<(), FetcherError> {
        if settings.paired {
            Self::loop_with_reader(
                PairReader::new(reader, settings.x.clone()),
                from_main_loop,
                to_main_loop,
            )
        } else {
            Self::loop_with_reader(
                LineReader::new(reader, settings.x.clone()),
                from_main_loop,
                to_main_loop,
            )
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
        if let Some(input_file) = settings.input_file.as_ref() {
            ContinuousFetcher::read_from(
                &settings,
                File::open(&input_file)?,
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
