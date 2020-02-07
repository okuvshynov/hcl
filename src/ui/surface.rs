use crate::app::settings::Settings;
use crate::data::state::State;
use crate::ui::chart::Charts;
use crate::ui::status_bar::StatusBar;
use crate::ui::style::EmptyBox;

use failure::Error;

use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::Widget;

pub trait Surface {
    fn height(&self) -> Result<i64, Error>;
    fn width(&self) -> Result<i64, Error>;
    fn render(&mut self, state: &State, settings: &Settings) -> Result<(), Error>;
}

// drawing surface; is aware of terminal size and layout
pub struct TermSurface<'a, B>
where
    B: Backend,
{
    terminal: &'a mut tui::Terminal<B>,
}

impl<'a, B> TermSurface<'a, B>
where
    B: Backend,
{
    pub fn new(terminal: &'a mut tui::Terminal<B>) -> Self {
        TermSurface { terminal }
    }
}

impl<'a, B> Surface for TermSurface<'a, B>
where
    B: Backend,
{
    // height in series. How many series will fit?
    fn height(&self) -> Result<i64, Error> {
        Ok((self.terminal.size()?.height as i64 - 2) / Charts::SERIES_HEIGHT)
    }

    fn width(&self) -> Result<i64, Error> {
        Ok(self.terminal.size()?.width as i64 - 1)
    }

    fn render(&mut self, state: &State, settings: &Settings) -> Result<(), Error> {
        let data = state.history.current();
        let mut data = &data.y[state.y.offset as usize..data.y.len()];

        let h = self.height()? as usize;
        if h < data.len() {
            data = &data[0..h];
        }

        let mut status_bar = StatusBar::new(
            state,
            settings,
            (
                state.y.offset as usize,
                state.y.offset as usize + data.len(),
            ),
        );

        let mut constraints = vec![];
        // x axis + all series
        constraints.push(Constraint::Length(
            1 + Charts::SERIES_HEIGHT as u16 * data.len() as u16,
        ));
        constraints.push(Constraint::Min(0));
        constraints.push(Constraint::Length(1)); // status bar
        self.terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(f.size());

            Charts::new(&state).render(&mut f, chunks[0]);
            status_bar.render(&mut f, chunks[2]);
            EmptyBox {}.render(&mut f, chunks[1]);
        })?;
        Ok(())
    }
}
