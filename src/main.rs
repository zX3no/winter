#![allow(unused)]
use std::{
    borrow::Cow,
    io::{stdout, Write},
    time::Instant,
};
use winter::*;

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
        stdout.flush().unwrap();

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
                Direction::Horizontal,
                (0, 0),
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                ],
                viewport,
            );

            {
                let lines = lines_s!("hi", bold().italic());
                // lines.draw(viewport, buf);
            }

            {
                let guage = guage(None, 0.75, None, bold(), bg(Blue), bg(Red));
                // guage.draw(chunks[0], buf);
                // guage.draw(viewport, buf);
            }

            {
                let block = block(None, Borders::ALL, BorderType::Rounded, style());
                let con = [Constraint::Percentage(50), Constraint::Percentage(50)];
                let text = String::from("first item first row");
                let rows = &[
                    //Row 1
                    row![&[
                        //Row 1 Column 1
                        lines_s!(
                            text,
                            // "first item first row",
                            fg(Cyan),
                            " <-- there is a space here",
                            fg(Blue).underlined()
                        ),
                        //Row 1 Column 2
                        lines!("second item", " first row")
                    ]],
                    //Row 2
                    row![&[
                        //Row 2 Column 1
                        lines_s!("first item second row", fg(Yellow)),
                        //Row 2 Column 2
                        lines!("second item second row")
                    ]],
                ];
                let lines = &[lines_s!("First", bold()), lines_s!("Second", bold())];
                let header = Some(header![lines]);

                let table = table(header, Some(block), &con, rows, Some("> "), fg(Blue));

                //TODO: Maybe state should hold a row, style and index.
                //That way you can set exacly what you want when selected.
                let mut state = table_state(Some(0));

                // table.draw(chunks[1], buf, &mut state);
                table.draw(viewport, buf, &mut state);
            }

            {
                let lines = lines!["hi", "test", "test", "test", "test", "test", "test", "test"];
                let mut state = list_state(Some(5));
                let list = list(
                    Some(block(None, Borders::ALL, BorderType::Rounded, fg(Red))),
                    lines,
                    Some("> "),
                    fg(Blue).bg(Red),
                );
                // list.draw(chunks[2], buf, &mut state);
                // list.draw(viewport, buf, &mut state);
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

        return;
        // std::thread::park();
        // std::thread::sleep(std::time::Duration::from_millis(16));
    }

    unreachable!();
}
