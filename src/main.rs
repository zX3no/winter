use winter::*;

fn main() {
    let term = Terminal::new();

    let Info {
        buffer_size: (x, y),
        terminal_size: _,
    } = window_info(&term);

    // println!("{}", "test".blue());
    // reset();

    clear();
    move_to(20, 2);
    print!("x");
    move_to(25, 4);
    print!("x");

    // clear_line();
    clear_line_from_cursor_to_start();
}
