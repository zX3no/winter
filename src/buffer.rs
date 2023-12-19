use crate::{layout::Rect, *};
use std::{cmp::min, io::Write};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub fn draw_modifier<W: Write>(w: &mut W, from: Modifier, to: Modifier) {
    let removed = from - to;
    if removed.contains(Modifier::INVERT) {
        write!(w, "{}", NO_INVERT).unwrap();
    }
    if removed.contains(Modifier::BOLD) {
        write!(w, "{}", NO_BOLD_OR_DIM).unwrap();
        if to.contains(Modifier::DIM) {
            write!(w, "{}", DIM).unwrap();
        }
    }
    if removed.contains(Modifier::ITALIC) {
        write!(w, "{}", NO_ITALIC).unwrap();
    }
    if removed.contains(Modifier::UNDERLINED) {
        write!(w, "{}", NO_UNDERLINE).unwrap();
    }
    if removed.contains(Modifier::DIM) {
        write!(w, "{}", NO_BOLD_OR_DIM).unwrap();
    }
    if removed.contains(Modifier::CROSSED_OUT) {
        write!(w, "{}", NO_STRIKETHROUGH).unwrap();
    }
    if removed.contains(Modifier::SLOW_BLINK) || removed.contains(Modifier::FAST_BLINK) {
        write!(w, "{}", NO_BLINKING).unwrap();
    }

    let added = to - from;
    if added.contains(Modifier::INVERT) {
        write!(w, "{}", INVERT).unwrap();
    }
    if added.contains(Modifier::BOLD) {
        write!(w, "{}", BOLD).unwrap();
    }
    if added.contains(Modifier::ITALIC) {
        write!(w, "{}", ITALIC).unwrap();
    }
    if added.contains(Modifier::UNDERLINED) {
        write!(w, "{}", UNDERLINE).unwrap();
    }
    if added.contains(Modifier::DIM) {
        write!(w, "{}", DIM).unwrap();
    }
    if added.contains(Modifier::CROSSED_OUT) {
        write!(w, "{}", STRIKETHROUGH).unwrap();
    }
    if added.contains(Modifier::SLOW_BLINK) {
        write!(w, "{}", SLOW_BLINKING).unwrap();
    }
    if added.contains(Modifier::FAST_BLINK) {
        write!(w, "{}", FAST_BLINKING).unwrap();
    }
}

///Note: Appends the cells to a buffer. Hides the cursor.
pub fn draw<W: Write>(w: &mut W, diff: Vec<(u16, u16, &Cell)>) {
    let mut fg = Color::Reset;
    let mut bg = Color::Reset;
    //TODO: Maybe have a variable for all modifiers?
    //That way we can turn each indiviually on and off.
    let mut modifier = Modifier::empty();
    let mut last_pos: Option<(u16, u16)> = None;

    //Move to start.
    hide_cursor(w);
    move_to(w, 1, 1);

    for (x, y, cell) in diff {
        //Apparently 0, 0 and 1, 1 are the same?
        let x = x + 1;
        let y = y + 1;
        if !matches!(last_pos, Some(p) if x == p.0 + 1 && y == p.1) {
            move_to(w, x, y);
        }
        last_pos = Some((x, y));

        if cell.modifier != modifier {
            draw_modifier(w, modifier, cell.modifier);
            modifier = cell.modifier;
        }
        if cell.fg != fg {
            write!(w, "{}", cell.fg.fg_code()).unwrap();
            fg = cell.fg;
        }
        if cell.bg != bg {
            write!(w, "{}", cell.bg.bg_code()).unwrap();
            bg = cell.bg;
        }

        write!(w, "{}", cell.symbol).unwrap();
    }

    //Always write reset as the last symbol.
    //That way styles never stay when the program is closed.
    write!(w, "{}", RESET).unwrap();
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
    pub fn get_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let i = self.index_of(x, y).unwrap();
        &mut self.content[i]
    }
    pub fn set_lines(
        &mut self,
        mut x: u16,
        y: u16,
        lines: &Lines<'_>,
        width: u16,
        scroll: bool,
    ) -> (u16, u16) {
        let mut remaining_width = width;

        let text_width = lines.iter().map(|text| text.width()).sum::<usize>();
        let mut overflow = text_width.saturating_sub(width as usize);

        for line in lines.iter() {
            if remaining_width == 0 {
                break;
            }

            let style = lines.style.unwrap_or(line.style);
            let line = if scroll {
                // If there is overflow, skip characters from the start of the line
                let mut skip = 0;
                if overflow > 0 {
                    let graphemes =
                        UnicodeSegmentation::grapheme_indices(&line.inner as &str, true)
                            .collect::<Vec<(usize, &str)>>();
                    for (_, grapheme) in graphemes.iter() {
                        if skip + grapheme.width() <= overflow {
                            skip += grapheme.width();
                        } else {
                            break;
                        }
                    }
                    overflow -= skip;
                }
                &line.inner[skip..]
            } else {
                &line.inner
            };

            let pos = self.set_stringn(x, y, line, remaining_width as usize, style);
            let w = pos.0.saturating_sub(x);
            x = pos.0;
            remaining_width = remaining_width.saturating_sub(w);
        }

        (x, y)
    }
    /// Print at most the first n characters of a string if enough space is available
    /// until the end of the line
    #[track_caller]
    pub fn set_stringn<S>(
        &mut self,
        x: u16,
        y: u16,
        string: S,
        width: usize,
        style: Style,
    ) -> (u16, u16)
    where
        S: AsRef<str>,
    {
        let mut index = self.index_of(x, y).unwrap();
        let mut x_offset = x as usize;
        let graphemes = UnicodeSegmentation::graphemes(string.as_ref(), true);
        let max_offset = min(self.area.right() as usize, width.saturating_add(x as usize));
        for s in graphemes {
            //FIXME: This dumbass character is broken for some reason.
            let s = if s == "a\u{304}" { "ā" } else { s };
            let width = s.width();
            if width == 0 {
                continue;
            }
            // `x_offset + width > max_offset` could be integer overflow on 32-bit machines if we
            // change dimenstions to usize or u32 and someone resizes the terminal to 1x2^32.
            if width > max_offset.saturating_sub(x_offset) {
                break;
            }

            self.content[index].set_symbol(s);
            self.content[index].set_style(style);
            // Reset following cells if multi-width (they would be hidden by the grapheme),
            for i in index + 1..index + width {
                self.content[i].reset();
            }
            index += width;
            x_offset += width;
        }
        (x_offset as u16, y)
    }
    pub fn set_style(&mut self, area: Rect, style: Style) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                self.get_mut(x, y).set_style(style);
            }
        }
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
    pub fn clear(&mut self, area: Rect) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                self.get_mut(x, y).reset();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
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
        self.fg = style.fg;
        self.bg = style.bg;
        if !style.modifier.is_empty() {
            self.modifier = style.modifier;
        }
        self
    }
    pub fn set_fg(&mut self, color: Color) -> &mut Cell {
        self.fg = color;
        self
    }
    pub fn set_bg(&mut self, color: Color) -> &mut Cell {
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
        self.bg = Color::Reset;
        self.modifier = Modifier::empty();
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            symbol: " ".into(),
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        }
    }
}
