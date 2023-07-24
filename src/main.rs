#![allow(unused)]
use std::{
    borrow::Cow,
    io::{stdout, Write},
    time::Instant,
};
use winter::buffer::{Buffer, Cell};
use winter::layout::Rect;
use winter::{block::*, *};

fn main() {
    let mut term = Terminal::new();
    let (width, height) = term.area();
    let mut viewport = Rect::new(0, 0, width, height);
    let mut buffers: [Buffer; 2] = [Buffer::empty(viewport), Buffer::empty(viewport)];
    let mut current = 0;

    //Prevents panic messages from being hidden.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let mut stdout = stdout();
        // disable_raw_mode();
        // disable_mouse_caputure();
        leave_alternate_screen(&mut stdout);
        show_cursor(&mut stdout);
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    //TODO: Enable raw mode.
    let mut stdout = stdout();
    hide_cursor(&mut stdout);
    enter_alternate_screen(&mut stdout);
    clear(&mut stdout);

    loop {
        //Draw widgets
        {
            let buf = &mut buffers[current];

            let str = "line 3asdlkasjdalskdjaslkd ajsdlk asjdasldkjasdl kajdaslkdjasld kasjd lkasjd aslkd jaslkdasjd laskdj alskd jasldkajs dlkasjd laskdj aslkd jaslk djasd asjlasldkasjd laksdj alskdjasldkasdlasjkdasjdlaskdjlaskdjalksddlkasdjaslkd jsalkd jalkdasjdlaskdj asldk jasdl kasjd laksjd aslkdajsdslkdjaslkdja final-word";
            let temp = &[
                text!(str),
                // text!(""),
                text!(str),
                // text!(""),
                text!("う ず ま き"),
            ];

            let chunks = layout_new(
                Direction::Vertical,
                (0, 0),
                [Constraint::Percentage(50), Constraint::Percentage(50)],
                viewport,
            );

            {
                let temp = &[
                    text!("hi"),
                    text!("test"),
                    text!("test2"),
                    text!("test"),
                    text!("test"),
                    text!("test"),
                    text!("test"),
                    text!("test"),
                    text!("test"),
                ];
                let lines = lines!(temp);
                let slice = &[lines];
                let list = list(None, slice, style(), Corner::TopLeft, style(), Some(">"));
                //TODO: Doesn't select correct item?
                let mut state = list_state(Some(1));
                //TODO: Doesn't draw at all?
                list.draw(chunks[0], buf, &mut state)
            }

            let title = text!("うずまき", fg(Blue).bg(White));
            let block = block(Some(title), Borders::ALL, BorderType::Rounded, fg(Red));
            let lines = lines!(temp, block);
            lines.draw_wrapping(chunks[0], buf);

            let guage = guage(None, 0.25, None, bg(Blue), style());
            guage.draw(chunks[1], buf);
        }

        //Calculate difference and draw
        let previous_buffer = &buffers[1 - current];
        let current_buffer = &buffers[current];
        let diff = previous_buffer.diff(current_buffer);
        buffer::draw(&mut stdout, diff);

        //Swap buffers
        buffers[1 - current].reset();
        current = 1 - current;

        //Resize
        let (width, height) = term.area();
        viewport = Rect::new(0, 0, width, height);
        if buffers[current].area != viewport {
            buffers[current].resize(viewport);
            buffers[1 - current].resize(viewport);
            // Reset the back buffer to make sure the next update will redraw everything.
            //TODO: Clear isn't buffered.
            clear(&mut stdout);
            buffers[1 - current].reset();
        }
    }

    unreachable!();
}

//TOOD: This might be a better way of doing things.
struct Buffers {
    front: Vec<Cell>,
    back: Vec<Cell>,
    area: Rect,
}
