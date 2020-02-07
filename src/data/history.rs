use crate::data::series::{SeriesSet, Slice};

/// represents a group of series set.
/// Terminology gets a little confusing, so
///  * Series == one line on a screen. For example 'cpu load';
///  * SeriesSet == whole screen -- multiple series, X axis;
///  * History == several 'screens', which one can navigate with t/T.
///
/// Elements of history are not updated, but can be added to the end.
/// Important question: do we make history empty originally, or create a
/// void SeriesSet? In incremental and autorefresh modes, having 'void series set'
/// from the start makes a lot of sense, but for batch mode, it makes it a little
/// ugly, with extra 'empty' field which is confusing and, in some sense, duplicating
/// information.
#[derive(Debug)]
pub struct History {
    sets: Vec<SeriesSet>,
    index: usize,
    empty: bool,
}

impl History {
    pub fn new() -> History {
        History {
            sets: vec![SeriesSet::default()],
            index: 0,
            empty: true,
        }
    }

    pub fn current<'a>(&'a self) -> &'a SeriesSet {
        &self.sets[self.index]
    }

    // These functions are mutating 'current index', but not data itself.
    pub fn forward(&mut self) -> bool {
        if self.index + 1 < self.sets.len() {
            self.index += 1;
            return true;
        }
        false
    }

    pub fn backward(&mut self) -> bool {
        if self.index > 0 {
            self.index -= 1;
            return true;
        }
        false
    }

    pub fn last(&mut self) -> bool {
        if self.index + 1 != self.sets.len() {
            self.index = self.sets.len() - 1;
            return true;
        }
        return false;
    }

    // these methods are mutating the data
    pub fn append_slice(&mut self, s: Slice) {
        self.sets[self.index].append_slice(s);
        self.empty = false;
    }

    pub fn replace_current(&mut self, s: SeriesSet) {
        self.sets[self.index] = s;
        self.empty = false;
    }

    pub fn append(&mut self, set: SeriesSet) {
        if self.empty {
            self.replace_current(set);
        } else {
            self.sets.push(set);
        }
    }
}
