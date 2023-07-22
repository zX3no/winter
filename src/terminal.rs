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
