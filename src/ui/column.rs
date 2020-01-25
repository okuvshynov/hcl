use crate::ui::style::default;
use tui::style::Color;
use tui::style::Style;

// Eventually might need to configure these
const GREEN: [Color; 6] = [
    Color::Indexed(231),
    Color::Indexed(194),
    Color::Indexed(150),
    Color::Indexed(107),
    Color::Indexed(64),
    Color::Indexed(22),
];
const RED: [Color; 6] = [
    Color::Indexed(231),
    Color::Indexed(224),
    Color::Indexed(181),
    Color::Indexed(131),
    Color::Indexed(88),
    Color::Indexed(52),
];

const SYMBOLS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub struct Column {
    pub style: Style,
    pub symbol: char,
}

impl Column {
    pub fn from_value(v: f64) -> Column {
        if v.is_nan() {
            return Column {
                style: default(),
                symbol: '!',
            };
        }
        let v = v.min(1.0).max(-1.0);
        let symbols = &SYMBOLS[..];
        let column_height = symbols.len() - 1;
        let palette = if v < 0.0 { &RED } else { &GREEN };
        let scaled_value = v.abs() * (palette.len() - 1) as f64;
        let mut color_index = scaled_value.div_euclid(1.0).round() as usize;
        let mut scaled_value =
            (scaled_value.rem_euclid(1.0) * column_height as f64).round() as usize;
        // empty at color C == full at color (C-1)
        if color_index + 1 == palette.len() && scaled_value == 0 {
            color_index -= 1;
            scaled_value = symbols.len() - 1;
        }
        let mut bg = palette[color_index];
        let mut fg = palette[color_index + 1];
        if v < 0.0 {
            std::mem::swap(&mut bg, &mut fg);
            scaled_value = column_height - scaled_value;
        }

        Column {
            style: Style::default().bg(bg).fg(fg),
            symbol: symbols[scaled_value],
        }
    }
}
