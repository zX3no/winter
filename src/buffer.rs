use crate::{layout::Rect, move_to, reset, BgColor, Color, Modifier, Style};
use std::io::Write;
use unicode_width::UnicodeWidthStr;

#[derive(Debug)]
pub struct Buffer {
    /// The area represented by this buffer
    pub area: Rect,
    /// The content of the buffer. The length of this Vec should always be equal to area.width *
    /// area.height
    pub content: Vec<Cell>,
}

impl Buffer {
    pub fn empty(area: Rect) -> Self {
        let size = area.area() as usize;
        let mut content = Vec::with_capacity(size);
        for _ in 0..size {
            content.push(Cell::default());
        }
        Self { area, content }
    }
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let i = self.index_of(x, y);
        &mut self.content[i]
    }
    pub fn index_of(&self, x: u16, y: u16) -> usize {
        debug_assert!(
            x >= self.area.left()
                && x < self.area.right()
                && y >= self.area.top()
                && y < self.area.bottom(),
            "Trying to access position outside the buffer: x={}, y={}, area={:?}",
            x,
            y,
            self.area
        );
        ((y - self.area.y) * self.area.width + (x - self.area.x)) as usize
    }
    pub fn diff<'a>(&self, other: &'a Buffer) -> Vec<(u16, u16, &'a Cell)> {
        let previous_buffer = &self.content;
        let next_buffer = &other.content;
        let width = self.area.width;

        let mut updates: Vec<(u16, u16, &Cell)> = vec![];
        // Cells invalidated by drawing/replacing preceeding multi-width characters:
        let mut invalidated: usize = 0;
        // Cells from the current buffer to skip due to preceeding multi-width characters taking their
        // place (the skipped cells should be blank anyway):
        let mut to_skip: usize = 0;
        for (i, (current, previous)) in next_buffer.iter().zip(previous_buffer.iter()).enumerate() {
            if (current != previous || invalidated > 0) && to_skip == 0 {
                let x = i as u16 % width;
                let y = i as u16 / width;
                updates.push((x, y, &next_buffer[i]));
            }

            to_skip = current.symbol.width().saturating_sub(1);

            let affected_width = std::cmp::max(current.symbol.width(), previous.symbol.width());
            invalidated = std::cmp::max(affected_width, invalidated).saturating_sub(1);
        }
        updates
    }
    pub fn to_vec<'a>(&'a self) -> Vec<(u16, u16, &'a Cell)> {
        let mut result = Vec::new();
        let width = self.area.width;

        for (i, cell) in self.content.iter().enumerate() {
            let row = (i as u16) / width;
            let col = (i as u16) % width;
            result.push((col, row, cell));
        }

        result
    }
    /// Resize the buffer so that the mapped area matches the given area and that the buffer
    /// length is equal to area.width * area.height
    pub fn resize(&mut self, area: Rect) {
        let length = area.area() as usize;
        if self.content.len() > length {
            self.content.truncate(length);
        } else {
            self.content.resize(length, Default::default());
        }
        self.area = area;
    }

    pub fn draw(&self) {
        let diff = self.to_vec();

        let mut fg = Color::Reset;
        let mut bg = BgColor::Reset;
        let mut modifier = Modifier::empty();
        let mut lock = std::io::stdout().lock();

        for (x, y, cell) in diff {
            //Apparently 0, 0 and 1, 1 are the same?
            let x = x + 1;
            let y = y + 1;
            move_to(x, y);

            if cell.modifier != modifier {
                // let diff = ModifierDiff {
                //     from: modifier,
                //     to: cell.modifier,
                // };
                // diff.queue(&mut self.buffer)?;
                modifier = cell.modifier;
            }
            if cell.fg != fg {
                write!(lock, "{}", cell.fg.code()).unwrap();
                fg = cell.fg;
            }
            if cell.bg != bg {
                write!(lock, "{}", cell.bg.code()).unwrap();
                bg = cell.bg;
            }

            write!(lock, "{}", cell.symbol).unwrap();
        }

        lock.flush().unwrap();

        reset();
    }
    /// Reset all cells in the buffer
    pub fn reset(&mut self) {
        for c in &mut self.content {
            c.reset();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub symbol: String,
    pub fg: Color,
    pub bg: BgColor,
    pub modifier: Modifier,
}

impl Cell {
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Cell {
        self.symbol.clear();
        self.symbol.push_str(symbol);
        self
    }
    pub fn set_char(&mut self, ch: char) -> &mut Cell {
        self.symbol.clear();
        self.symbol.push(ch);
        self
    }
    pub fn set_style(&mut self, style: Style) -> &mut Cell {
        if let Some(c) = style.fg {
            self.fg = c;
        }
        if let Some(c) = style.bg {
            self.bg = c;
        }
        if !style.modifier.is_empty() {
            self.modifier = style.modifier;
        }
        self
    }
    pub fn set_fg(&mut self, color: Color) -> &mut Cell {
        self.fg = color;
        self
    }
    pub fn set_bg(&mut self, color: BgColor) -> &mut Cell {
        self.bg = color;
        self
    }
    pub fn set_modifier(&mut self, modifier: Modifier) -> &mut Cell {
        self.modifier = modifier;
        self
    }
    pub fn reset(&mut self) {
        self.symbol.clear();
        self.symbol.push(' ');
        self.fg = Color::Reset;
        self.bg = BgColor::Reset;
        self.modifier = Modifier::empty();
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            symbol: " ".into(),
            fg: Color::Reset,
            bg: BgColor::Reset,
            modifier: Modifier::empty(),
        }
    }
}
