pub use std::os::raw::c_void;
pub type HANDLE = *mut c_void;

#[link(name = "user32")]
extern "system" {
    //TODO: Swap to A
    pub fn ReadConsoleInputW(
        hConsoleInput: *mut c_void,
        lpBuffer: *mut INPUT_RECORD,
        nLength: u32,
        lpNumberOfEventsRead: *mut u32,
    ) -> i32;
    pub fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: u32) -> i32;
    pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut u32) -> i32;
    pub fn GetNumberOfConsoleInputEvents(hConsoleInput: HANDLE, lpNumberOfEvents: *mut u32) -> i32;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: u32) -> u32;
    //TODO: Swap to A
    pub fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: u32,
        dwShareMode: u32,
        lpSecurityAttributes: *mut SECURITY_ATTRIBUTES,
        dwCreationDisposition: u32,
        dwFlagsAndAttributes: u32,
        hTemplateFile: HANDLE,
    ) -> HANDLE;

    pub fn GetConsoleScreenBufferInfo(
        hConsoleOutput: HANDLE,
        lpConsoleScreenBufferInfo: *mut CONSOLE_SCREEN_BUFFER_INFO,
    ) -> i32;

    pub fn ToUnicode(
        wVirtKey: u32,
        wScanCode: u32,
        lpKeyState: *const u8,
        lwszBuff: *mut u16,
        cchBuff: i32,
        wFlags: u32,
    ) -> i32;

}

#[repr(C)]
pub struct SECURITY_ATTRIBUTES {
    pub nLength: u32,
    pub lpSecurityDescriptor: *mut c_void,
    pub bInheritHandle: i32,
}

#[repr(C)]
pub struct CONSOLE_SCREEN_BUFFER_INFO {
    pub dwSize: COORD,
    pub dwCursorPosition: COORD,
    pub wAttributes: u16,
    pub srWindow: SMALL_RECT,
    pub dwMaximumWindowSize: COORD,
}

#[repr(C)]
pub struct SMALL_RECT {
    pub Left: u16,
    pub Top: u16,
    pub Right: u16,
    pub Bottom: u16,
}

pub const FILE_SHARE_READ: u32 = 0x00000001;
pub const FILE_SHARE_WRITE: u32 = 0x00000002;
pub const FILE_SHARE_DELETE: u32 = 0x00000004;
pub const GENERIC_READ: u32 = 0x80000000;
pub const GENERIC_WRITE: u32 = 0x40000000;
pub const GENERIC_EXECUTE: u32 = 0x20000000;
pub const GENERIC_ALL: u32 = 0x10000000;

pub const WAIT_TIMEOUT: u32 = 258;

pub const CTRL_C_EVENT: u32 = 0;
pub const CTRL_BREAK_EVENT: u32 = 1;
pub const CTRL_CLOSE_EVENT: u32 = 2;
pub const CTRL_LOGOFF_EVENT: u32 = 5;
pub const CTRL_SHUTDOWN_EVENT: u32 = 6;
pub const ENABLE_PROCESSED_INPUT: u32 = 0x0001;
pub const ENABLE_LINE_INPUT: u32 = 0x0002;
pub const ENABLE_ECHO_INPUT: u32 = 0x0004;
pub const ENABLE_WINDOW_INPUT: u32 = 0x0008;
pub const ENABLE_MOUSE_INPUT: u32 = 0x0010;
pub const ENABLE_INSERT_MODE: u32 = 0x0020;
pub const ENABLE_QUICK_EDIT_MODE: u32 = 0x0040;
pub const ENABLE_EXTENDED_FLAGS: u32 = 0x0080;
pub const ENABLE_AUTO_POSITION: u32 = 0x0100;
pub const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;
pub const ENABLE_PROCESSED_OUTPUT: u32 = 0x0001;
pub const ENABLE_WRAP_AT_EOL_OUTPUT: u32 = 0x0002;
pub const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
pub const DISABLE_NEWLINE_AUTO_RETURN: u32 = 0x0008;
pub const ENABLE_LVB_GRID_WORLDWIDE: u32 = 0x0010;

pub const INFINITE: u32 = 0xFFFFFFFF;
pub const STD_INPUT_HANDLE: u32 = -10i32 as u32;
pub const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;

pub const STATUS_ABANDONED_WAIT_0: u32 = 0x00000080;
pub const WAIT_ABANDONED_0: u32 = STATUS_ABANDONED_WAIT_0 as u32;
pub const STATUS_WAIT_0: u32 = 0x00000000;
pub const WAIT_OBJECT_0: u32 = STATUS_WAIT_0 as u32;
pub const WAIT_FAILED: u32 = 0xFFFFFFFF;

pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;

pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
// pub const STATUS_CONTROL_C_EXIT: i32 = 0xC000013A;
// pub const CONTROL_C_EXIT: u32 = STATUS_CONTROL_C_EXIT as u32;

pub const VK_LBUTTON: i32 = 0x01;
pub const VK_RBUTTON: i32 = 0x02;
pub const VK_CANCEL: i32 = 0x03;
pub const VK_MBUTTON: i32 = 0x04;
pub const VK_XBUTTON1: i32 = 0x05;
pub const VK_XBUTTON2: i32 = 0x06;
pub const VK_BACK: i32 = 0x08;
pub const VK_TAB: i32 = 0x09;
pub const VK_CLEAR: i32 = 0x0C;
pub const VK_RETURN: i32 = 0x0D;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_MENU: i32 = 0x12;
pub const VK_PAUSE: i32 = 0x13;
pub const VK_CAPITAL: i32 = 0x14;
pub const VK_KANA: i32 = 0x15;
pub const VK_HANGEUL: i32 = 0x15;
pub const VK_HANGUL: i32 = 0x15;
pub const VK_JUNJA: i32 = 0x17;
pub const VK_FINAL: i32 = 0x18;
pub const VK_HANJA: i32 = 0x19;
pub const VK_KANJI: i32 = 0x19;
pub const VK_ESCAPE: i32 = 0x1B;
pub const VK_CONVERT: i32 = 0x1C;
pub const VK_NONCONVERT: i32 = 0x1D;
pub const VK_ACCEPT: i32 = 0x1E;
pub const VK_MODECHANGE: i32 = 0x1F;
pub const VK_SPACE: i32 = 0x20;
pub const VK_PRIOR: i32 = 0x21;
pub const VK_NEXT: i32 = 0x22;
pub const VK_END: i32 = 0x23;
pub const VK_HOME: i32 = 0x24;
pub const VK_LEFT: i32 = 0x25;
pub const VK_UP: i32 = 0x26;
pub const VK_RIGHT: i32 = 0x27;
pub const VK_DOWN: i32 = 0x28;
pub const VK_SELECT: i32 = 0x29;
pub const VK_PRINT: i32 = 0x2A;
pub const VK_EXECUTE: i32 = 0x2B;
pub const VK_SNAPSHOT: i32 = 0x2C;
pub const VK_INSERT: i32 = 0x2D;
pub const VK_DELETE: i32 = 0x2E;
pub const VK_HELP: i32 = 0x2F;
pub const VK_LWIN: i32 = 0x5B;
pub const VK_RWIN: i32 = 0x5C;
pub const VK_APPS: i32 = 0x5D;
pub const VK_SLEEP: i32 = 0x5F;
pub const VK_NUMPAD0: i32 = 0x60;
pub const VK_NUMPAD1: i32 = 0x61;
pub const VK_NUMPAD2: i32 = 0x62;
pub const VK_NUMPAD3: i32 = 0x63;
pub const VK_NUMPAD4: i32 = 0x64;
pub const VK_NUMPAD5: i32 = 0x65;
pub const VK_NUMPAD6: i32 = 0x66;
pub const VK_NUMPAD7: i32 = 0x67;
pub const VK_NUMPAD8: i32 = 0x68;
pub const VK_NUMPAD9: i32 = 0x69;
pub const VK_MULTIPLY: i32 = 0x6A;
pub const VK_ADD: i32 = 0x6B;
pub const VK_SEPARATOR: i32 = 0x6C;
pub const VK_SUBTRACT: i32 = 0x6D;
pub const VK_DECIMAL: i32 = 0x6E;
pub const VK_DIVIDE: i32 = 0x6F;
pub const VK_F1: i32 = 0x70;
pub const VK_F2: i32 = 0x71;
pub const VK_F3: i32 = 0x72;
pub const VK_F4: i32 = 0x73;
pub const VK_F5: i32 = 0x74;
pub const VK_F6: i32 = 0x75;
pub const VK_F7: i32 = 0x76;
pub const VK_F8: i32 = 0x77;
pub const VK_F9: i32 = 0x78;
pub const VK_F10: i32 = 0x79;
pub const VK_F11: i32 = 0x7A;
pub const VK_F12: i32 = 0x7B;
pub const VK_F13: i32 = 0x7C;
pub const VK_F14: i32 = 0x7D;
pub const VK_F15: i32 = 0x7E;
pub const VK_F16: i32 = 0x7F;
pub const VK_F17: i32 = 0x80;
pub const VK_F18: i32 = 0x81;
pub const VK_F19: i32 = 0x82;
pub const VK_F20: i32 = 0x83;
pub const VK_F21: i32 = 0x84;
pub const VK_F22: i32 = 0x85;
pub const VK_F23: i32 = 0x86;
pub const VK_F24: i32 = 0x87;
pub const VK_NAVIGATION_VIEW: i32 = 0x88;
pub const VK_NAVIGATION_MENU: i32 = 0x89;
pub const VK_NAVIGATION_UP: i32 = 0x8A;
pub const VK_NAVIGATION_DOWN: i32 = 0x8B;
pub const VK_NAVIGATION_LEFT: i32 = 0x8C;
pub const VK_NAVIGATION_RIGHT: i32 = 0x8D;
pub const VK_NAVIGATION_ACCEPT: i32 = 0x8E;
pub const VK_NAVIGATION_CANCEL: i32 = 0x8F;
pub const VK_NUMLOCK: i32 = 0x90;
pub const VK_SCROLL: i32 = 0x91;
pub const VK_OEM_NEC_EQUAL: i32 = 0x92;
pub const VK_OEM_FJ_JISHO: i32 = 0x92;
pub const VK_OEM_FJ_MASSHOU: i32 = 0x93;
pub const VK_OEM_FJ_TOUROKU: i32 = 0x94;
pub const VK_OEM_FJ_LOYA: i32 = 0x95;
pub const VK_OEM_FJ_ROYA: i32 = 0x96;
pub const VK_LSHIFT: i32 = 0xA0;
pub const VK_RSHIFT: i32 = 0xA1;
pub const VK_LCONTROL: i32 = 0xA2;
pub const VK_RCONTROL: i32 = 0xA3;
pub const VK_LMENU: i32 = 0xA4;
pub const VK_RMENU: i32 = 0xA5;
pub const VK_BROWSER_BACK: i32 = 0xA6;
pub const VK_BROWSER_FORWARD: i32 = 0xA7;
pub const VK_BROWSER_REFRESH: i32 = 0xA8;
pub const VK_BROWSER_STOP: i32 = 0xA9;
pub const VK_BROWSER_SEARCH: i32 = 0xAA;
pub const VK_BROWSER_FAVORITES: i32 = 0xAB;
pub const VK_BROWSER_HOME: i32 = 0xAC;
pub const VK_VOLUME_MUTE: i32 = 0xAD;
pub const VK_VOLUME_DOWN: i32 = 0xAE;
pub const VK_VOLUME_UP: i32 = 0xAF;
pub const VK_MEDIA_NEXT_TRACK: i32 = 0xB0;
pub const VK_MEDIA_PREV_TRACK: i32 = 0xB1;
pub const VK_MEDIA_STOP: i32 = 0xB2;
pub const VK_MEDIA_PLAY_PAUSE: i32 = 0xB3;
pub const VK_LAUNCH_MAIL: i32 = 0xB4;
pub const VK_LAUNCH_MEDIA_SELECT: i32 = 0xB5;
pub const VK_LAUNCH_APP1: i32 = 0xB6;
pub const VK_LAUNCH_APP2: i32 = 0xB7;
pub const VK_OEM_1: i32 = 0xBA;
pub const VK_OEM_PLUS: i32 = 0xBB;
pub const VK_OEM_COMMA: i32 = 0xBC;
pub const VK_OEM_MINUS: i32 = 0xBD;
pub const VK_OEM_PERIOD: i32 = 0xBE;
pub const VK_OEM_2: i32 = 0xBF;
pub const VK_OEM_3: i32 = 0xC0;
pub const VK_GAMEPAD_A: i32 = 0xC3;
pub const VK_GAMEPAD_B: i32 = 0xC4;
pub const VK_GAMEPAD_X: i32 = 0xC5;
pub const VK_GAMEPAD_Y: i32 = 0xC6;
pub const VK_GAMEPAD_RIGHT_SHOULDER: i32 = 0xC7;
pub const VK_GAMEPAD_LEFT_SHOULDER: i32 = 0xC8;
pub const VK_GAMEPAD_LEFT_TRIGGER: i32 = 0xC9;
pub const VK_GAMEPAD_RIGHT_TRIGGER: i32 = 0xCA;
pub const VK_GAMEPAD_DPAD_UP: i32 = 0xCB;
pub const VK_GAMEPAD_DPAD_DOWN: i32 = 0xCC;
pub const VK_GAMEPAD_DPAD_LEFT: i32 = 0xCD;
pub const VK_GAMEPAD_DPAD_RIGHT: i32 = 0xCE;
pub const VK_GAMEPAD_MENU: i32 = 0xCF;
pub const VK_GAMEPAD_VIEW: i32 = 0xD0;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_BUTTON: i32 = 0xD1;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_BUTTON: i32 = 0xD2;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_UP: i32 = 0xD3;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_DOWN: i32 = 0xD4;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_RIGHT: i32 = 0xD5;
pub const VK_GAMEPAD_LEFT_THUMBSTICK_LEFT: i32 = 0xD6;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_UP: i32 = 0xD7;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_DOWN: i32 = 0xD8;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_RIGHT: i32 = 0xD9;
pub const VK_GAMEPAD_RIGHT_THUMBSTICK_LEFT: i32 = 0xDA;
pub const VK_OEM_4: i32 = 0xDB;
pub const VK_OEM_5: i32 = 0xDC;
pub const VK_OEM_6: i32 = 0xDD;
pub const VK_OEM_7: i32 = 0xDE;
pub const VK_OEM_8: i32 = 0xDF;
pub const VK_OEM_AX: i32 = 0xE1;
pub const VK_OEM_102: i32 = 0xE2;
pub const VK_ICO_HELP: i32 = 0xE3;
pub const VK_ICO_00: i32 = 0xE4;
pub const VK_PROCESSKEY: i32 = 0xE5;
pub const VK_ICO_CLEAR: i32 = 0xE6;
pub const VK_PACKET: i32 = 0xE7;
pub const VK_OEM_RESET: i32 = 0xE9;
pub const VK_OEM_JUMP: i32 = 0xEA;
pub const VK_OEM_PA1: i32 = 0xEB;
pub const VK_OEM_PA2: i32 = 0xEC;
pub const VK_OEM_PA3: i32 = 0xED;
pub const VK_OEM_WSCTRL: i32 = 0xEE;
pub const VK_OEM_CUSEL: i32 = 0xEF;
pub const VK_OEM_ATTN: i32 = 0xF0;
pub const VK_OEM_FINISH: i32 = 0xF1;
pub const VK_OEM_COPY: i32 = 0xF2;
pub const VK_OEM_AUTO: i32 = 0xF3;
pub const VK_OEM_ENLW: i32 = 0xF4;
pub const VK_OEM_BACKTAB: i32 = 0xF5;
pub const VK_ATTN: i32 = 0xF6;
pub const VK_CRSEL: i32 = 0xF7;
pub const VK_EXSEL: i32 = 0xF8;
pub const VK_EREOF: i32 = 0xF9;
pub const VK_PLAY: i32 = 0xFA;
pub const VK_ZOOM: i32 = 0xFB;
pub const VK_NONAME: i32 = 0xFC;
pub const VK_PA1: i32 = 0xFD;
pub const VK_OEM_CLEAR: i32 = 0xFE;

pub const RIGHT_ALT_PRESSED: u32 = 0x0001;
pub const LEFT_ALT_PRESSED: u32 = 0x0002;
pub const RIGHT_CTRL_PRESSED: u32 = 0x0004;
pub const LEFT_CTRL_PRESSED: u32 = 0x0008;
pub const SHIFT_PRESSED: u32 = 0x0010;
pub const NUMLOCK_ON: u32 = 0x0020;
pub const SCROLLLOCK_ON: u32 = 0x0040;
pub const CAPSLOCK_ON: u32 = 0x0080;

pub const KEY_EVENT: u16 = 0x0001;
pub const MOUSE_EVENT: u16 = 0x0002;
pub const WINDOW_BUFFER_SIZE_EVENT: u16 = 0x0004;
pub const MENU_EVENT: u16 = 0x0008;
pub const FOCUS_EVENT: u16 = 0x0010;

pub const FROM_LEFT_1ST_BUTTON_PRESSED: u32 = 0x0001;
pub const RIGHTMOST_BUTTON_PRESSED: u32 = 0x0002;
pub const FROM_LEFT_2ND_BUTTON_PRESSED: u32 = 0x0004;
pub const FROM_LEFT_3RD_BUTTON_PRESSED: u32 = 0x0008;
pub const FROM_LEFT_4TH_BUTTON_PRESSED: u32 = 0x0010;
pub const MOUSE_MOVED: u32 = 0x0001;
pub const DOUBLE_CLICK: u32 = 0x0002;
pub const MOUSE_WHEELED: u32 = 0x0004;
pub const MOUSE_HWHEELED: u32 = 0x0008;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct INPUT_RECORD {
    pub EventType: u16,
    pub Event: INPUT_RECORD_Event,
}

#[repr(C)]
pub struct KEY_EVENT_RECORD {
    pub bKeyDown: i32,
    pub wRepeatCount: u16,
    pub wVirtualKeyCode: u16,
    pub wVirtualScanCode: u16,
    pub uChar: KEY_EVENT_RECORD_uChar,
    pub dwControlKeyState: u32,
}

#[repr(C)]
pub struct MOUSE_EVENT_RECORD {
    pub dwMousePosition: COORD,
    pub dwButtonState: u32,
    pub dwControlKeyState: u32,
    pub dwEventFlags: u32,
}

#[repr(C)]
pub struct WINDOW_BUFFER_SIZE_RECORD {
    pub dwSize: COORD,
}

#[repr(C)]
pub struct MENU_EVENT_RECORD {
    pub dwCommandId: u32,
}

#[repr(C)]
pub struct FOCUS_EVENT_RECORD {
    pub bSetFocus: i32,
}

#[repr(C)]
pub struct COORD {
    pub X: i16,
    pub Y: i16,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct INPUT_RECORD_Event([u32; 4]);

impl INPUT_RECORD_Event {
    #[inline]
    pub unsafe fn KeyEvent(&self) -> &KEY_EVENT_RECORD {
        &*(self as *const _ as *const KEY_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn KeyEvent_mut(&mut self) -> &mut KEY_EVENT_RECORD {
        &mut *(self as *mut _ as *mut KEY_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn MouseEvent(&self) -> &MOUSE_EVENT_RECORD {
        &*(self as *const _ as *const MOUSE_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn MouseEvent_mut(&mut self) -> &mut MOUSE_EVENT_RECORD {
        &mut *(self as *mut _ as *mut MOUSE_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn WindowBufferSizeEvent(&self) -> &WINDOW_BUFFER_SIZE_RECORD {
        &*(self as *const _ as *const WINDOW_BUFFER_SIZE_RECORD)
    }
    #[inline]
    pub unsafe fn WindowBufferSizeEvent_mut(&mut self) -> &mut WINDOW_BUFFER_SIZE_RECORD {
        &mut *(self as *mut _ as *mut WINDOW_BUFFER_SIZE_RECORD)
    }
    #[inline]
    pub unsafe fn MenuEvent(&self) -> &MENU_EVENT_RECORD {
        &*(self as *const _ as *const MENU_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn MenuEvent_mut(&mut self) -> &mut MENU_EVENT_RECORD {
        &mut *(self as *mut _ as *mut MENU_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn FocusEvent(&self) -> &FOCUS_EVENT_RECORD {
        &*(self as *const _ as *const FOCUS_EVENT_RECORD)
    }
    #[inline]
    pub unsafe fn FocusEvent_mut(&mut self) -> &mut FOCUS_EVENT_RECORD {
        &mut *(self as *mut _ as *mut FOCUS_EVENT_RECORD)
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct KEY_EVENT_RECORD_uChar([u16; 1]);

impl KEY_EVENT_RECORD_uChar {
    #[inline]
    pub unsafe fn UnicodeChar(&self) -> &u16 {
        &*(self as *const _ as *const u16)
    }
    #[inline]
    pub unsafe fn UnicodeChar_mut(&mut self) -> &mut u16 {
        &mut *(self as *mut _ as *mut u16)
    }
    #[inline]
    pub unsafe fn AsciiChar(&self) -> &i8 {
        &*(self as *const _ as *const i8)
    }
    #[inline]
    pub unsafe fn AsciiChar_mut(&mut self) -> &mut i8 {
        &mut *(self as *mut _ as *mut i8)
    }
}
