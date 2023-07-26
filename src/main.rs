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
    let mut terminal = Terminal::new();
    let (width, height) = terminal.area();
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
        'draw: {
            let buf = &mut buffers[current];

            let test = lines!("hi", "hi", String::from("test"));
            let test = lines_s!("hi", style(), "hi", style(), String::from("hi"), style());
            let str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed vitae nisi et mi sollicitudin vulputate. Vestibulum et hendrerit mauris. Nam euismod, nulla sit amet bibendum consequat, arcu sapien hendrerit odio, ut venenatis elit urna et risus. Vivamus laoreet volutpat urna, at interdum massa eleifend a. Fusce ut congue lectus. Aenean quis cursus arcu. Sed fermentum, enim vitae fermentum ultrices, orci risus blandit sem, nec egestas tortor odio id dui. Sed quis quam eu mauris hendrerit aliquam. Sed malesuada iaculis neque, id porttitor velit vulputate nec. Duis ac dapibus mi, nec gravida mauris. Ut id";
            let temp = lines![str, str, "う ず ま き"];

            // temp.draw(viewport, buf);
            // break 'draw;

            let chunks = layout(
                Direction::Vertical,
                (0, 0),
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ],
                viewport,
            );

            {
                let guage = guage(None, 0.25, "", bg(Blue), style());
                guage.draw(chunks[0], buf);
            }

            {
                let block = block(None, Borders::ALL, BorderType::Rounded, style());
                //TODO: Only the first part of text shows up?
                //TODO: Styles seem quite broken.
                let lines = lines!["hi", "test", "????"];
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
                let table = table(None, Some(block), &con, Some("> "), temp, style(), style());
                table.draw(chunks[1], buf, &mut state);
            }

            {
                let lines = lines!["hi", "test", "test", "test", "test", "test", "test", "test"];
                let mut state = list_state(Some(5));
                let list = list_fn(
                    Some(block(None, Borders::ALL, BorderType::Rounded, fg(Red))),
                    lines,
                    style(),
                    Corner::TopLeft,
                    style(),
                    Some("> "),
                    |list| list.draw(chunks[2], buf, &mut state),
                );
            }
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
        let (width, height) = terminal.area();
        viewport = Rect::new(0, 0, width, height);
        if buffers[current].area != viewport {
            buffers[current].resize(viewport);
            buffers[1 - current].resize(viewport);
            // Reset the back buffer to make sure the next update will redraw everything.
            //TODO: Clear isn't buffered.
            clear(&mut stdout);
            buffers[1 - current].reset();
        }

        std::thread::park();
        // std::thread::sleep(std::time::Duration::from_millis(16));
    }

    unreachable!();
}
