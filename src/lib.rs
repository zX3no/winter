#![allow(soft_unstable)]
use std::{
    io::{stdout, Stdout, Write},
    mem::zeroed,
    process::Command,
};
use winapi::{
    ctypes::c_void,
    um::{
        consoleapi::SetConsoleMode,
        processenv::GetStdHandle,
        wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO},
    },
};

pub use stylize::Stylize;

#[allow(unused)]
pub use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

//Widgets
pub use text::*;
pub mod block;
pub mod text;

pub use style::Color::*;
pub use style::*;

pub mod buffer;
pub mod color;
pub mod layout;
pub mod style;
pub mod stylize;
pub mod symbols;

const STD_HANDLE: u32 = -11i32 as u32;
const ENABLE_VIRTUAL_INPUT_PROCESSING: u32 = 0x0004;
const ENABLE_MOUSE_INPUT: u32 = 0x0010;

pub struct Info {
    pub buffer_size: (u16, u16),
    pub window_size: (u16, u16),
    pub cursor_position: (u16, u16),
}

pub struct Terminal {
    pub handle: *mut c_void,
    pub stdout: Stdout,
    pub mode: u32,
}

impl Terminal {
    pub fn new() -> Self {
        Self {
            handle: unsafe { GetStdHandle(STD_HANDLE) },
            stdout: stdout(),
            mode: 0,
        }
    }

    ///Get the terminal area that is usable.
    pub fn area(&self) -> (u16, u16) {
        unsafe {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
            let result = GetConsoleScreenBufferInfo(self.handle, &mut info);
            if result != 1 {
                panic!("Could not get window size.");
            }
            (
                (info.srWindow.Right - info.srWindow.Left) as u16,
                (info.srWindow.Bottom - info.srWindow.Top) as u16,
            )
        }
    }

    //TODO: Cleanup into term.size() or something this API sucks.
    /// Get the screen buffer information like terminal size, cursor position, buffer size.
    ///
    /// This wraps
    /// [`GetConsoleScreenBufferInfo`](https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfo).
    pub fn info(&self) -> Info {
        unsafe {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
            let result = GetConsoleScreenBufferInfo(self.handle, &mut info);
            if result != 1 {
                panic!("Could not get window size.");
            }
            Info {
                buffer_size: (info.dwSize.X as u16, info.dwSize.Y as u16),
                window_size: (
                    (info.srWindow.Right - info.srWindow.Left) as u16,
                    (info.srWindow.Bottom - info.srWindow.Top) as u16,
                ),
                cursor_position: (
                    info.dwCursorPosition.X as u16,
                    info.dwCursorPosition.Y as u16,
                ),
            }
        }
    }

    //TODO: Not sure if these are working correctly.
    pub fn enable_raw_mode(&mut self) {
        self.mode = self.mode | ENABLE_VIRTUAL_INPUT_PROCESSING;
        self.set_mode(self.mode);
    }
    pub fn disable_raw_mode(&mut self) {
        self.mode = self.mode & ENABLE_VIRTUAL_INPUT_PROCESSING;
        self.set_mode(self.mode);
    }
    pub fn enable_mouse_input(&mut self) {
        self.mode = self.mode | ENABLE_MOUSE_INPUT;
        self.set_mode(self.mode);
    }
    pub fn disable_mouse_input(&mut self) {
        self.mode = self.mode & ENABLE_MOUSE_INPUT;
        self.set_mode(self.mode);
    }

    ///
    ///
    /// This wraps
    /// [`SetConsoleMode`](https://learn.microsoft.com/en-us/windows/console/setconsolemode).
    pub fn set_mode(&self, mode: u32) {
        unsafe {
            let result = SetConsoleMode(self.handle, mode);
            if result != 1 {
                panic!("Failed to set console mode {:?}", mode);
            }
        }
    }
}

//[](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)

///Clear the entire screen, using `cmd /c cls`.
pub fn clear_all() {
    Command::new("cmd").args(["/C", "cls"]).status().unwrap();
}
pub fn show_cursor<W: Write>(w: &mut W) {
    write!(w, "\x1b[?25h").unwrap();
}
pub fn hide_cursor<W: Write>(w: &mut W) {
    write!(w, "\x1b[?25l").unwrap();
}
pub fn enter_alternate_screen<W: Write>(w: &mut W) {
    write!(w, "\x1b[?1049h").unwrap();
}
pub fn leave_alternate_screen<W: Write>(w: &mut W) {
    write!(w, "\x1b[?1049l").unwrap();
}
pub fn move_to<W: Write>(w: &mut W, x: u16, y: u16) {
    write!(w, "\x1b[{};{}H", y, x).unwrap();
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
