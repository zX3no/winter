#![allow(soft_unstable)]
use std::{ffi::OsStr, mem::zeroed, os::windows::prelude::OsStrExt, process::Command};
use winapi::{
    ctypes::c_void,
    um::{
        consoleapi::{SetConsoleMode, WriteConsoleW},
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
pub mod terminal;

pub const STD_HANDLE: u32 = -11i32 as u32;

pub enum ConsoleMode {
    EnableVirtualInputProcessing = 0x0004,
}

pub struct Info {
    pub buffer_size: (u16, u16),
    pub window_size: (u16, u16),
    pub cursor_position: (u16, u16),
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
            } else {
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
    }

    ///
    ///
    /// This wraps
    /// [`SetConsoleMode`](https://learn.microsoft.com/en-us/windows/console/setconsolemode).
    pub fn set_mode(&self, mode: ConsoleMode) {
        unsafe {
            SetConsoleMode(self.handle, mode as u32);
        }
    }
    ///
    ///
    /// This wraps
    /// [`WriteConsoleW`](https://learn.microsoft.com/en-us/windows/console/writeconsole).
    pub fn write(&self, buf: &[u8]) {
        let utf16: Vec<u16> = OsStr::new(std::str::from_utf8(buf).unwrap())
            .encode_wide()
            .collect();

        let mut cells_written: u32 = 0;

        let result = unsafe {
            WriteConsoleW(
                self.handle,
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
}

//[](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
pub fn show_cursor() {
    print!("\x1b[?25h");
}

pub fn hide_cursor() {
    print!("\x1b[?25l");
}

pub fn enter_alternate_screen() {
    print!("\x1b[?1049h");
}

pub fn leave_alternate_screen() {
    print!("\x1b[?1049l");
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
