use crate::{buffer::Buffer, layout::Rect, symbols::*, *};
use bitflags::bitflags;

bitflags! {
    /// Bitflags that can be composed to set the visible borders essentially on the block widget.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Borders: u32 {
        /// Show no border (default)
        const NONE  = 0b0000_0001;
        /// Show the top border
        const TOP   = 0b0000_0010;
        /// Show the right border
        const RIGHT = 0b0000_0100;
        /// Show the bottom border
        const BOTTOM = 0b000_1000;
        /// Show the left border
        const LEFT = 0b0001_0000;
        /// Show all borders
        const ALL = Self::TOP.bits() | Self::RIGHT.bits() | Self::BOTTOM.bits() | Self::LEFT.bits();
    }
}

pub const ALL: Borders = Borders::ALL;
pub const TOP: Borders = Borders::TOP;
pub const RIGHT: Borders = Borders::RIGHT;
pub const BOTTOM: Borders = Borders::BOTTOM;
pub const LEFT: Borders = Borders::LEFT;
pub const NONE: Borders = Borders::NONE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    Plain,
    Rounded,
    Double,
    Thick,
}

//TODO: This is probably better than a macro.
//Macros aren't powerfull enough to allow omitting all fields.
//This is simple and consistant.
//It allows gets people in the habbit of using style()
//instead of omitting it.

//TODO: Title has bad ergonomics. Some("title".into())
//Maybe title should be Option<Text<'a>> that way it has it's own style too?
pub const fn block(
    title: Option<Text<'_>>,
    borders: Borders,
    border_type: BorderType,
) -> Block<'_> {
    Block {
        title,
        margin: 0,
        borders,
        border_type,
        style: style(),
    }
}

//TODO: Title alignment?
#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub title: Option<Text<'a>>,
    pub margin: u16,
    // pub title_alignment: Alignment,
    pub borders: Borders,
    pub border_type: BorderType,
    pub style: Style,
}

impl<'a> Block<'a> {
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    pub fn margin(mut self, title_margin: u16) -> Self {
        self.margin = title_margin;
        self
    }
    /// Compute the inner area of a block based on its border visibility rules.
    pub fn inner(&self, area: Rect) -> Rect {
        let mut inner = area;
        if self.borders.intersects(Borders::LEFT) {
            inner.x = inner.x.saturating_add(1).min(inner.right());
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::TOP) || self.title.is_some() {
            inner.y = inner.y.saturating_add(1).min(inner.bottom());
            inner.height = inner.height.saturating_sub(1);
        }
        if self.borders.intersects(Borders::RIGHT) {
            inner.width = inner.width.saturating_sub(1);
        }
        if self.borders.intersects(Borders::BOTTOM) {
            inner.height = inner.height.saturating_sub(1);
        }
        inner
    }
    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        if area.area() == 0 {
            return;
        }
        let symbols = match self.border_type {
            BorderType::Plain => line::NORMAL,
            BorderType::Rounded => line::ROUNDED,
            BorderType::Double => line::DOUBLE,
            BorderType::Thick => line::THICK,
        };

        // Sides
        if self.borders.intersects(Borders::LEFT) {
            for y in area.top()..area.bottom() {
                buf.get_mut(area.left(), y)
                    .set_symbol(symbols.vertical)
                    .set_style(self.style);
            }
        }
        if self.borders.intersects(Borders::TOP) {
            for x in area.left()..area.right() {
                buf.get_mut(x, area.top())
                    .set_symbol(symbols.horizontal)
                    .set_style(self.style);
            }
        }
        if self.borders.intersects(Borders::RIGHT) {
            let x = area.right() - 1;
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_symbol(symbols.vertical)
                    .set_style(self.style);
            }
        }
        if self.borders.intersects(Borders::BOTTOM) {
            let y = area.bottom() - 1;
            for x in area.left()..area.right() {
                buf.get_mut(x, y)
                    .set_symbol(symbols.horizontal)
                    .set_style(self.style);
            }
        }

        // Corners
        if self.borders.contains(Borders::RIGHT | Borders::BOTTOM) {
            buf.get_mut(area.right() - 1, area.bottom() - 1)
                .set_symbol(symbols.bottom_right)
                .set_style(self.style);
        }
        if self.borders.contains(Borders::RIGHT | Borders::TOP) {
            buf.get_mut(area.right() - 1, area.top())
                .set_symbol(symbols.top_right)
                .set_style(self.style);
        }
        if self.borders.contains(Borders::LEFT | Borders::BOTTOM) {
            buf.get_mut(area.left(), area.bottom() - 1)
                .set_symbol(symbols.bottom_left)
                .set_style(self.style);
        }
        if self.borders.contains(Borders::LEFT | Borders::TOP) {
            buf.get_mut(area.left(), area.top())
                .set_symbol(symbols.top_left)
                .set_style(self.style);
        }

        // Title
        if let Some(title) = &self.title {
            let left_border_dx = if self.borders.intersects(Borders::LEFT) {
                1
            } else {
                0
            };

            let right_border_dx = if self.borders.intersects(Borders::RIGHT) {
                1
            } else {
                0
            };

            let title_area_width = area
                .width
                .saturating_sub(left_border_dx)
                .saturating_sub(right_border_dx);

            // let title_dx = match title_alignment {
            //     Alignment::Left => left_border_dx,
            //     Alignment::Center => area.width.saturating_sub(title.width() as u16) / 2,
            //     Alignment::Right => area
            //         .width
            //         .saturating_sub(title.width() as u16)
            //         .saturating_sub(right_border_dx),
            // };
            let title_dx = left_border_dx;
            let title_x = area.left() + title_dx + self.margin;
            let title_y = area.top();

            buf.set_stringn(
                title_x,
                title_y,
                title,
                title_area_width as usize,
                title.style,
            );
        }
    }
}
