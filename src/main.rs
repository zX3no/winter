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
                    "hi".into(),
                    "test".into(),
                    "test".into(),
                    "test".into(),
                    "test".into(),
                    "test".into(),
                    "test".into(),
                    "test".into(),
                ];
                let lines = lines!(temp);

                let mut state = list_state(Some(5));

                let list = list_fn(
                    Some(block(None, Borders::ALL, BorderType::Rounded, fg(Red))),
                    lines,
                    style(),
                    Corner::TopLeft,
                    style(),
                    Some("> "),
                    |list| list.draw(chunks[0], buf, &mut state),
                );
            }
            {
                let block = block(None, Borders::ALL, BorderType::Rounded, style());
                //TODO: Only the first part of text shows up?
                //TODO: Styles seem quite broken.
                let temp = &["hi".into(), "test".into(), "?????".into()];
                let lines = lines!(temp);
                let row = row(
                    vec![lines.clone(), lines.clone(), lines.clone(), lines.clone()],
                    fg(Blue).bg(Blue),
                    0,
                );
                let temp = &[row.clone(), row.clone(), row.clone()];
                let mut state = table_state(Some(2));
                let con = [
                    Constraint::Length(2),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ];
                let table = table(
                    Some(block),
                    style(),
                    &con,
                    1,
                    style(),
                    Some("> "),
                    None,
                    temp,
                    false,
                );
                table.draw(chunks[1], buf, &mut state);
            }

            let title = text!("うずまき", fg(Blue).bg(White));
            let block = block(Some(title), Borders::ALL, BorderType::Rounded, fg(Red));
            let lines = lines!(temp, block);
            // lines.draw_wrapping(chunks[0], buf);

            // let guage = guage(None, 0.25, None, bg(Blue), style());
            // guage.draw(chunks[1], buf);
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
