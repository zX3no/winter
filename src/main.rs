use winter::*;

fn main() {
    let term = Terminal::new();

    let Info {
        buffer_size: (x, y),
        terminal_size: _,
    } = window_info(&term);

    // clear();

    // move_to(x, y - 4);

    // print!("x");
    // write_char(&term, b"xxxxxxxxxxxxx");

    // for x in 0..x {
    //     move_to(x, 0);
    //     print!("x");
    // }

    // for x in 0..x {
    //     move_to(x, y);
    //     print!("x");
    // }

    // for y in 0..y {
    //     move_to(0, y);
    //     print!("x");
    // }

    // for y in 0..y {
    //     move_to(x, y);
    //     print!("x");
    // }
}
