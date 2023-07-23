use crate::{layout::Rect, move_to, reset, BgColor, Color, Modifier, Style};
use std::io::Write;
use unicode_width::UnicodeWidthStr;

//Move out of function and into main loop.
//That way variables are not reinitialized.
pub fn draw(diff: Vec<(u16, u16, &Cell)>) {
    let mut fg = Color::Reset;
    let mut bg = BgColor::Reset;
    //TODO: Maybe have a variable for all modifiers?
    //That way we can turn each indiviually on and off.
    let mut modifier = Modifier::empty();
    let mut last_pos: Option<(u16, u16)> = None;
    let mut lock = std::io::stdout().lock();

    //Move to start.
    move_to(1, 1);
    for (x, y, cell) in diff {
        //Apparently 0, 0 and 1, 1 are the same?
        let x = x + 1;
        let y = y + 1;
        if !matches!(last_pos, Some(p) if x == p.0 + 1 && y == p.1) {
            move_to(x, y);
        }
        last_pos = Some((x, y));

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
    pub fn get_mut(&mut self, x: u16, y: u16) -> Result<&mut Cell, String> {
        let i = self.index_of(x, y)?;
        Ok(&mut self.content[i])
    }
    pub fn index_of(&self, x: u16, y: u16) -> Result<usize, String> {
        if !(x >= self.area.left()
            && x < self.area.right()
            && y >= self.area.top()
            && y < self.area.bottom())
        {
            Err(format!(
                "Trying to access position outside the buffer: x={}, y={}, area={:?}",
                x, y, self.area
            ))
        } else {
            Ok(((y - self.area.y) * self.area.width + (x - self.area.x)) as usize)
        }
    }
    ///Does not allow for multi-width characters.
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
    ///Allows for multi-width characters.
    pub fn diff_wide<'a>(&self, other: &'a Buffer) -> Vec<(u16, u16, &'a Cell)> {
        let previous_buffer = &self.content;
        let next_buffer = &other.content;
        let width = self.area.width;

        let mut updates: Vec<(u16, u16, &Cell)> = vec![];
        let mut skip_count: usize = 0;

        for (i, (current, previous)) in next_buffer.iter().zip(previous_buffer.iter()).enumerate() {
            let x = i as u16 % width;
            let y = i as u16 / width;

            if skip_count == 0 {
                if current != previous {
                    updates.push((x, y, current));
                }

                let mut affected_width = current.symbol.width();
                if affected_width > 1 {
                    // Check if this multi-width character spans into the next cells.
                    for j in 1..affected_width {
                        let next_index = i + j;
                        if next_index >= next_buffer.len() {
                            break;
                        }

                        let next_cell = &next_buffer[next_index];
                        if next_cell.symbol.width() == 0 {
                            break;
                        }

                        // If the next cell is part of the same multi-width character, mark it as skipped.
                        updates.push((x + j as u16, y, next_cell));
                        skip_count += 1;
                        affected_width = std::cmp::max(affected_width, next_cell.symbol.width());
                    }
                }

                // The number of cells to skip due to multi-width character.
                skip_count = affected_width.saturating_sub(1);
            } else {
                // Skip the cell since it's part of a multi-width character.
                skip_count -= 1;
            }
        }

        updates
    }
    pub fn to_vec<'a>(&'a self) -> Vec<(u16, u16, &'a Cell)> {
        self.content
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let row = (i as u16) / self.area.width;
                let col = (i as u16) % self.area.width;
                (col, row, cell)
            })
            .collect()
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
        draw(diff);
    }
    pub fn is_empty(&self) -> bool {
        for c in &self.content {
            if c.symbol != " " {
                return false;
            }
        }
        true
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
