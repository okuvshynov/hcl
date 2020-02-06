use crate::data::series::SeriesSet;

/// represents a group of series set. 
/// Terminology gets a little confusing, so
///  -- Series == one line on a screen. For example 'cpu load'
///  -- SeriesSet == whole screen -- multiple series, X axis
///  -- History == several 'screens', which one can navigate with t/T
///  -- Elements of history are not updated, but can be added to the end.
#[derive(Debug)]
pub struct History {
    sets: Vec<SeriesSet>,
    index: usize,
}

impl History {
    pub fn new() -> History {
        History {
            sets: vec![SeriesSet::default()],
            index: 0,
        }
    }

    pub fn append(&mut self, set: SeriesSet) {
        self.sets.push(set);
    }

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

    pub fn current<'a>(&'a self) -> &'a SeriesSet {
        &self.sets[self.index]
    }

    pub fn current_mut<'a>(&'a mut self) -> &'a mut SeriesSet {
        &mut self.sets[self.index]
    }

    pub fn replace_current(&mut self, s: SeriesSet) {
        self.sets[self.index] = s;
    }
}

