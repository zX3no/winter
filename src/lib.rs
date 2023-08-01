//TODO: Remove
#![allow(unused)]
///![](https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences)
use std::{
    collections::VecDeque,
    io::Write,
    mem::zeroed,
    process::Command,
    ptr::null_mut,
    sync::Mutex,
    time::{Duration, Instant},
};
use winapi::um::{
    winnt::CHAR,
    winuser::{ToUnicode, VK_TAB},
};
use winapi::{
    ctypes::c_void,
    shared::{minwindef::DWORD, winerror::WAIT_TIMEOUT},
    um::{
        consoleapi::{
            GetConsoleMode, GetNumberOfConsoleInputEvents, ReadConsoleInputW, SetConsoleMode,
        },
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::INVALID_HANDLE_VALUE,
        processenv::GetStdHandle,
        synchapi::WaitForSingleObject,
        winbase::{
            INFINITE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, WAIT_ABANDONED_0, WAIT_FAILED,
            WAIT_OBJECT_0,
        },
        wincon::{
            GetConsoleScreenBufferInfo, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT,
            ENABLE_EXTENDED_FLAGS, ENABLE_LINE_INPUT, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_INPUT,
            ENABLE_QUICK_EDIT_MODE, ENABLE_VIRTUAL_TERMINAL_INPUT,
            ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT,
        },
        wincontypes::{
            DOUBLE_CLICK, ENHANCED_KEY, FROM_LEFT_1ST_BUTTON_PRESSED, FROM_LEFT_2ND_BUTTON_PRESSED,
            INPUT_RECORD, KEY_EVENT, MOUSE_EVENT, MOUSE_WHEELED, RIGHTMOST_BUTTON_PRESSED,
            WINDOW_BUFFER_SIZE_EVENT,
        },
        winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE},
    },
};
use winapi::{
    shared::minwindef::BYTE,
    um::winuser::{
        VK_BACK, VK_CONTROL, VK_ESCAPE, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_MENU, VK_RCONTROL,
        VK_RETURN, VK_RMENU, VK_RSHIFT, VK_SHIFT, VK_SPACE,
    },
};
use winapi::{shared::ntdef::WCHAR, um::winuser::VK_F1};

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

//TODO: What is the difference between this and `GetStdHandle(STD_INPUT_HANDLE)`
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

//TODO: This enables raw mode
///Also breaks CTRL+C
pub fn enable_mouse_capture() {
    let handle = current_in_handle();
    set_mode(
        handle,
        //Also turns off text selection.
        ENABLE_MOUSE_INPUT | ENABLE_EXTENDED_FLAGS,
    );
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

//TODO: Figure out what flags are actually useful and move into a function.
//Something like enable_terminal_events() or something.
pub fn enable_resize_events() {
    let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };

    if handle == INVALID_HANDLE_VALUE {
        panic!("Failed to get the input handle");
    }

    let mut mode = get_mode(handle);

    // mode &= !ENABLE_EXTENDED_FLAGS;
    // mode &= !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);

    // mode |= ENABLE_MOUSE_MODE;

    //Enables resize events.
    mode |= ENABLE_WINDOW_INPUT;
    set_mode(handle, mode);
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

/// Get the screen buffer information like terminal size, cursor position, buffer size.
///
/// This wraps
/// [`GetConsoleScreenBufferInfo`](https://docs.microsoft.com/en-us/windows/console/getconsolescreenbufferinfo).
pub fn info(handle: *mut c_void) -> Info {
    unsafe {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let result = GetConsoleScreenBufferInfo(handle, &mut info);
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

///TODO: What is a good way to do events.
///I don't like Event::Mouse(Mouse::Left).
///Why not just Event::LeftMouse ?
///That way you can do:
///```rs
/// if event == Event::LeftMouse {
///     println!("Pressed left mouse button!");
/// }
///```
/// Also how are we going to handle double clicks?
/// They might be nice to have...or not.
#[derive(Debug)]
pub enum Event {
    //Mouse
    LeftMouse,
    RightMouse,
    MiddleMouse,
    ScrollUp,
    ScrollDown,

    //Key
    Char(char),
    Function(u8),
    Space,
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

    //Other
    Resize(u16, u16),
}

///Blocking
pub fn read_single_input_event(handle: *mut c_void) -> INPUT_RECORD {
    let mut record: INPUT_RECORD = unsafe { zeroed() };

    //Convert an INPUT_RECORD to an &mut [INPUT_RECORD] of length 1
    let buf = std::slice::from_mut(&mut record);
    let mut num_records = 0;
    debug_assert!(buf.len() < std::u32::MAX as usize);
    let result =
        unsafe { ReadConsoleInputW(handle, buf.as_mut_ptr(), buf.len() as u32, &mut num_records) };
    if result == 0 {
        panic!("Failed to read input");
    }

    //The windows API promises that ReadConsoleInput returns at least 1 element.
    debug_assert!(num_records == 1);

    record
}

pub unsafe fn convert_event(event: INPUT_RECORD) -> Option<Event> {
    match event.EventType {
        KEY_EVENT => {
            let key_event = event.Event.KeyEvent();
            if key_event.bKeyDown == 1 {
                let virtual_keycode = key_event.wVirtualKeyCode;
                let scan_code = key_event.wVirtualScanCode;
                let is_extended = (key_event.dwControlKeyState & ENHANCED_KEY) != 0;
                const F24: i32 = VK_F1 + 23;

                match virtual_keycode as i32 {
                    VK_RETURN => return Some(Event::Enter),
                    VK_SPACE => return Some(Event::Space),
                    VK_BACK => return Some(Event::Backspace),
                    VK_ESCAPE => return Some(Event::Escape),
                    VK_TAB => return Some(Event::Tab),
                    VK_SHIFT | VK_LSHIFT | VK_RSHIFT => return Some(Event::Shift),
                    VK_CONTROL | VK_LCONTROL | VK_RCONTROL => return Some(Event::Control),
                    VK_MENU | VK_LMENU | VK_RMENU => return Some(Event::Alt),
                    VK_F1..=F24 => {
                        return Some(Event::Function((virtual_keycode - VK_F1 as u16 + 1) as u8))
                    }
                    // Handle alphanumeric keys (A-Z, 0-9).
                    0x30..=0x39 | 0x41..=0x5A => {
                        // Buffer to hold the Unicode characters.
                        let mut buffer: [WCHAR; 5] = [0; 5];

                        //This would return a multi-character key if this wasn't blank.
                        let mut keyboard_state: [BYTE; 256] = [0; 256];

                        let result = ToUnicode(
                            virtual_keycode as u32,
                            scan_code as u32,
                            keyboard_state.as_ptr(),
                            buffer.as_mut_ptr(),
                            buffer.len() as i32,
                            0,
                        );
                        match result {
                            // The key is a dead key or is a key that has no translation.
                            // This is usually used for keys that modify other characters, like accent keys.
                            -1 => todo!("Dead key or no translation."),
                            // The key is a dead key and a buffer is not provided, or the key is not a dead key but is a key that does not produce a character.
                            0 => todo!("No character produced."),
                            // The key is a single character key.
                            1 => return Some(Event::Char(buffer[0] as u8 as char)),
                            _ => {
                                // The key is a multi-character key, e.g., a CTRL+key combination.
                                let result_chars: Vec<WCHAR> =
                                    buffer.iter().take(result as usize).copied().collect();
                                let result_str: String =
                                    result_chars.iter().map(|&c| c as u8 as char).collect();
                                todo!("Multi-character produced: {}", result_str);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        MOUSE_EVENT => {
            //Keep in mind mouse_event.dwControlKeyState for control clicks.
            let mouse_event = event.Event.MouseEvent();
            let event_flags = mouse_event.dwEventFlags;

            match event_flags {
                //TODO: Double click event?
                0 | DOUBLE_CLICK => match mouse_event.dwButtonState {
                    FROM_LEFT_1ST_BUTTON_PRESSED => return Some(Event::LeftMouse),
                    RIGHTMOST_BUTTON_PRESSED => return Some(Event::RightMouse),
                    FROM_LEFT_2ND_BUTTON_PRESSED => return Some(Event::MiddleMouse),
                    _ => (),
                },
                MOUSE_WHEELED => {
                    if mouse_event.dwButtonState == 0x800000 {
                        return Some(Event::ScrollUp);
                    }
                    if mouse_event.dwButtonState == 0xff800000 {
                        return Some(Event::ScrollDown);
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
///    if let Some(event) = poll(Duration::from_millis(3)) {
///        dbg!(event);
///    }
///}
/// ```
pub fn poll(timeout: Duration) -> Option<Event> {
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
