use crate::{buffer::Buffer, layout::Rect, symbols::*, Style, Text};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderType {
    Plain,
    Rounded,
    Double,
    Thick,
}

impl BorderType {
    pub fn line_symbols(border_type: BorderType) -> line::Set {
        match border_type {
            BorderType::Plain => line::NORMAL,
            BorderType::Rounded => line::ROUNDED,
            BorderType::Double => line::DOUBLE,
            BorderType::Thick => line::THICK,
        }
    }
}

pub fn draw<'a>(
    title: Option<Text<'a>>,
    borders: Borders,
    border_type: BorderType,
    style: Style,
    area: Rect,
    buf: &mut Buffer,
) {
    if area.area() == 0 {
        return;
    }
    // buf.set_style(area, self.style);
    let symbols = BorderType::line_symbols(border_type);

    // Sides
    if borders.intersects(Borders::LEFT) {
        for y in area.top()..area.bottom() {
            buf.get_mut(area.left(), y)
                .unwrap()
                .set_symbol(symbols.vertical)
                .set_style(style);
        }
    }
    if borders.intersects(Borders::TOP) {
        for x in area.left()..area.right() {
            buf.get_mut(x, area.top())
                .unwrap()
                .set_symbol(symbols.horizontal)
                .set_style(style);
        }
    }
    if borders.intersects(Borders::RIGHT) {
        let x = area.right() - 1;
        for y in area.top()..area.bottom() {
            buf.get_mut(x, y)
                .unwrap()
                .set_symbol(symbols.vertical)
                .set_style(style);
        }
    }
    if borders.intersects(Borders::BOTTOM) {
        let y = area.bottom() - 1;
        for x in area.left()..area.right() {
            buf.get_mut(x, y)
                .unwrap()
                .set_symbol(symbols.horizontal)
                .set_style(style);
        }
    }

    // Corners
    if borders.contains(Borders::RIGHT | Borders::BOTTOM) {
        buf.get_mut(area.right() - 1, area.bottom() - 1)
            .unwrap()
            .set_symbol(symbols.bottom_right)
            .set_style(style);
    }
    if borders.contains(Borders::RIGHT | Borders::TOP) {
        buf.get_mut(area.right() - 1, area.top())
            .unwrap()
            .set_symbol(symbols.top_right)
            .set_style(style);
    }
    if borders.contains(Borders::LEFT | Borders::BOTTOM) {
        buf.get_mut(area.left(), area.bottom() - 1)
            .unwrap()
            .set_symbol(symbols.bottom_left)
            .set_style(style);
    }
    if borders.contains(Borders::LEFT | Borders::TOP) {
        buf.get_mut(area.left(), area.top())
            .unwrap()
            .set_symbol(symbols.top_left)
            .set_style(style);
    }

    // Title
    if let Some(title) = title {
        let left_border_dx = if borders.intersects(Borders::LEFT) {
            1
        } else {
            0
        };

        let right_border_dx = if borders.intersects(Borders::RIGHT) {
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

        let title_x = area.left() + title_dx;
        let title_y = area.top();

        //TODO: Title style and title offset.
        buf.set_stringn(
            title_x,
            title_y,
            &title.text,
            title_area_width as usize,
            title.style,
        );
    }
}
