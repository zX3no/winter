#![feature(macro_metavar_expr, const_fn_floating_point_arithmetic)]
#![allow(non_camel_case_types, non_snake_case)]
use std::{
    fmt::Display,
    io::{stdin, stdout, Stdin, Stdout, Write},
    mem::zeroed,
    os::windows::io::AsRawHandle,
    process::Command,
    ptr::null_mut,
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

pub use buffer::{Buffer, Cell};
pub use layout::Alignment::*;
pub use style::{Color::*, *};
pub use win32::*;

//TODO: Remove
pub use layout::Constraint::*;
pub use layout::Direction::*;
pub use layout::*;

pub mod buffer;
pub mod layout;
pub mod style;
pub mod symbols;

pub struct Winter {
    pub viewport: Rect,
    pub buffers: [Buffer; 2],
    pub current: usize,
    pub stdout: Stdout,
    pub stdin: Stdin,
}

impl Winter {
    pub fn new() -> Self {
        let mut stdout = stdout();
        // let handle = current_in_handle();
        let stdin = stdin();
        let mode =
            (ENABLE_MOUSE_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_WINDOW_INPUT) & !NOT_RAW_MODE_MASK;
        assert!(mode & NOT_RAW_MODE_MASK == 0);
        set_mode(stdin.as_raw_handle(), mode);
        show_alternate_screen(&mut stdout);
        clear(&mut stdout);

        let raw = stdout.as_raw_handle();
        let (width, height) = info(raw).window_size;
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
        let (width, height) = info(self.stdout.as_raw_handle()).window_size;
        self.viewport = Rect::new(0, 0, width, height);

        //Resize
        if self.buffers[self.current].area != self.viewport {
            self.buffers[self.current].resize(self.viewport);
            self.buffers[1 - self.current].resize(self.viewport);

            // Reset the back buffer to make sure the next update will redraw everything.
            self.buffers[1 - self.current].reset();
            clear(&mut self.stdout);
        }
    }
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stdout.flush()
    }
    pub fn buffer(&mut self) -> &mut Buffer {
        &mut self.buffers[self.current]
    }
}

impl Drop for Winter {
    fn drop(&mut self) {
        uninit(&mut self.stdout, &mut self.stdin);
    }
}

pub struct Info {
    pub buffer_size: (u16, u16),
    ///Use this one.
    pub window_size: (u16, u16),
    pub cursor_position: (u16, u16),
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

const NOT_RAW_MODE_MASK: u32 = ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT;

/// Used with panic handlers.
///
/// ```
///  let orig_hook = std::panic::take_hook();
///  std::panic::set_hook(Box::new(move |panic_info| {
///      let mut stdout = stdout();
///      uninit(&mut stdout);
///      stdout.flush().unwrap();
///      orig_hook(panic_info);
///      std::process::exit(1);
///  }));
/// ```
pub fn uninit(stdout: &mut Stdout, stdin: &mut Stdin) {
    // let handle = current_in_handle();
    let stdin = stdin.as_raw_handle();
    let mut mode = get_mode(stdin);
    mode &= ENABLE_MOUSE_INPUT | ENABLE_EXTENDED_FLAGS | ENABLE_WINDOW_INPUT;
    mode |= NOT_RAW_MODE_MASK;
    assert!(mode & NOT_RAW_MODE_MASK != 0);
    set_mode(stdin, mode);
    hide_alternate_screen(stdout);
    show_cursor(stdout);
    stdout.flush().unwrap();
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

//TODO: This sucks.
use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
    pub struct KeyState: u16 {
        const CTRL              = 0b0000_0000_0001;
        const SHIFT             = 0b0000_0000_0010;
        const ALT               = 0b0000_0000_0100;
    }
}

impl KeyState {
    pub fn shift(&self) -> bool {
        self.contains(KeyState::SHIFT)
    }
    pub fn ctrl(&self) -> bool {
        self.contains(KeyState::CTRL)
    }
    pub fn alt(&self) -> bool {
        self.contains(KeyState::ALT)
    }
}

pub fn key_state(event: INPUT_RECORD) -> KeyState {
    if event.EventType == KEY_EVENT {
        let ks = unsafe { event.Event.KeyEvent().dwControlKeyState };
        let mut state = KeyState::empty();
        if ks & SHIFT_PRESSED != 0 {
            state |= KeyState::SHIFT;
        }

        if ks & LEFT_CTRL_PRESSED != 0 || ks & RIGHT_CTRL_PRESSED != 0 {
            state |= KeyState::CTRL;
        }

        if ks & LEFT_ALT_PRESSED != 0 || ks & RIGHT_ALT_PRESSED != 0 {
            state |= KeyState::ALT;
        }

        state
    } else {
        KeyState::empty()
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

///Read terminal events.
///```rs
///loop {
///    if let Some(event) = poll(Duration::from_millis(16)) {
///        dbg!(event);
///    }
///}
/// ```
pub fn poll(timeout: Duration) -> Option<(Event, KeyState)> {
    let now = Instant::now();
    //TODO: Stop grabbing this everywhere. File handles are expensive you know.
    let handle = current_in_handle();

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
