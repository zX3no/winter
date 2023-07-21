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

pub use style::Stylize;

pub mod block;
pub mod buffer;
pub mod layout;
pub mod spans;
pub mod test_style;

pub mod color;
pub mod rect;
pub mod style;
pub mod symbols;
pub mod terminal;

pub const STD_HANDLE: u32 = -11i32 as u32;

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
