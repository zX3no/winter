#![allow(soft_unstable)]
use bitflags::bitflags;
use std::{ffi::OsStr, mem::zeroed, os::windows::prelude::OsStrExt, process::Command};
use winapi::{
    ctypes::c_void,
    um::{
        consoleapi::{SetConsoleMode, WriteConsoleW},
        processenv::GetStdHandle,
        wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO},
    },
};

pub use style::Stylize;

pub mod block;
pub mod buffer;
pub mod layout;
pub mod spans;

pub mod color;
pub mod rect;
pub mod style;
pub mod symbols;

pub const STD_HANDLE: u32 = -11i32 as u32;

/// Example of changing color
/// ```rs
/// let color = Color::Red;
/// print!("{}", color.code());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    Reset,
}

impl Color {
    pub fn code(self) -> &'static str {
        match self {
            Color::Black => "\x1B[30m",
            Color::Red => "\x1B[31m",
            Color::Green => "\x1B[32m",
            Color::Yellow => "\x1B[33m",
            Color::Blue => "\x1B[34m",
            Color::Magenta => "\x1B[35m",
            Color::Cyan => "\x1B[36m",
            Color::White => "\x1B[37m",

            Color::BrightBlack => "\x1B[90m",
            Color::BrightRed => "\x1B[91m",
            Color::BrightGreen => "\x1B[92m",
            Color::BrightYellow => "\x1B[93m",
            Color::BrightBlue => "\x1B[94m",
            Color::BrightMagenta => "\x1B[95m",
            Color::BrightCyan => "\x1B[96m",
            Color::BrightWhite => "\x1B[97m",

            Color::Reset => "\x1B[0m",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum BackgroundColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    Reset,
}

impl BackgroundColor {
    pub fn code(self) -> &'static str {
        match self {
            BackgroundColor::Black => "\x1B[40m",
            BackgroundColor::Red => "\x1B[41m",
            BackgroundColor::Green => "\x1B[42m",
            BackgroundColor::Yellow => "\x1B[43m",
            BackgroundColor::Blue => "\x1B[44m",
            BackgroundColor::Magenta => "\x1B[45m",
            BackgroundColor::Cyan => "\x1B[46m",
            BackgroundColor::White => "\x1B[47m",

            BackgroundColor::BrightBlack => "\x1B[100m",
            BackgroundColor::BrightRed => "\x1B[101m",
            BackgroundColor::BrightGreen => "\x1B[102m",
            BackgroundColor::BrightYellow => "\x1B[103m",
            BackgroundColor::BrightBlue => "\x1B[104m",
            BackgroundColor::BrightMagenta => "\x1B[105m",
            BackgroundColor::BrightCyan => "\x1B[106m",
            BackgroundColor::BrightWhite => "\x1B[107m",

            BackgroundColor::Reset => "\x1B[49m",
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq, Copy)]
    pub struct Modifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINED        = 0b0000_0000_1000;
        const SLOW_BLINK        = 0b0000_0001_0000;
        const RAPID_BLINK       = 0b0000_0010_0000;
        const REVERSED          = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const CROSSED_OUT       = 0b0001_0000_0000;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<BackgroundColor>,
    pub modifier: Modifier,
}

impl Default for Style {
    fn default() -> Style {
        Self {
            fg: None,
            bg: None,
            modifier: Modifier::empty(),
        }
    }
}

pub enum ConsoleMode {
    EnableVirtualInputProcessing = 0x0004,
}

pub struct Terminal {
    pub handle: *mut c_void,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            handle: unsafe { GetStdHandle(STD_HANDLE) },
        }
    }
}

pub struct Info {
    pub buffer_size: (u16, u16),
    pub terminal_size: (u16, u16),
}

pub fn window_info(term: &Terminal) -> Info {
    unsafe {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let result = GetConsoleScreenBufferInfo(term.handle, &mut info);
        if result != 1 {
            panic!("Could not get window size.");
        } else {
            Info {
                buffer_size: (info.dwSize.X as u16, info.dwSize.Y as u16),
                terminal_size: (
                    (info.srWindow.Right - info.srWindow.Left) as u16,
                    (info.srWindow.Bottom - info.srWindow.Top) as u16,
                ),
            }
        }
    }
}

pub fn write(term: &Terminal, buf: &[u8]) {
    let utf16: Vec<u16> = OsStr::new(std::str::from_utf8(buf).unwrap())
        .encode_wide()
        .collect();

    let mut cells_written: u32 = 0;

    let result = unsafe {
        WriteConsoleW(
            term.handle,
            utf16.as_ptr() as *const c_void,
            utf16.len() as u32,
            &mut cells_written,
            zeroed(),
        )
    };

    if result != 1 {
        panic!("Could not write to console.");
    }
}

pub fn set_mode(term: &Terminal, mode: ConsoleMode) {
    unsafe {
        SetConsoleMode(term.handle, mode as u32);
    }
}

//https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797

//TODO: Use a buffer instead of print!
pub fn move_to(x: u16, y: u16) {
    print!("\x1b[{};{}H", y, x);
}

pub fn clear_all() {
    Command::new("cmd").args(["/C", "cls"]).status().unwrap();
}

pub fn shift_up(amount: u16) {
    print!("\x1b[{}S", amount);
}

pub fn shift_down(amount: u16) {
    print!("\x1b[{}T", amount);
}

///Reset all modes (styles and colors)
pub fn reset() {
    print!("\x1b[0m");
}

pub use clear::*;

mod clear {
    ///Same as \x1b[0J
    pub fn clear_from_cursor() {
        print!("\x1b[J");
    }

    pub fn clear_from_cursor_to_start() {
        print!("\x1b[1J");
    }

    pub fn clear() {
        print!("\x1b[2J");
    }

    ///Same as \x1b[0K
    pub fn clear_line_from_cursor() {
        print!("\x1b[K");
    }

    pub fn clear_line_from_cursor_to_start() {
        print!("\x1b[1K");
    }

    pub fn clear_line() {
        print!("\x1b[2K");
    }
}
