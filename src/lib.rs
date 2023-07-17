use std::{mem::zeroed, process::Command};
use winapi::{
    ctypes::c_void,
    um::{
        consoleapi::{SetConsoleMode, WriteConsoleW},
        processenv::GetStdHandle,
        wincon::{GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO},
    },
};

mod rect;
mod terminal;

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
    pub buffer_size: (i16, i16),
    pub terminal_size: (i16, i16),
}

pub fn window_info(term: &Terminal) -> Info {
    unsafe {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let result = GetConsoleScreenBufferInfo(term.handle, &mut info);
        if result != 1 {
            panic!("Could not get window size.");
        } else {
            Info {
                buffer_size: (info.dwSize.X, info.dwSize.Y),
                terminal_size: (
                    info.srWindow.Right - info.srWindow.Left,
                    info.srWindow.Bottom - info.srWindow.Top,
                ),
            }
        }
    }
}

pub fn write_char(term: &Terminal, buf: &[u8]) {
    let utf8 = std::str::from_utf8(buf).unwrap();
    let utf16: Vec<u16> = utf8.encode_utf16().collect();
    let utf16_ptr = utf16.as_ptr() as *const c_void;
    let mut cells_written: u32 = 0;

    let result = unsafe {
        WriteConsoleW(
            term.handle,
            utf16_ptr,
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

pub fn move_to(x: i16, y: i16) {
    print!("\x1b[{};{}H", y, x);
}

pub fn clear_all() {
    Command::new("cmd").args(["/C", "cls"]).status().unwrap();
}

pub fn clear() {
    print!("\x1b[2J");
}

pub fn shift_up(amount: u16) {
    print!("\x1b[{}S", amount);
}

pub fn shift_down(amount: u16) {
    print!("\x1b[{}T", amount);
}
