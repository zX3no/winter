#![allow(unused)]
use winter::{
    block::{BorderType, Borders},
    buffer::Cell,
    test_style::{Color, Modifier, Style},
    *,
};

use winter::buffer::Buffer;
use winter::layout::Rect;

fn render_block(area: Rect, buf: &mut Buffer) {
    let borders = Borders::ALL;
    let border_type = BorderType::Plain;
    let border_style = Style::default();

    if area.area() == 0 {
        return;
    }
    // buf.set_style(area, self.style);
    let symbols = BorderType::line_symbols(border_type);

    // Sides
    if borders.intersects(Borders::LEFT) {
        for y in area.top()..area.bottom() {
            buf.get_mut(area.left(), y)
                .set_symbol(symbols.vertical)
                .set_style(border_style);
        }
    }
    if borders.intersects(Borders::TOP) {
        for x in area.left()..area.right() {
            buf.get_mut(x, area.top())
                .set_symbol(symbols.horizontal)
                .set_style(border_style);
        }
    }
    if borders.intersects(Borders::RIGHT) {
        let x = area.right() - 1;
        for y in area.top()..area.bottom() {
            buf.get_mut(x, y)
                .set_symbol(symbols.vertical)
                .set_style(border_style);
        }
    }
    if borders.intersects(Borders::BOTTOM) {
        let y = area.bottom() - 1;
        for x in area.left()..area.right() {
            buf.get_mut(x, y)
                .set_symbol(symbols.horizontal)
                .set_style(border_style);
        }
    }

    // Corners
    if borders.contains(Borders::RIGHT | Borders::BOTTOM) {
        buf.get_mut(area.right() - 1, area.bottom() - 1)
            .set_symbol(symbols.bottom_right)
            .set_style(border_style);
    }
    if borders.contains(Borders::RIGHT | Borders::TOP) {
        buf.get_mut(area.right() - 1, area.top())
            .set_symbol(symbols.top_right)
            .set_style(border_style);
    }
    if borders.contains(Borders::LEFT | Borders::BOTTOM) {
        buf.get_mut(area.left(), area.bottom() - 1)
            .set_symbol(symbols.bottom_left)
            .set_style(border_style);
    }
    if borders.contains(Borders::LEFT | Borders::TOP) {
        buf.get_mut(area.left(), area.top())
            .set_symbol(symbols.top_left)
            .set_style(border_style);
    }

    // Title
    // if let Some(title) = title {
    //     let left_border_dx = if self.borders.intersects(Borders::LEFT) {
    //         1
    //     } else {
    //         0
    //     };

    //     let right_border_dx = if self.borders.intersects(Borders::RIGHT) {
    //         1
    //     } else {
    //         0
    //     };

    //     let title_area_width = area
    //         .width
    //         .saturating_sub(left_border_dx)
    //         .saturating_sub(right_border_dx);

    //     let title_dx = match self.title_alignment {
    //         Alignment::Left => left_border_dx,
    //         Alignment::Center => area.width.saturating_sub(title.width() as u16) / 2,
    //         Alignment::Right => area
    //             .width
    //             .saturating_sub(title.width() as u16)
    //             .saturating_sub(right_border_dx),
    //     };

    //     let title_x = area.left() + title_dx;
    //     let title_y = area.top();

    //     buf.set_spans(title_x, title_y, &title, title_area_width);
    // }
}

/// Obtains a difference between the previous and the current buffer and passes it to the
/// current backend for drawing.
// pub fn flush(&mut self) -> io::Result<()> {
//     let previous_buffer = &self.buffers[1 - self.current];
//     let current_buffer = &self.buffers[self.current];
//     let updates = previous_buffer.diff(current_buffer);
//     self.backend.draw(updates.into_iter())
// }

fn draw(diff: Vec<(u16, u16, &Cell)>) {
    let mut fg = Color::Reset;
    let mut bg = Color::Reset;
    let mut modifier = Modifier::empty();
    let mut last_pos: Option<(u16, u16)> = None;

    for (x, y, cell) in diff {
        // Move the cursor if the previous location was not (x - 1, y)
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
            let color = Color::from(cell.fg);
            // map_error(queue!(self.buffer, SetForegroundColor(color)))?;
            fg = cell.fg;
        }
        if cell.bg != bg {
            let color = Color::from(cell.bg);
            // map_error(queue!(self.buffer, SetBackgroundColor(color)))?;
            bg = cell.bg;
        }

        print!("{}", cell.symbol);
    }

    reset();
}

fn main() {
    let term = Terminal::new();

    let Info {
        buffer_size,
        terminal_size: (width, height),
    } = window_info(&term);

    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);
    render_block(area, &mut buffer);

    let mut empty = Buffer::empty(area);

    let diff = empty.diff(&buffer);

    clear();
    draw(diff);

    // dbg!(buffer);

    // clear();
    // move_to(0, 0);
    // print!("xxxxxxxxxxxxxxxxxx");
    // move_to(0, 10);
    // print!("xxxxxxxxxxxxxxxxxx");
}
