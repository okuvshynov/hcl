use crate::data::scale::Scale;
use crate::data::state::State;
use crate::ui::column::Column;
use crate::ui::style::{default, EmptyBox};

use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::Widget;

pub struct Charts<'a> {
    state: &'a State,
}

impl<'a> Charts<'a> {
    pub const SERIES_HEIGHT: i64 = 2;
    pub fn new(state: &'a State) -> Self {
        Charts { state }
    }
}

impl<'a> Widget for Charts<'a> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        EmptyBox::fill(area, buf);

        // effective width, as 1 character is used for the markers.
        let w = area.width - 1;

        let render_cursor = |x: u16, y: u16, label: &str, symbol: &str, buf: &mut Buffer| {
            if x * 2 < w {
                buf.set_string(x, y, format!("{}{}", symbol, label), default());
            } else {
                let cursor = format!("{}{}", label, symbol);
                buf.set_string(x - cursor.len() as u16 + 1, y, cursor, default());
            }
        };

        let data = self.state.history.current();

        let scales = self.state.scales.as_ref().map(|s| s.materialize(&data.y));

        // charts
        data.y
            .iter()
            .skip(self.state.y.offset as usize)
            .take(area.height as usize / 2)
            .enumerate()
            .for_each(|(i, series)| {
                let scale = scales
                    .as_ref()
                    .and_then(|scales| scales.pick(&series.title))
                    .unwrap_or_else(|| Scale::auto(&series.values[..]));

                let y = area.top() + i as u16 * 2;

                buf.set_string(area.left(), y, format!("┌{}", series.title), default());
                buf.set_string(area.left(), y + 1, "└", default());

                series
                    .values
                    .iter()
                    .skip(self.state.x.offset as usize)
                    .take(w as usize)
                    .enumerate()
                    .for_each(|(j, v)| {
                        let c = Column::from_value(scale.run(*v));
                        buf.get_mut(area.left() + 1 + j as u16, y + 1)
                            .set_style(c.style)
                            .set_char(c.symbol);

                        // draw cursor
                        if j == self.state.x.cursor as usize {
                            render_cursor(
                                area.left() + 1 + j as u16,
                                y,
                                &format!("{:.3}", *v),
                                "|",
                                buf,
                            );
                        }
                    });
            });

        // x axis
        if let Some((_, x)) = &data.x {
            let from = self.state.x.offset as usize;
            let to = std::cmp::min(self.state.x.offset as usize + w as usize, x.len());
            let xx = &x[from..to];
            if let Some(xv) = xx.first() {
                let symbol = if from > 0 { "<" } else { "|" };
                render_cursor(area.left() + 1, area.bottom() - 1, xv, symbol, buf);
            }
            if let Some(xv) = xx.last() {
                let symbol = if to < x.len() { ">" } else { "|" };
                render_cursor(
                    area.left() + xx.len() as u16,
                    area.bottom() - 1,
                    xv,
                    symbol,
                    buf,
                );
            }

            if xx.len() > self.state.x.cursor as usize {
                render_cursor(
                    area.left() + 1 + self.state.x.cursor as u16,
                    area.bottom() - 1,
                    &xx[self.state.x.cursor as usize],
                    "|",
                    buf,
                );
            }
        }
    }
}
