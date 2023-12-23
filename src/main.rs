#![allow(unused)]
use std::{
    borrow::Cow,
    io::{stdin, stdout, Write},
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

    let list = list(&items).block(block().title("Output Device").title_margin(1));
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
        let block = block().title(title.bold()).title_margin(1);
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

fn draw(buf: &mut Buffer) {
    let area = buf.area;

    {
        let area = area.centered(100, 22).unwrap();

        block().style(bg(Red)).draw(area, buf);

        // .inner((2, 2));
        let v = layout(area, Vertical, &[Percentage(50), Length(5)]);
        // dbg!(area.width, &v);
        // dbg!(&buf.area, area, &v);
        block().draw(v[0], buf);
        block().draw(v[1], buf);
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
        // let table = table(rows, &con)
        //     .header(header!["First".bold(), "Second".bold()])
        //     .block(block())
        //     .symbol("> ");

        //TODO: Maybe state should hold a row, style and index.
        //That way you can set exacly what you want when selected.

        // table.draw(chunks[1], buf, Some(1));
        // table.draw(area, buf, Some(0));
    }
}

fn main() {
    let mut winter = Winter::new();

    //Prevents panic messages from being hidden.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        uninit(&mut stdout(), &mut stdin());
        orig_hook(panic_info);
        std::process::exit(1);
    }));

    let mut index = 0;
    let mut cursor = (10, 5);

    loop {
        //Draw the widgets into the front buffer.
        draw(&mut winter.buffer());

        // show_cursor(&mut stdout);
        // show_blinking(&mut stdout);
        // browser(viewport, &mut buffers[current], None);
        // settings(viewport, &mut buffers[current]);

        //Handle events
        {
            if let Some((event, state)) = winter.poll() {
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
                if event == Event::Char('c') && state.control() || event == Event::Escape {
                    break;
                }
            }
        }

        winter.draw();
    }
}
