use crate::{buffer::Buffer, layout::Rect};

/// API Mockups
/// ```rs
/// let terminal = Terminal::new();
/// let text = widget::text(["blue text".blue(), "red text".red()]);
///
/// terminal.draw(text);
///
/// ```
///
/// ```rs
/// let terminal = Terminal::new();
/// let text = widget::text("blue underlined text", [blue(), underlined()]);
///
/// terminal.draw(text);
///
/// ```
#[derive(Debug)]
pub struct Terminal {
    /// Holds the results of the current and previous draw calls. The two are compared at the end
    /// of each draw pass to output the necessary updates to the terminal
    pub buffers: [Buffer; 2],
    /// Index of the current buffer in the previous array
    pub current: usize,
    pub cursor_hidden: bool,
    pub viewport: Rect,
}
