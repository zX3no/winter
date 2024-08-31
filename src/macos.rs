use crate::{Event, KeyModifiers};
use crossterm::{
    event::{
        read, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode, KeyEventKind,
        KeyModifiers as CKeyModifiers, MouseButton, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{stdin, stdout, Stdin, Stdout},
    time::Duration,
};

extern "C" {
    pub fn ioctl(fd: i32, request: u64, ...) -> i32;
}

#[repr(C)]
struct TermSize {
    row: u16,
    col: u16,
    x: u16,
    y: u16,
}

pub const STDOUT_FILENO: i32 = 1;

pub const TIOCGWINSZ: u64 = 0x40087468;

pub fn terminal_size() -> (u16, u16) {
    unsafe {
        let mut size: TermSize = core::mem::zeroed();
        //TODO: Handle errors.
        ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut size as *mut _);
        (size.col as u16, size.row as u16)
    }
}

pub fn initialise() -> (Stdout, Stdin) {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    let stdin = stdin();

    execute!(stdout, EnableMouseCapture).unwrap();

    (stdout, stdin)
}

pub fn uninit(stdout: &mut Stdout) {
    disable_raw_mode().unwrap();
    execute!(stdout, DisableMouseCapture).unwrap();
}

pub fn poll() -> Option<(Event, KeyModifiers)> {
    if !crossterm::event::poll(Duration::from_millis(16)).ok()? {
        return None;
    }

    match read().ok()? {
        CEvent::Key(key) if key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat => {
            let mut modifiers = 0;
            if key.modifiers.contains(CKeyModifiers::SHIFT) {
                modifiers |= KeyModifiers::SHIFT;
            }
            if key.modifiers.contains(CKeyModifiers::CONTROL) {
                modifiers |= KeyModifiers::CONTROL;
            }
            if key.modifiers.contains(CKeyModifiers::ALT) {
                modifiers |= KeyModifiers::ALT;
            }
            let modifiers = KeyModifiers(modifiers);

            match key.code {
                KeyCode::Backspace => Some((Event::Backspace, modifiers)),
                KeyCode::Enter => Some((Event::Enter, modifiers)),
                KeyCode::Left => Some((Event::Left, modifiers)),
                KeyCode::Right => Some((Event::Right, modifiers)),
                KeyCode::Up => Some((Event::Up, modifiers)),
                KeyCode::Down => Some((Event::Down, modifiers)),
                KeyCode::Tab => Some((Event::Tab, modifiers)),
                KeyCode::F(f) => Some((Event::Function(f), modifiers)),
                KeyCode::Char(char) => Some((Event::Char(char), modifiers)),
                KeyCode::Esc => Some((Event::Escape, modifiers)),
                _ => None,
            }
        }
        CEvent::Mouse(mouse) => {
            let mut modifiers = 0;
            if mouse.modifiers.contains(CKeyModifiers::SHIFT) {
                modifiers |= KeyModifiers::SHIFT;
            }
            if mouse.modifiers.contains(CKeyModifiers::CONTROL) {
                modifiers |= KeyModifiers::CONTROL;
            }
            if mouse.modifiers.contains(CKeyModifiers::ALT) {
                modifiers |= KeyModifiers::ALT;
            }
            let modifiers = KeyModifiers(modifiers);

            match mouse.kind {
                MouseEventKind::Down(button) => match button {
                    MouseButton::Left => {
                        Some((Event::LeftMouse(mouse.column, mouse.row), modifiers))
                    }
                    MouseButton::Right => {
                        Some((Event::RightMouse(mouse.column, mouse.row), modifiers))
                    }
                    MouseButton::Middle => {
                        Some((Event::MiddleMouse(mouse.column, mouse.row), modifiers))
                    }
                },
                MouseEventKind::ScrollUp => Some((Event::ScrollUp, modifiers)),
                MouseEventKind::ScrollDown => Some((Event::ScrollDown, modifiers)),
                _ => None,
            }
        }
        CEvent::Resize(width, height) => Some((Event::Resize(width, height), KeyModifiers(0))),
        _ => None,
    }
}
