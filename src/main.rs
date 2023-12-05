#![allow(unused)]
use std::{
    borrow::Cow,
    io::{stdout, Write},
    time::{Duration, Instant},
};
use winter::*;

pub fn settings(area: Rect, buf: &mut Buffer) {
    //TODO: I liked the old item menu bold selections.
    //It doesn't work on most terminals though :(
    let devices = ["OUT 1-2", "OUT 1-4", "MONITOR"];
    let current_device = "MONITOR";
    let index = Some(0);

    let mut items = Vec::new();
    for device in devices {
        let item = if device == current_device {
            lines!(">> ".dim(), device)
        } else {
            lines!("   ", device)
        };
        items.push(item);
    }

    if let Some(index) = index {
        //TODO: Style doesn't apply across the entire row. Just the text area.
        items[index].style = Some(fg(Black).bg(White));
    }

    let list = list(&items).block(block().title("Output Device").margin(1));
    list.draw(area, buf, index);
}

pub fn browser(area: Rect, buf: &mut Buffer, index: Option<usize>) {
    let size = area.width / 3;
    let rem = area.width % 3;

    let chunks = layout(
        area,
        Horizontal,
        &[Length(size), Length(size), Length(size + rem)],
    );

    let a: [Lines<'_>; 3] = ["Artist 1".into(), "Artist 2".into(), "Artist 3".into()];
    let b: [Lines<'_>; 3] = ["Album 1".into(), "Album 2".into(), "Album 3".into()];
    let c: [Lines<'_>; 3] = ["Song 1".into(), "Song 2".into(), "Song 3".into()];

    fn browser_list<'a>(title: &'static str, content: &[Lines<'a>], use_symbol: bool) -> List<'a> {
        let block = block().title(title.bold()).margin(1);
        let symbol = if use_symbol { ">" } else { " " };
        list(content).block(block).symbol(symbol)
    }

    let artists = browser_list("Aritst", &a, false);
    let albums = browser_list("Album", &b, false);
    let songs = browser_list("Song", &c, true);

    artists.draw(chunks[0], buf, Some(0));
    albums.draw(chunks[1], buf, Some(0));
    songs.draw(chunks[2], buf, index);
}

fn draw(area: Rect, buf: &mut Buffer) {
    let test = lines!("hi", "hi", String::from("test"));
    let test = lines!("hi", "hi", String::from("hi"));
    let str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed vitae nisi et mi sollicitudin vulputate. Vestibulum et hendrerit mauris. Nam euismod, nulla sit amet bibendum consequat, arcu sapien hendrerit odio, ut venenatis elit urna et risus. Vivamus laoreet volutpat urna, at interdum massa eleifend a. Fusce ut congue lectus. Aenean quis cursus arcu. Sed fermentum, enim vitae fermentum ultrices, orci risus blandit sem, nec egestas tortor odio id dui. Sed quis quam eu mauris hendrerit aliquam. Sed malesuada iaculis neque, id porttitor velit vulputate nec. Duis ac dapibus mi, nec gravida mauris. Ut id";

    let chunks = layout(
        area,
        Direction::Horizontal,
        &[
            Constraint::Percentage(15),
            Constraint::Percentage(45),
            Constraint::Percentage(30),
        ],
    );

    {
        let lines = lines!("hi".bold().italic());
        // lines.draw(viewport, buf);
    }

    {
        let chunks = layout(
            area,
            Direction::Vertical,
            &[
                Constraint::Length(10),
                Constraint::Percentage(10),
                Constraint::Percentage(45),
            ],
        );

        for chunk in chunks {
            // block().draw(*chunk, buf);
        }
    }

    {
        let guage = guage(None, 0.75, "Label".into(), bg(Blue), bg(Red));
        // guage.draw(chunks[0], buf);
        // guage.draw(viewport, buf);
    }

    {
        let con = [Constraint::Percentage(50), Constraint::Percentage(50)];
        let text = String::from("first item first row");

        let str = format_args!("hello {}", 1).to_string();
        let text = text!("hello {} {}", "test", 1);

        //TODO: Delete the Lines struct and swap to using newlines instead????

        let rows = [
            //Row 1
            row![
                //Row 1 Column 1
                lines!(
                    text.fg(Cyan),
                    " <-- there is a space here".fg(Blue).underlined()
                ),
                //Row 1 Column 2
                text!("second item{}", " first row")
            ],
            //Row 2
            row![
                //Row 2 Column 1
                "Namā".fg(Yellow),
                //Row 2 Column 2
                "second item second row"
            ],
            row!["Row 3"],
            row!["Row 4"],
            row!["Row 5"],
            row!["Row 6"],
            row!["Row 7"],
            row!["Row 8"],
        ];

        // let table = table(header, Some(block()), &con, rows, Some("> "), fg(Blue));
        let table = table(rows, &con)
            .header(header!["First".bold(), "Second".bold()])
            .block(block())
            .symbol("> ");

        //TODO: Maybe state should hold a row, style and index.
        //That way you can set exacly what you want when selected.

        // table.draw(chunks[1], buf, Some(1));
        // table.draw(area, buf, Some(0));
    }

    {
        let list = list(&[
            lines!["hi".fg(Red), " there".fg(Blue)],
            lines!["these are", " some more ", "lines"],
        ])
        .block(block())
        .symbol("> ")
        .selection_style(fg(Blue).bg(Red).into());
        // list.draw(area, buf, Some(1));
    }

    {
        let fill = area.height - 3 - 3;
        // dbg!(area.height);
        let area = layout(
            area,
            Direction::Vertical,
            &[
                Constraint::Length(3),
                Constraint::Length(fill),
                Constraint::Length(3),
            ],
        );

        let table = table(
            [row!["Row 1"], row!["Row 2"]],
            &[Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .header(header!["First".bold(), "Second".bold()])
        .block(block())
        .symbol("> ");

        // dbg!(area);
        block().draw(area[0], buf);
        table.draw(area[1], buf, None);
        block().draw(area[2], buf);
    }

    {
        // let l = lines!("This is a test of some text");
        // l.draw(area, buf);

        let top = lines![
            "─│ ",
            "",
            "TEST ARTIST".fg(Blue),
            " ─ ",
            "Test Album".fg(Green),
            "",
            " │─"
        ];
        top.align(Center).draw(area, buf);
    }

    //Empty
    {
        let l = lines!();
        let t = text!();
    }
}

fn main() {
    let (output_handle, input_handle) = handles();
    let (width, height) = info(output_handle).window_size;

    let mut viewport = Rect::new(0, 0, width, height);
    let mut buffers: [Buffer; 2] = [Buffer::empty(viewport), Buffer::empty(viewport)];
    let mut current = 0;

    //Prevents panic messages from being hidden.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let mut stdout = stdout();
        uninit(&mut stdout);
        stdout.flush().unwrap();
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    //TODO: Might need to wrap stdout, viewport and current buffer.
    //v.area(), v.stdout(), v.buffer(). maybe maybe not.
    let mut stdout = stdout();
    init(&mut stdout);

    let mut index = 0;

    loop {
        //Draw the widgets into the front buffer.
        draw(viewport, &mut buffers[current]);
        // browser(viewport, &mut buffers[current], None);
        // settings(viewport, &mut buffers[current]);

        //Handle events
        {
            if let Some((event, state)) = poll(Duration::from_millis(16)) {
                // println!("{}", event);

                //TODO: I might want a class or trait or something to handle this pattern.
                if event == Event::Up {
                    if index != 0 {
                        index -= 1;
                    }
                }
                if event == Event::Down {
                    index += 1;
                }
                if event == Event::Char('c') && state.ctrl() || event == Event::Escape {
                    break;
                }
            }
        }

        //Calculate difference and draw to the terminal.
        let previous_buffer = &buffers[1 - current];
        let current_buffer = &buffers[current];
        let diff = previous_buffer.diff(current_buffer);
        buffer::draw(&mut stdout, diff);

        //Swap buffers
        buffers[1 - current].reset();
        current = 1 - current;

        //Update the viewport area.
        //TODO: I think there is a resize event that might be better.
        let (width, height) = info(output_handle).window_size;
        viewport = Rect::new(0, 0, width, height);

        //Resize
        if buffers[current].area != viewport {
            buffers[current].resize(viewport);
            buffers[1 - current].resize(viewport);

            // Reset the back buffer to make sure the next update will redraw everything.
            buffers[1 - current].reset();
            clear(&mut stdout);
        }

        // break;
    }

    uninit(&mut stdout);
}
