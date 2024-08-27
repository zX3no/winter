#![feature(macro_metavar_expr)]
#![allow(non_camel_case_types, non_snake_case)]
use std::{
    fmt::Display,
    io::{stdin, stdout, Stdin, Stdout, Write},
    mem::zeroed,
    process::Command,
    time::{Duration, Instant},
};

//Widgets
pub use block::*;
pub use block::{block, Block, BorderType::*, ALL, BOTTOM, LEFT, RIGHT, TOP};
pub use guage::*;
pub use list::*;
pub use table::*;
pub use text::*;

pub mod block;
pub mod guage;
pub mod list;
pub mod table;
pub mod text;
pub mod win32;

pub mod windows;

// #[cfg(target_os = "windows")]
pub use windows::*;

pub use buffer::{Buffer, Cell};
pub use layout::Alignment::*;
pub use style::{Color::*, *};
pub use win32::*;

pub use layout::Constraint::*;
pub use layout::Direction::*;
pub use layout::*;

pub mod buffer;
pub mod layout;
pub mod style;
pub mod symbols;

//Re-export unicode width.
pub use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub struct Winter {
    pub viewport: Rect,
    pub buffers: [Buffer; 2],
    pub current: usize,
    pub stdout: Stdout,
    pub stdin: Stdin,
}

impl Winter {
    //TODO: WinterOpitons or something?
    //Alternate screen, raw mode, mouse support.
    //I want to be able to use copy and paste sometimes.
    pub fn new() -> Self {
        let (mut stdout, stdin) = initialise();

        show_alternate_screen(&mut stdout);
        clear(&mut stdout);

        let (width, height) = window_size(&stdout);
        let viewport = Rect::new(0, 0, width, height);

        Self {
            viewport,
            buffers: [Buffer::empty(viewport), Buffer::empty(viewport)],
            current: 0,
            stdout,
            stdin,
        }
    }
    pub fn draw(&mut self) {
        //Calculate difference and draw to the terminal.
        let previous_buffer = &self.buffers[1 - self.current];
        let current_buffer = &self.buffers[self.current];
        let diff = previous_buffer.diff(current_buffer);
        buffer::draw(&mut self.stdout, diff);

        //Swap buffers
        self.buffers[1 - self.current].reset();
        self.current = 1 - self.current;

        //Update the viewport area.
        //TODO: I think there is a resize event that might be better.
        let (width, height) = window_size(&self.stdout);
        self.viewport = Rect::new(0, 0, width, height);

        //Resize
        if self.buffers[self.current].area != self.viewport {
            self.buffers[self.current].resize(self.viewport);
            self.buffers[1 - self.current].resize(self.viewport);

            //Reset the back buffer to make sure the next update will redraw everything.
            self.buffers[1 - self.current].reset();
            //Screen must be cleared here.
            clear(&mut self.stdout);
        }
    }

    pub fn poll(&self) -> Option<(Event, KeyState)> {
        self.poll_timeout(Duration::from_secs(0))
    }
    pub fn poll_timeout(&self, timeout: Duration) -> Option<(Event, KeyState)> {
        poll_timeout(&self.stdin, timeout)
    }
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stdout.flush()
    }
    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current]
    }
}

/// Used with panic handlers.
///
/// ```
/// let orig_hook = std::panic::take_hook();
///     std::panic::set_hook(Box::new(move |panic_info| {
///         let mut stdout = std::io::stdout();
///         let mut stdin = std::io::stdin();
///         winter::uninit(&mut stdout, &mut stdin);
///         orig_hook(panic_info);
///         std::process::exit(1);
/// }));
/// ```
pub fn uninit(stdout: &mut Stdout, stdin: &mut Stdin) {
    reset_stdin(stdin);
    hide_alternate_screen(stdout);
    show_cursor(stdout);
    reset(stdout);
    stdout.flush().unwrap();
}

impl Drop for Winter {
    fn drop(&mut self) {
        uninit(&mut self.stdout, &mut self.stdin);
    }
}

// TODO: double clicks? They might be nice to have...or not.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    //Mouse
    LeftMouse(u16, u16),
    RightMouse(u16, u16),
    MiddleMouse(u16, u16),
    ScrollUp,
    ScrollDown,

    //Key
    Char(char),
    Function(u8),
    Enter,
    Backspace,
    Escape,
    Control,
    Shift,
    Alt,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Unknown(u16),

    //Other
    Resize(u16, u16),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Char(c) => write!(f, "Event::Char('{}')", c),
            _ => write!(f, "Event::{:?}", self),
        }
    }
}

//https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences

///Clear the entire screen, using `cmd /c cls`.
pub fn clear_all() {
    Command::new("cmd").args(["/C", "cls"]).status().unwrap();
}
pub fn show_cursor<W: Write>(w: &mut W) {
    write!(w, "\x1b[?25h").unwrap();
}
///Must be called after entering an alternate screen.
pub fn hide_cursor<W: Write>(w: &mut W) {
    write!(w, "\x1b[?25l").unwrap();
}
pub fn move_to<W: Write>(w: &mut W, x: u16, y: u16) {
    write!(w, "\x1b[{};{}H", y, x).unwrap();
}
pub fn show_blinking<W: Write>(w: &mut W) {
    write!(w, "\x1b[?12h").unwrap();
}
pub fn hide_blinking<W: Write>(w: &mut W) {
    write!(w, "\x1b[?12l").unwrap();
}

pub fn show_alternate_screen<W: Write>(w: &mut W) {
    write!(w, "\x1b[?1049h").unwrap();
}
pub fn hide_alternate_screen<W: Write>(w: &mut W) {
    write!(w, "\x1b[?1049l").unwrap();
}

pub fn shift_up<W: Write>(w: &mut W, amount: u16) {
    write!(w, "\x1b[{}S", amount).unwrap();
}
pub fn shift_down<W: Write>(w: &mut W, amount: u16) {
    write!(w, "\x1b[{}T", amount).unwrap();
}
///Reset all modes (styles and colors)
pub fn reset<W: Write>(w: &mut W) {
    write!(w, "\x1b[0m").unwrap();
}
///Same as \x1b[0J
pub fn clear_from_cursor<W: Write>(w: &mut W) {
    write!(w, "\x1b[J").unwrap();
}
pub fn clear_from_cursor_to_start<W: Write>(w: &mut W) {
    write!(w, "\x1b[1J").unwrap();
}
pub fn clear<W: Write>(w: &mut W) {
    write!(w, "\x1b[2J").unwrap();
}
///Same as \x1b[0K
pub fn clear_line_from_cursor<W: Write>(w: &mut W) {
    write!(w, "\x1b[K").unwrap();
}
pub fn clear_line_from_cursor_to_start<W: Write>(w: &mut W) {
    write!(w, "\x1b[1K").unwrap();
}
pub fn clear_line<W: Write>(w: &mut W) {
    write!(w, "\x1b[2K").unwrap();
}
