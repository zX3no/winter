#![allow(unused)]
use std::{borrow::Cow, io::Write};
use winter::layout::Rect;
use winter::{block::*, *};
use winter::{
    buffer::{Buffer, Cell},
    layout::Margin,
};

//List of widgets gonk uses:
//Text with different styles
//Guage
//List
//Table
//Block
//Paragraph

fn main() {
    let mut term = Terminal::new();
    let (width, height) = term.info().terminal_size;
    let mut viewport = Rect::new(0, 0, width, height);
    let mut buffers: [Buffer; 2] = [Buffer::empty(viewport), Buffer::empty(viewport)];
    let mut current = 0;

    //Prevents panic messages from being hidden.
    // let orig_hook = std::panic::take_hook();
    // std::panic::set_hook(Box::new(move |panic_info| {
    //     // disable_raw_mode();
    //     // disable_mouse_caputure();
    //     leave_alternate_screen();
    //     orig_hook(panic_info);
    //     std::process::exit(1);
    // }));

    //TODO: Enable raw mode.
    // enter_alternate_screen();
    // clear();

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

            let str = "line 3asdlkasjdalskdjaslkd ajsdlk asjdasldkjasdl kajdaslkdjasld kasjd lkasjd aslkd jaslkdasjd laskdj alskd jasldkajs dlkasjd laskdj aslkd jaslk djasd asjlasldkasjd laksdj alskdjasldkasdlasjkdasjdlaskdjlaskdjalksddlkasdjaslkd jsalkd jalkdasjdlaskdj asldk jasdl kasjd laksjd aslkdajsdslkdjaslkdja final-word";

            //TODO: Draw text inside of block.
            let temp = &[
                text!(str),
                text!(str, style().fg(Color::Blue)),
                text!("う ず ま き "),
            ];
            let l = lines!(temp, style().fg(Color::Red));

            let mut v = viewport.clone();
            v.x += 2;

            l.draw_wrapping(viewport, &mut buffers[current]);

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

        clear();
        buffer::draw(diff);

        return;

        //Swap buffers
        buffers[1 - current].reset();
        current = 1 - current;

        //Resize
        let (width, height) = term.info().terminal_size;
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

    leave_alternate_screen();
}

//TOOD: This might be a better way of doing things.
struct Buffers {
    front: Vec<Cell>,
    back: Vec<Cell>,
    area: Rect,
}
