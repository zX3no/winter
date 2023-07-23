#![allow(unused)]
use std::{borrow::Cow, io::Write};
use winter::layout::Rect;
use winter::{block::*, *};
use winter::{
    buffer::{Buffer, Cell},
    layout::Margin,
};

//Move out of function and into main loop.
//That way variables are reinitialized.
fn draw(diff: Vec<(u16, u16, &Cell)>) {
    let mut fg = Color::Reset;
    let mut bg = BgColor::Reset;
    //TODO: Maybe have a variable for all modifiers?
    //That way we can turn each indiviually on and off.
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

//TODO: There needs to be a way to calculate the width and height of text.
//There is `unicode_width::UnicodeWidthStr`
//Height should just be the number of lines.
//There should be a way to write text without wrapping. i.e cutting off the end.
fn draw_text(text: &str, style: Style, area: Rect, buf: &mut Buffer) {
    let mut chars = text.chars();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(char) = chars.next() {
                buf.get_mut(x, y).set_char(char).set_style(style);
            } else {
                return;
            }
        }
    }
}

fn draw_lines(lines: &[&str], style: Style, area: Rect, buf: &mut Buffer) {
    let mut lines_iter = lines.iter();
    for y in area.top()..area.bottom() {
        if let Some(line) = lines_iter.next() {
            let mut chars = line.chars();
            for x in area.left()..area.right() {
                if let Some(char) = chars.next() {
                    buf.get_mut(x, y).set_char(char).set_style(style);
                } else {
                    break;
                }
            }
        } else {
            break;
        }
    }
}

fn draw_lines_wrapping(lines: &[&str], style: Style, area: Rect, buf: &mut Buffer) {
    if lines.is_empty() {
        return;
    }

    let mut lines = lines.iter();
    let Some(mut line) = lines.next() else {
        return;
    };
    let mut chars = line.chars();

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(char) = chars.next() {
                buf.get_mut(x, y).set_char(char).set_style(style);
            } else {
                if let Some(l) = lines.next() {
                    line = l;
                    chars = line.chars();
                } else {
                    return;
                }
                break;
            }
        }
    }
}

//List of widgets gonk uses:
//Text with different styles
//Guage
//List
//Table
//Block
//Paragraph

fn main() {
    let term = Terminal::new();
    let (width, height) = window_info(&term).terminal_size;
    let mut viewport = Rect::new(0, 0, width, height);
    let mut buffers: [Buffer; 2] = [Buffer::empty(viewport), Buffer::empty(viewport)];
    let mut current = 0;

    loop {
        //Draw widgets
        {
            block::draw(
                Borders::ALL,
                BorderType::Rounded,
                Style::default(),
                viewport,
                &mut buffers[current],
            );

            // draw_lines(
            //     &["testing", "line 2", "line 3asdlkasjdalskdjaslkd ajsdlk asjdasldkjasdl kajdaslkdjasld kasjd lkasjd aslkd jaslkdasjd laskdj alskd jasldkajs dlkasjd laskdj aslkd jaslk djasd asjlasldkasjd laksdj alskdjasldkasdlasjkdasjdlaskdjlaskdjalksddlkasdjaslkd jsalkd jalkdasjdlaskdj asldk jasdl kasjd laksjd aslkdajsdslkdjaslkdja final-word", "test"],
            //     style(),
            //     viewport.inner(&Margin {
            //         vertical: 2,
            //         horizontal: 2,
            //     }),
            //     &mut buffers[current],
            // );

            let str = "line 3asdlkasjdalskdjaslkd ajsdlk asjdasldkjasdl kajdaslkdjasld kasjd lkasjd aslkd jaslkdasjd laskdj alskd jasldkajs dlkasjd laskdj aslkd jaslk djasd asjlasldkasjd laksdj alskdjasldkasdlasjkdasjdlaskdjlaskdjalksddlkasdjaslkd jsalkd jalkdasjdlaskdj asldk jasdl kasjd laksjd aslkdajsdslkdjaslkdja final-word";
            // let t = text(str, style().fg(Color::Blue));

            let temp = &[text!(str), text!(str, style().fg(Color::Blue))];
            let l = lines!(temp);
            l.draw_wrapping(viewport, &mut buffers[current]);
            // t.draw(viewport, &mut buffers[current]);

            //TODO
            //text("test", blue());
            //text("test", fg_blue());
            //Here style is enum {Color, Background, Modifier}
            //
            //OR
            //text("test", style().blue().bg_red())
        }

        //Calculate difference and draw
        let previous_buffer = &buffers[1 - current];
        let current_buffer = &buffers[current];
        let diff = previous_buffer.diff(current_buffer);

        //TODO: Move before loop.
        clear();

        draw(diff);

        //Skip looping for now.
        return;

        //Swap buffers
        buffers[1 - current].reset();
        current = 1 - current;

        //Resize
        let (width, height) = window_info(&term).terminal_size;
        viewport = Rect::new(0, 0, width, height);
        if buffers[current].area != viewport {
            buffers[current].resize(viewport);
            buffers[1 - current].resize(viewport);
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
