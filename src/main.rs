#![allow(unused)]
use winter::{
    block::{BorderType, Borders},
    buffer::Cell,
    test_style::{Modifier, Style},
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
use std::io::Write;

fn draw(diff: Vec<(u16, u16, &Cell)>) {
    let mut fg = Color::Reset;
    let mut bg = BackgroundColor::Reset;
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
            write!(lock, "{}", cell.fg.code());
            fg = cell.fg;
        }
        if cell.bg != bg {
            write!(lock, "{}", cell.bg.code());
            bg = cell.bg;
        }

        write!(lock, "{}", cell.symbol).unwrap();
    }

    lock.flush().unwrap();

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
    // print!("{}", BackgroundColor::Red.code());
    draw(diff);
}
