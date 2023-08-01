///![](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
use std::{
    collections::VecDeque,
    ffi::OsString,
    io::Write,
    mem::zeroed,
    os::windows::prelude::OsStringExt,
    process::Command,
    ptr::null_mut,
    sync::Mutex,
    time::{Duration, Instant},
};
use winapi::{
    ctypes::c_void,
    shared::{minwindef::DWORD, winerror::WAIT_TIMEOUT},
    um::{
        consoleapi::{
            GetConsoleMode, GetNumberOfConsoleInputEvents, ReadConsoleInputW, SetConsoleMode,
        },
        errhandlingapi::GetLastError,
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        processenv::GetStdHandle,
        synchapi::{WaitForMultipleObjects, WaitForSingleObject},
        winbase::{
            FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
            FORMAT_MESSAGE_IGNORE_INSERTS, INFINITE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
            WAIT_ABANDONED_0, WAIT_FAILED, WAIT_OBJECT_0,
        },
        wincon::{
            GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT,
            ENABLE_EXTENDED_FLAGS, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT,
            ENABLE_VIRTUAL_TERMINAL_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT,
        },
        wincontypes::{
            DOUBLE_CLICK, FROM_LEFT_1ST_BUTTON_PRESSED, FROM_LEFT_2ND_BUTTON_PRESSED,
            FROM_LEFT_3RD_BUTTON_PRESSED, FROM_LEFT_4TH_BUTTON_PRESSED, INPUT_RECORD, KEY_EVENT,
            MOUSE_EVENT, MOUSE_HWHEELED, MOUSE_MOVED, MOUSE_WHEELED, RIGHTMOST_BUTTON_PRESSED,
            WINDOW_BUFFER_SIZE_EVENT,
        },
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
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

pub fn current_in_handle() -> *mut c_void {
    let utf16: Vec<u16> = "CONIN$\0".encode_utf16().collect();
    let utf16_ptr: *const u16 = utf16.as_ptr();

    unsafe {
        CreateFileW(
            utf16_ptr,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            0,
            null_mut(),
        )
    }
}

const NOT_RAW_MODE_MASK: DWORD = ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT;
const ENABLE_MOUSE_MODE: u32 = 0x0010 | 0x0080 | 0x0008;

pub fn enable_mouse_capture() {
    let handle = current_in_handle();
    // let orignal_mode = get_mode(handle);
    set_mode(handle, ENABLE_MOUSE_MODE);
}

//TODO: Re-write this. Just testing how crossterm does it.
pub fn enable_raw_mode() {
    let handle = current_in_handle();
    let dw_mode = get_mode(handle);
    let new_mode = dw_mode & !NOT_RAW_MODE_MASK;
    set_mode(handle, new_mode);
}

pub fn disable_raw_mode() {
    let handle = current_in_handle();
    let dw_mode = get_mode(handle);
    let new_mode = dw_mode | NOT_RAW_MODE_MASK;
    set_mode(handle, new_mode);
}

pub fn is_raw_mode() -> bool {
    let handle = current_in_handle();
    let dw_mode = get_mode(handle);
    dw_mode & NOT_RAW_MODE_MASK == 0
}

/// This wraps
/// [`SetConsoleMode`](https://learn.microsoft.com/en-us/windows/console/setconsolemode).
pub fn set_mode(handle: *mut c_void, mode: u32) {
    unsafe {
        let result = SetConsoleMode(handle, mode);
        if result != 1 {
            let os_error = std::io::Error::last_os_error();
            panic!("Failed to set console mode to {}: {os_error:#}", mode);
        }
    }
}

pub fn get_mode(handle: *mut c_void) -> u32 {
    unsafe {
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) == 0 {
            panic!("Failed to get console mode");
        }
        mode
    }
}

// fn get_last_error_message(error_code: DWORD) -> String {
//     let mut buffer: *mut winapi::ctypes::c_void = null_mut();

//     unsafe {
//         let size = FormatMessageW(
//             FORMAT_MESSAGE_ALLOCATE_BUFFER
//                 | FORMAT_MESSAGE_FROM_SYSTEM
//                 | FORMAT_MESSAGE_IGNORE_INSERTS,
//             null_mut(),
//             error_code,
//             0,
//             &mut buffer as *mut _ as *mut u16,
//             0,
//             null_mut(),
//         );
//         if size == 0 {
//             panic!("Failed to get error message.");
//         }

//         let error_message = OsString::from_wide(std::slice::from_raw_parts(
//             buffer as *const u16,
//             size as usize,
//         ))
//         .to_string_lossy()
//         .trim()
//         .to_string();

//         LocalFree(buffer);
//         error_message
//     }
// }

pub fn handles() -> (*mut c_void, *mut c_void) {
    unsafe {
        let output = GetStdHandle(STD_OUTPUT_HANDLE);
        let input = GetStdHandle(STD_OUTPUT_HANDLE);
        if output == INVALID_HANDLE_VALUE || input == INVALID_HANDLE_VALUE {
            panic!("Failed to get the input handle");
        }
        (output, input)
    }
}

//TODO: windows starts counting at 0, unix at 1, add one to replicated unix behaviour.
//I still haven't figured out why my drawing is different than crossterm.
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
    pub output_mode: u32,
    pub input_mode: u32,
}

impl Terminal {
    pub fn new() -> Self {
        unsafe {
            let output = GetStdHandle(STD_OUTPUT_HANDLE);
            let input = GetStdHandle(STD_INPUT_HANDLE);

            let mut output_mode: u32 = 0;
            if GetConsoleMode(output, &mut output_mode) == 0 {
                panic!("Failed to get console mode");
            }

            let mut input_mode: u32 = 0;
            if GetConsoleMode(input, &mut input_mode) == 0 {
                panic!("Failed to get console mode");
            }

            Self {
                output,
                input,
                output_mode,
                input_mode,
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

    //TODO: Should this poll with mpsc? seems like a decent idea.
    //Although I kind of hate multi-threading things like this.
    //I'm thinking something like poll() with a Once loop with
    //a message channel. Wait for an event. If one becomes available return immediately.
    //
    //https://github.com/crossterm-rs/crossterm/blob/master/src/event/sys/windows/parse.rs
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

        // mode &= !ENABLE_EXTENDED_FLAGS;
        // mode &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);

        // mode |= ENABLE_MOUSE_MODE;

        //Enables resize events.
        mode |= ENABLE_WINDOW_INPUT;

        //Setting this flag directs user input into
        //Virtual Terminal Sequences that can be retrieved
        //through ReadFile or ReadConsole functions.
        // mode |= ENABLE_VIRTUAL_TERMINAL_INPUT;

        //This causes issues with raw mode.
        // mode |= ENABLE_PROCESSED_INPUT;

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
                        //Keep in mind mouse_event.dwControlKeyState for control clicks.
                        let mouse_event = events[i].Event.MouseEvent();
                        let event_flags = mouse_event.dwEventFlags;

                        match event_flags {
                            0 => {
                                if mouse_event.dwButtonState == FROM_LEFT_1ST_BUTTON_PRESSED {
                                    println!("Left mouse button pressed");
                                }
                                if mouse_event.dwButtonState == RIGHTMOST_BUTTON_PRESSED {
                                    println!("Right mouse button pressed");
                                }
                                if mouse_event.dwButtonState == FROM_LEFT_2ND_BUTTON_PRESSED {
                                    println!("Middle mouse button pressed");
                                }
                            }
                            DOUBLE_CLICK => {
                                if mouse_event.dwButtonState == FROM_LEFT_1ST_BUTTON_PRESSED {
                                    println!("Left mouse button double clicked");
                                }
                                if mouse_event.dwButtonState == RIGHTMOST_BUTTON_PRESSED {
                                    println!("Right mouse button double clicked");
                                }
                                if mouse_event.dwButtonState == FROM_LEFT_2ND_BUTTON_PRESSED {
                                    println!("Middle mouse button double clicked");
                                }
                            }
                            MOUSE_WHEELED => {
                                if mouse_event.dwButtonState == 0x800000 {
                                    println!("Mouse wheeled up");
                                }
                                if mouse_event.dwButtonState == 0xff800000 {
                                    println!("Mouse wheeled down");
                                }
                            }
                            // MOUSE_HWHEELED => {}
                            // MOUSE_MOVED => {}
                            _ => {}
                        }
                    }
                    WINDOW_BUFFER_SIZE_EVENT => {
                        let size = events[i].Event.WindowBufferSizeEvent().dwSize;
                        println!("{} {}", size.X, size.Y);
                    }
                    _ => (),
                }
            }
        }
    }
}

pub static EVENT_READER: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());

#[derive(Debug)]
pub enum Code {}

#[derive(Debug)]
pub enum Event {
    Mouse(Code),
    Key(Code),
    Resize(u16, u16),
}

///Blocking
pub fn read_single_input_event(handle: *mut c_void) -> INPUT_RECORD {
    let mut record: INPUT_RECORD = unsafe { zeroed() };

    // Convert an INPUT_RECORD to an &mut [INPUT_RECORD] of length 1
    let buf = std::slice::from_mut(&mut record);
    let num_read = read_input(handle, buf);

    // The windows API promises that ReadConsoleInput returns at least
    // 1 element
    debug_assert!(num_read == 1);

    record
}

fn read_input(handle: *mut c_void, buf: &mut [INPUT_RECORD]) -> u32 {
    let mut num_records = 0;
    debug_assert!(buf.len() < std::u32::MAX as usize);

    unsafe {
        let result =
            ReadConsoleInputW(handle, buf.as_mut_ptr(), buf.len() as u32, &mut num_records);
        if result == 0 {
            panic!("Failed to read input");
        }
        num_records
    }
}

pub unsafe fn convert_event(event: INPUT_RECORD) -> Option<Event> {
    match event.EventType {
        KEY_EVENT => {
            let key_event = event.Event.KeyEvent();
            if key_event.bKeyDown == 1 {
                println!("Key pressed: {}", key_event.wVirtualKeyCode);
            }
        }
        MOUSE_EVENT => {
            //Keep in mind mouse_event.dwControlKeyState for control clicks.
            let mouse_event = event.Event.MouseEvent();
            let event_flags = mouse_event.dwEventFlags;

            match event_flags {
                0 => {
                    if mouse_event.dwButtonState == FROM_LEFT_1ST_BUTTON_PRESSED {
                        println!("Left mouse button pressed");
                    }
                    if mouse_event.dwButtonState == RIGHTMOST_BUTTON_PRESSED {
                        println!("Right mouse button pressed");
                    }
                    if mouse_event.dwButtonState == FROM_LEFT_2ND_BUTTON_PRESSED {
                        println!("Middle mouse button pressed");
                    }
                }
                DOUBLE_CLICK => {
                    if mouse_event.dwButtonState == FROM_LEFT_1ST_BUTTON_PRESSED {
                        println!("Left mouse button double clicked");
                    }
                    if mouse_event.dwButtonState == RIGHTMOST_BUTTON_PRESSED {
                        println!("Right mouse button double clicked");
                    }
                    if mouse_event.dwButtonState == FROM_LEFT_2ND_BUTTON_PRESSED {
                        println!("Middle mouse button double clicked");
                    }
                }
                MOUSE_WHEELED => {
                    if mouse_event.dwButtonState == 0x800000 {
                        println!("Mouse wheeled up");
                    }
                    if mouse_event.dwButtonState == 0xff800000 {
                        println!("Mouse wheeled down");
                    }
                }
                // MOUSE_HWHEELED => {}
                // MOUSE_MOVED => {}
                _ => {}
            }
        }
        WINDOW_BUFFER_SIZE_EVENT => {
            let size = event.Event.WindowBufferSizeEvent().dwSize;
            println!("{} {}", size.X, size.Y);
        }
        _ => (),
    };
    None
}

//TODO: Does this ever return None?
pub fn poll_event(timeout: Option<Duration>) -> bool {
    let dw_millis = match timeout {
        Some(duration) => duration.as_millis() as u32,
        None => INFINITE,
    };

    let console_handle = current_in_handle();
    let output = unsafe { WaitForSingleObject(console_handle, dw_millis) };

    match output {
        WAIT_OBJECT_0 => {
            // input handle triggered
            true
        }
        WAIT_TIMEOUT | WAIT_ABANDONED_0 => {
            // timeout elapsed
            false
        }
        WAIT_FAILED => panic!("{:#}", std::io::Error::last_os_error()),
        _ => panic!("WaitForMultipleObjects returned unexpected result."),
    }
}

///Read terminal events.
///```rs
///loop {
///    if let Some(event) = read(Duration::from_millis(3)) {
///        dbg!(event);
///    }
///}
/// ```
pub fn read(timeout: Duration) -> Option<Event> {
    let now = Instant::now();

    let handle = current_in_handle();
    loop {
        let leftover = timeout.saturating_sub(now.elapsed());
        let n = number_of_console_input_events(handle);
        if poll_event(Some(leftover)) && n != 0 {
            let event = read_single_input_event(handle);
            return unsafe { convert_event(event) };
        }

        //Timeout elapsed
        if now.elapsed().as_millis() >= timeout.as_millis() {
            return None;
        }
    }
}

pub fn number_of_console_input_events(handle: *mut c_void) -> u32 {
    let mut buf_len: DWORD = 0;
    let result = unsafe { GetNumberOfConsoleInputEvents(handle, &mut buf_len) };
    if result == 0 {
        panic!("Could not get number of console input events.");
    }
    buf_len
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
