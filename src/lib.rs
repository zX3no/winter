///![](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
use std::{
    ffi::OsString, io::Write, mem::zeroed, os::windows::prelude::OsStringExt, process::Command,
    ptr::null_mut,
};
use winapi::{
    ctypes::c_void,
    shared::minwindef::DWORD,
    um::{
        consoleapi::{GetConsoleMode, ReadConsoleInputW, SetConsoleMode},
        errhandlingapi::GetLastError,
        handleapi::INVALID_HANDLE_VALUE,
        processenv::GetStdHandle,
        winbase::{
            FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
            FORMAT_MESSAGE_IGNORE_INSERTS, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
        },
        wincon::{
            GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT,
            ENABLE_EXTENDED_FLAGS, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT,
            ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_WINDOW_INPUT,
        },
        wincontypes::{
            FROM_LEFT_1ST_BUTTON_PRESSED, INPUT_RECORD, KEY_EVENT, MOUSE_EVENT,
            RIGHTMOST_BUTTON_PRESSED,
        },
    },
};

//Widgets
pub use block::*;
pub use block::*;
pub use guage::*;
pub use list::*;
pub use table::*;
pub use text::*;

pub mod block;
pub mod guage;
pub mod list;
pub mod table;
pub mod text;

pub use buffer::{Buffer, Cell};
pub use layout::*;
pub use style::{Color::*, *};

pub mod buffer;
pub mod layout;
pub mod style;
pub mod symbols;

pub struct Info {
    pub buffer_size: (u16, u16),
    pub window_size: (u16, u16),
    pub cursor_position: (u16, u16),
}

fn get_last_error_message(error_code: DWORD) -> String {
    let mut buffer: *mut winapi::ctypes::c_void = null_mut();

    unsafe {
        let size = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            null_mut(),
            error_code,
            0,
            &mut buffer as *mut _ as *mut u16,
            0,
            null_mut(),
        );
        if size == 0 {
            panic!("Failed to get error message.");
        }

        let error_message = OsString::from_wide(std::slice::from_raw_parts(
            buffer as *const u16,
            size as usize,
        ))
        .to_string_lossy()
        .trim()
        .to_string();

        LocalFree(buffer);
        error_message
    }
}

pub fn handles() -> (*mut c_void, *mut c_void) {
    unsafe {
        (
            GetStdHandle(STD_OUTPUT_HANDLE),
            GetStdHandle(STD_INPUT_HANDLE),
        )
    }
}

pub fn area(output_handle: *mut c_void) -> (u16, u16) {
    unsafe {
        // let handle = GetStdHandle(-11i32 as u32);
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let result = GetConsoleScreenBufferInfo(output_handle, &mut info);
        if result != 1 {
            panic!("Could not get window size. result: {}", result);
        }
        (
            (info.srWindow.Right - info.srWindow.Left) as u16,
            (info.srWindow.Bottom - info.srWindow.Top) as u16,
        )
    }
}

//TODO: This struct serves zero purpose.
//You should delete this now.
pub struct Terminal {
    pub output: *mut c_void,
    pub input: *mut c_void,
    pub mode: u32,
}

impl Terminal {
    pub fn new() -> Self {
        unsafe {
            let input = GetStdHandle(STD_INPUT_HANDLE);
            let mut mode: u32 = 0;
            if GetConsoleMode(input, &mut mode) == 0 {
                panic!("Failed to get console mode");
            }
            Self {
                output: GetStdHandle(STD_OUTPUT_HANDLE),
                input: GetStdHandle(STD_INPUT_HANDLE),
                mode,
            }
        }
    }

    ///Get the terminal area that is usable.
    pub fn area(&self) -> (u16, u16) {
        unsafe {
            // let handle = GetStdHandle(-11i32 as u32);
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
            let result = GetConsoleScreenBufferInfo(self.output, &mut info);
            if result != 1 {
                panic!("Could not get window size. result: {}", result);
            }
            (
                (info.srWindow.Right - info.srWindow.Left) as u16,
                (info.srWindow.Bottom - info.srWindow.Top) as u16,
            )
        }
    }

    /// Get the screen buffer information like terminal size, cursor position, buffer size.
    ///
    /// This wraps
    /// [`GetConsoleScreenBufferInfo`](https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfo).
    pub fn info(&self) -> Info {
        unsafe {
            let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
            let result = GetConsoleScreenBufferInfo(self.output, &mut info);
            if result != 1 {
                panic!("Could not get window size. result: {}", result);
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
        self.mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;
        self.set_mode(self.mode);
    }
    pub fn disable_raw_mode(&mut self) {
        self.mode &= ENABLE_VIRTUAL_TERMINAL_INPUT;
        self.set_mode(self.mode);
    }
    pub fn enable_mouse_input(&mut self) {
        self.mode |= ENABLE_MOUSE_INPUT;
        self.set_mode(self.mode);
    }
    pub fn disable_mouse_input(&mut self) {
        self.mode &= ENABLE_MOUSE_INPUT;
        self.set_mode(self.mode);
    }
    pub fn enable_window_input(&mut self) {
        self.mode |= ENABLE_WINDOW_INPUT;
        self.set_mode(self.mode);
    }
    pub fn disable_window_input(&mut self) {
        self.mode &= ENABLE_WINDOW_INPUT;
        self.set_mode(self.mode);
    }

    /// This wraps
    /// [`SetConsoleMode`](https://learn.microsoft.com/en-us/windows/console/setconsolemode).
    pub fn set_mode(&self, mode: u32) {
        unsafe {
            let result = SetConsoleMode(self.output, mode);
            if result != 1 {
                let error_code = GetLastError();
                let err = get_last_error_message(error_code);
                panic!("Failed to set console mode to {}: {}", mode, err);
            }
        }
    }

    //TODO: This does not work.
    pub fn reset_mode() {
        let terminal = Terminal::new();
        terminal.set_mode(0);
    }

    //TODO: Should this poll with mpsc? seems like a decent idea.
    //Although I kind of hate multi-threading things like this.
    pub unsafe fn test() {
        //Note: This is an input handle not output.
        let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };

        if handle == INVALID_HANDLE_VALUE {
            panic!("Failed to get the input handle");
        }

        // Set the console input mode to raw input
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            panic!("Failed to get console mode");
        }

        mode &= !ENABLE_EXTENDED_FLAGS; // Disable extended flags
        mode &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT); // Disable line and echo input

        mode |= ENABLE_MOUSE_INPUT;
        mode |= ENABLE_WINDOW_INPUT;
        mode |= ENABLE_PROCESSED_INPUT;

        if SetConsoleMode(handle, mode) == 0 {
            panic!("Failed to set console mode");
        }

        let mut events: [INPUT_RECORD; 128] = std::mem::zeroed();
        let mut events_read: DWORD = 0;

        loop {
            if ReadConsoleInputW(
                handle,
                events.as_mut_ptr(),
                events.len() as DWORD,
                &mut events_read,
            ) == 0
            {
                panic!("Failed to read console input");
            }

            for i in 0..events_read as usize {
                match events[i].EventType {
                    KEY_EVENT => {
                        let key_event = events[i].Event.KeyEvent();
                        if key_event.bKeyDown == 1 {
                            println!("Key pressed: {}", key_event.wVirtualKeyCode);
                        }
                    }
                    MOUSE_EVENT => {
                        let mouse_event = events[i].Event.MouseEvent();
                        let event_flags = mouse_event.dwEventFlags;

                        if event_flags & FROM_LEFT_1ST_BUTTON_PRESSED != 0 {
                            println!("Left mouse button pressed");
                        }

                        if event_flags & RIGHTMOST_BUTTON_PRESSED != 0 {
                            println!("Right mouse button pressed");
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}

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
