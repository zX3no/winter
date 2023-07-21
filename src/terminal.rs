use crate::buffer::Buffer;
use crate::rect::Rect;

/// Interface to the terminal backed by Termion
#[derive(Debug)]
pub struct Terminal {
    /// Holds the results of the current and previous draw calls. The two are compared at the end
    /// of each draw pass to output the necessary updates to the terminal
    buffers: [Buffer; 2],
    /// Index of the current buffer in the previous array
    current: usize,
    /// Whether the cursor is currently hidden
    hidden_cursor: bool,
    /// Viewport
    viewport: Rect,
}

/*
/// Synchronizes terminal size, calls the rendering closure, flushes the current internal state
/// and prepares for the next draw call.
pub fn draw<F>(&mut self, f: F) -> io::Result<CompletedFrame>
where
    F: FnOnce(&mut Frame<B>),
{
    // Autoresize - otherwise we get glitches if shrinking or potential desync between widgets
    // and the terminal (if growing), which may OOB.
    self.autoresize()?;

    let mut frame = self.get_frame();
    f(&mut frame);
    // We can't change the cursor position right away because we have to flush the frame to
    // stdout first. But we also can't keep the frame around, since it holds a &mut to
    // Terminal. Thus, we're taking the important data out of the Frame and dropping it.
    let cursor_position = frame.cursor_position;

    // Draw to stdout
    self.flush()?;

    match cursor_position {
        None => self.hide_cursor()?,
        Some((x, y)) => {
            self.show_cursor()?;
            self.set_cursor(x, y)?;
        }
    }

    // Swap buffers
    self.buffers[1 - self.current].reset();
    self.current = 1 - self.current;

    // Flush
    stdout.flush();
    Ok(CompletedFrame {
        buffer: &self.buffers[1 - self.current],
        area: self.viewport.area,
    })
}
/// Obtains a difference between the previous and the current buffer and passes it to the
/// current backend for drawing.
pub fn flush(&mut self) -> io::Result<()> {
    let previous_buffer = &self.buffers[1 - self.current];
    let current_buffer = &self.buffers[self.current];
    let updates = previous_buffer.diff(current_buffer);
    self.backend.draw(updates.into_iter())
}

/// Updates the Terminal so that internal buffers match the requested size. Requested size will
/// be saved so the size can remain consistent when rendering.
/// This leads to a full clear of the screen.
pub fn resize(&mut self, area: Rect) -> io::Result<()> {
    self.buffers[self.current].resize(area);
    self.buffers[1 - self.current].resize(area);
    self.viewport.area = area;
    self.clear()
}

/// Queries the backend for size and resizes if it doesn't match the previous size.
pub fn autoresize(&mut self) -> io::Result<()> {
    if self.viewport.resize_behavior == ResizeBehavior::Auto {
        let size = self.size()?;
        if size != self.viewport.area {
            self.resize(size)?;
        }
    };
    Ok(())
}
 */

#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum Color {
    /// Resets the terminal color.
    Reset,

    /// Black color.
    Black,

    /// Dark grey color.
    DarkGrey,

    /// Light red color.
    Red,

    /// Dark red color.
    DarkRed,

    /// Light green color.
    Green,

    /// Dark green color.
    DarkGreen,

    /// Light yellow color.
    Yellow,

    /// Dark yellow color.
    DarkYellow,

    /// Light blue color.
    Blue,

    /// Dark blue color.
    DarkBlue,

    /// Light magenta color.
    Magenta,

    /// Dark magenta color.
    DarkMagenta,

    /// Light cyan color.
    Cyan,

    /// Dark cyan color.
    DarkCyan,

    /// White color.
    White,

    /// Grey color.
    Grey,

    /// An RGB color. See [RGB color model](https://en.wikipedia.org/wiki/RGB_color_model) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    Rgb { r: u8, g: u8, b: u8 },

    /// An ANSI color. See [256 colors - cheat sheet](https://jonasjacek.github.io/colors/) for more info.
    ///
    /// Most UNIX terminals and Windows 10 supported only.
    /// See [Platform-specific notes](enum.Color.html#platform-specific-notes) for more info.
    AnsiValue(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Modifier;

impl Modifier {
    pub const BOLD: u16 = 0b0000_0000_0001;
    pub const DIM: u16 = 0b0000_0000_0010;
    pub const ITALIC: u16 = 0b0000_0000_0100;
    pub const UNDERLINED: u16 = 0b0000_0000_1000;
    pub const SLOW_BLINK: u16 = 0b0000_0001_0000;
    pub const RAPID_BLINK: u16 = 0b0000_0010_0000;
    pub const REVERSED: u16 = 0b0000_0100_0000;
    pub const HIDDEN: u16 = 0b0000_1000_0000;
    pub const CROSSED_OUT: u16 = 0b0001_0000_0000;
    pub fn empty() -> Self {
        Self {}
    }
}

/// A buffer cell
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub symbol: String,
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl Cell {
    pub fn set_symbol(&mut self, symbol: &str) -> &mut Cell {
        self.symbol.clear();
        self.symbol.push_str(symbol);
        self
    }

    pub fn set_char(&mut self, ch: char) -> &mut Cell {
        self.symbol.clear();
        self.symbol.push(ch);
        self
    }

    pub fn set_fg(&mut self, color: Color) -> &mut Cell {
        self.fg = color;
        self
    }

    pub fn set_bg(&mut self, color: Color) -> &mut Cell {
        self.bg = color;
        self
    }

    pub fn reset(&mut self) {
        self.symbol.clear();
        self.symbol.push(' ');
        self.fg = Color::Reset;
        self.bg = Color::Reset;
        self.modifier = Modifier::empty();
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell {
            symbol: " ".into(),
            fg: Color::Reset,
            bg: Color::Reset,
            modifier: Modifier::empty(),
        }
    }
}
