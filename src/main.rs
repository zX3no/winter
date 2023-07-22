#![allow(unused)]
use std::io::Write;
use winter::layout::Rect;
use winter::{block::*, buffer::Cell, *};
use winter::{buffer::Buffer, layout::Margin};

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

fn draw_text(text: &str, area: Rect, buf: &mut Buffer) {
    let mut chars = text.chars();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(char) = chars.next() {
                buf.get_mut(x, y).set_char(char);
            } else {
                return;
            }
        }
    }
}

fn main() {
    let term = Terminal::new();
    let (width, height) = window_info(&term).terminal_size;
    let mut area = Rect::new(0, 0, width, height);
    let mut buffers: [Buffer; 2] = [Buffer::empty(area), Buffer::empty(area)];
    let mut current = 0;

    clear();

    loop {
        //Draw widgets
        {
            block::draw(
                Borders::ALL,
                BorderType::Rounded,
                Style::default(),
                area,
                &mut buffers[current],
            );

            draw_text(
                "testing",
                area.inner(&Margin {
                    vertical: 2,
                    horizontal: 2,
                }),
                &mut buffers[current],
            );
        }

        //Calculate difference and draw
        let previous_buffer = &buffers[1 - current];
        let current_buffer = &buffers[current];
        let diff = previous_buffer.diff(current_buffer);
        draw(diff);

        //Swap buffers
        buffers[1 - current].reset();
        current = 1 - current;

        //Resize
        let (width, height) = window_info(&term).terminal_size;
        area = Rect::new(0, 0, width, height);
        if buffers[current].area != area {
            buffers[current].resize(area);
            buffers[1 - current].resize(area);
            // Reset the back buffer to make sure the next update will redraw everything.
            //TODO: Clear isn't buffered.
            clear();
            buffers[1 - current].reset();
        }
    }
}

//TOOD: This might be a better way of doing things.
struct Buffers {
    front: Vec<Cell>,
    back: Vec<Cell>,
    area: Rect,
}
