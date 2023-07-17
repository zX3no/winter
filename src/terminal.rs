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

/// A buffer that maps to the desired content of the terminal after the draw call
///
/// No widget in the library interacts directly with the terminal. Instead each of them is required
/// to draw their state to an intermediate buffer. It is basically a grid where each cell contains
/// a grapheme, a foreground color and a background color. This grid will then be used to output
/// the appropriate escape sequences and characters to draw the UI as the user has defined it.
///
/// # Examples:
///
/// ```
/// use tui::buffer::{Buffer, Cell};
/// use tui::layout::Rect;
/// use tui::style::{Color, Style, Modifier};
///
/// let mut buf = Buffer::empty(Rect{x: 0, y: 0, width: 10, height: 5});
/// buf.get_mut(0, 2).set_symbol("x");
/// assert_eq!(buf.get(0, 2).symbol, "x");
/// buf.set_string(3, 0, "string", Style::default().fg(Color::Red).bg(Color::White));
/// assert_eq!(buf.get(5, 0), &Cell{
///     symbol: String::from("r"),
///     fg: Color::Red,
///     bg: Color::White,
///     modifier: Modifier::empty()
/// });
/// buf.get_mut(5, 0).set_char('x');
/// assert_eq!(buf.get(5, 0).symbol, "x");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Buffer {
    /// The area represented by this buffer
    pub area: Rect,
    /// The content of the buffer. The length of this Vec should always be equal to area.width *
    /// area.height
    pub content: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
    Rgb(u8, u8, u8),
    Indexed(u8),
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

    // pub fn set_style(&mut self, style: Style) -> &mut Cell {
    //     if let Some(c) = style.fg {
    //         self.fg = c;
    //     }
    //     if let Some(c) = style.bg {
    //         self.bg = c;
    //     }
    //     self.modifier.insert(style.add_modifier);
    //     self.modifier.remove(style.sub_modifier);
    //     self
    // }

    // pub fn style(&self) -> Style {
    //     Style::default()
    //         .fg(self.fg)
    //         .bg(self.bg)
    //         .add_modifier(self.modifier)
    // }

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
