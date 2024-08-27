use std::ffi::c_void;
use std::io::{stdout, Stdin, Stdout};
use std::os::windows::io::AsRawHandle;
use std::time::{Duration, Instant};
use std::{io::stdin, mem::zeroed};

use crate::{
    Event, GetConsoleScreenBufferInfo, Info, KeyState, CONSOLE_SCREEN_BUFFER_INFO, ENABLE_EXTENDED_FLAGS, ENABLE_MOUSE_INPUT, ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, ENABLE_WINDOW_INPUT, INPUT_RECORD
};

// #[cfg(target_os = "windows")]
pub fn initialise() -> (Stdout, Stdin) {
    //Enable ANSI codes on conhost terminals, can also use:
    //[HKEY_CURRENT_USER\Console]
    //"VirtualTerminalLevel"=dword:00000001
    //https://ss64.com/nt/syntax-ansi.html

        let mut stdout = stdout();
        let stdin = stdin();

    set_mode(
        stdout.as_raw_handle(),
        ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING,
    );

    set_mode(
        stdin.as_raw_handle(),
        ENABLE_WINDOW_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_MOUSE_INPUT,
    );

    (stdout, stdin)
}

///TODO: windows starts counting at 0, unix at 1, add one to replicated unix behaviour.
///I still haven't figured out why my drawing is different than crossterm.
pub fn info(output: *mut c_void) -> Info {
    unsafe {
        let mut info: CONSOLE_SCREEN_BUFFER_INFO = zeroed();
        let result = GetConsoleScreenBufferInfo(output, &mut info);
        if result != 1 {
            panic!("Could not get window size. result: {}", result);
        }
        Info {
            buffer_size: (info.dwSize.X as u16, info.dwSize.Y as u16),
            window_size: (
                (info.srWindow.Right - info.srWindow.Left) as u16 + 1,
                (info.srWindow.Bottom - info.srWindow.Top) as u16 + 1,
            ),
            cursor_position: (
                info.dwCursorPosition.X as u16,
                info.dwCursorPosition.Y as u16,
            ),
        }
    }
}

pub fn window_size(stdout: &Stdout) -> (u16, u16) {
    stdout.as_raw_handle().window_size
}

pub fn reset_stdin(stdin: &mut Stdin) {
    set_mode(stdin.as_raw_handle(), 0);
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

pub fn poll_timeout(stdin: &Stdin, timeout: Duration) -> Option<(Event, KeyState)> {
    let now = Instant::now();
    let handle = stdin.as_raw_handle();

    loop {
        let leftover = timeout.saturating_sub(now.elapsed());
        if event_ready(handle, Some(leftover)) && event_count(handle) != 0 {
            let input_event = read_input_event(handle);
            let event = unsafe { convert_event(input_event.clone()) };
            //TODO: This could be done better.
            return if let Some(event) = event {
                let state = key_state(input_event);
                Some((event, state))
            } else {
                None
            };
        }

        //Timeout elapsed
        if now.elapsed().as_millis() >= timeout.as_millis() {
            return None;
        }
    }
}

pub const CONTROL: u32 = 0b0000_0000_0001;
pub const SHIFT: u32 = 0b0000_0000_0010;
pub const ALT: u32 = 0b0000_0000_0100;

pub struct KeyState(u32);

pub struct Info {
    pub buffer_size: (u16, u16),
    ///Use this one.
    pub window_size: (u16, u16),
    pub cursor_position: (u16, u16),
}

impl KeyState {
    pub fn control(&self) -> bool {
        (self.0 & CONTROL) != 0
    }
    pub fn shift(&self) -> bool {
        (self.0 & SHIFT) != 0
    }
    pub fn alt(&self) -> bool {
        (self.0 & ALT) != 0
    }
}

pub fn key_state(event: INPUT_RECORD) -> KeyState {
    if event.EventType == KEY_EVENT {
        let ks = unsafe { event.Event.KeyEvent().dwControlKeyState };
        let mut state = 0;
        if ks & SHIFT_PRESSED != 0 {
            state |= SHIFT;
        }
        if ks & LEFT_CTRL_PRESSED != 0 || ks & RIGHT_CTRL_PRESSED != 0 {
            state |= CONTROL;
        }
        if ks & LEFT_ALT_PRESSED != 0 || ks & RIGHT_ALT_PRESSED != 0 {
            state |= ALT;
        }
        KeyState(state)
    } else {
        KeyState(0)
    }
}

pub unsafe fn convert_event(event: INPUT_RECORD) -> Option<Event> {
    match event.EventType {
        KEY_EVENT => {
            let key_event = event.Event.KeyEvent();
            if key_event.bKeyDown == 1 {
                const F24: i32 = VK_F1 + 23;
                let vk = key_event.wVirtualKeyCode;
                let sc = key_event.wVirtualScanCode;
                let shift = key_event.dwControlKeyState & SHIFT_PRESSED != 0;

                match vk as i32 {
                    VK_UP => return Some(Event::Up),
                    VK_DOWN => return Some(Event::Down),
                    VK_LEFT => return Some(Event::Left),
                    VK_RIGHT => return Some(Event::Right),
                    VK_RETURN => return Some(Event::Enter),
                    VK_SPACE => return Some(Event::Char(' ')),
                    VK_BACK => return Some(Event::Backspace),
                    VK_ESCAPE => return Some(Event::Escape),
                    VK_TAB => return Some(Event::Tab),
                    VK_SHIFT | VK_LSHIFT | VK_RSHIFT => return Some(Event::Shift),
                    VK_CONTROL | VK_LCONTROL | VK_RCONTROL => return Some(Event::Control),
                    VK_MENU | VK_LMENU | VK_RMENU => return Some(Event::Alt),

                    //TODO: Tilde is kind of an odd ball.
                    //Might need to handle this one better.
                    VK_OEM_PLUS if shift => return Some(Event::Char('+')),
                    VK_OEM_MINUS if shift => return Some(Event::Char('_')),
                    VK_OEM_3 if shift => return Some(Event::Char('~')),
                    VK_OEM_4 if shift => return Some(Event::Char('{')),
                    VK_OEM_6 if shift => return Some(Event::Char('}')),
                    VK_OEM_5 if shift => return Some(Event::Char('|')),
                    VK_OEM_1 if shift => return Some(Event::Char(':')),
                    VK_OEM_7 if shift => return Some(Event::Char('"')),
                    VK_OEM_COMMA if shift => return Some(Event::Char('<')),
                    VK_OEM_PERIOD if shift => return Some(Event::Char('>')),
                    VK_OEM_2 if shift => return Some(Event::Char('?')),
                    VK_OEM_PLUS => return Some(Event::Char('=')),
                    VK_OEM_MINUS => return Some(Event::Char('-')),

                    VK_OEM_3 => return Some(Event::Char('`')),
                    VK_OEM_4 => return Some(Event::Char('[')),
                    VK_OEM_6 => return Some(Event::Char(']')),
                    VK_OEM_5 => return Some(Event::Char('\\')),
                    VK_OEM_1 => return Some(Event::Char(';')),
                    VK_OEM_7 => return Some(Event::Char('\'')),
                    VK_OEM_COMMA => return Some(Event::Char(',')),
                    VK_OEM_PERIOD => return Some(Event::Char('.')),
                    VK_OEM_2 => return Some(Event::Char('/')),

                    VK_F1..=F24 => return Some(Event::Function((vk - VK_F1 as u16 + 1) as u8)),
                    // Handle alphanumeric keys (A-Z, 0-9).
                    0x30..=0x39 | 0x41..=0x5A => {
                        // Buffer to hold the Unicode characters.
                        let mut buffer: [u16; 5] = [0; 5];

                        //This would return a multi-character key if this wasn't blank.
                        let keyboard_state: [u8; 256] = [0; 256];

                        let result = ToUnicode(
                            vk as u32,
                            sc as u32,
                            keyboard_state.as_ptr(),
                            buffer.as_mut_ptr(),
                            buffer.len() as i32,
                            0,
                        );
                        match result {
                            1 if shift => {
                                let char = buffer[0] as u8 as char;
                                return Some(Event::Char(match char {
                                    '1' => '!',
                                    '2' => '@',
                                    '3' => '#',
                                    '4' => '$',
                                    '5' => '%',
                                    '6' => '^',
                                    '7' => '&',
                                    '8' => '*',
                                    '9' => '(',
                                    '0' => ')',
                                    _ => char.to_ascii_uppercase(),
                                }));
                            }
                            1 => return Some(Event::Char(buffer[0] as u8 as char)),
                            _ => unimplemented!(),
                        }
                    }
                    _ => return Some(Event::Unknown(vk)),
                }
            }
        }
        MOUSE_EVENT => {
            //Keep in mind mouse_event.dwControlKeyState for control clicks.
            let mouse_event = event.Event.MouseEvent();
            let event_flags = mouse_event.dwEventFlags;
            let (x, y) = (
                mouse_event.dwMousePosition.X as u16,
                mouse_event.dwMousePosition.Y as u16,
            );

            match event_flags {
                //TODO: Double click event?
                0 | DOUBLE_CLICK => match mouse_event.dwButtonState {
                    FROM_LEFT_1ST_BUTTON_PRESSED => return Some(Event::LeftMouse(x, y)),
                    RIGHTMOST_BUTTON_PRESSED => return Some(Event::RightMouse(x, y)),
                    FROM_LEFT_2ND_BUTTON_PRESSED => return Some(Event::MiddleMouse(x, y)),
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
            let size = &event.Event.WindowBufferSizeEvent().dwSize;
            return Some(Event::Resize(size.X as u16, size.Y as u16));
        }
        _ => (),
    };
    None
}

pub fn read_input_event(input: *mut c_void) -> INPUT_RECORD {
    let mut record: INPUT_RECORD = unsafe { zeroed() };

    //Convert an INPUT_RECORD to an &mut [INPUT_RECORD] of length 1
    let buf = std::slice::from_mut(&mut record);
    let mut num_records = 0;
    debug_assert!(buf.len() < std::u32::MAX as usize);
    let result =
        unsafe { ReadConsoleInputW(input, buf.as_mut_ptr(), buf.len() as u32, &mut num_records) };
    if result == 0 {
        panic!("Failed to read input");
    }

    //The windows API promises that ReadConsoleInput returns at least 1 element.
    debug_assert!(num_records == 1);

    record
}

pub fn event_ready(input: *mut c_void, timeout: Option<Duration>) -> bool {
    let dw_millis = match timeout {
        Some(duration) => duration.as_millis() as u32,
        None => INFINITE,
    };

    let output = unsafe { WaitForSingleObject(input, dw_millis) };

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

pub fn event_count(input: *mut c_void) -> u32 {
    let mut buf_len: u32 = 0;
    let result = unsafe { GetNumberOfConsoleInputEvents(input, &mut buf_len) };
    if result == 0 {
        panic!("Could not get number of console input events.");
    }
    buf_len
}
