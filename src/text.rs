use crate::{block::Block, buffer::Buffer, layout::Rect, Style};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

//TODO: Text alignment?
// fn get_line_offset(line_width: u16, text_area_width: u16, alignment: Alignment) -> u16 {
//     match alignment {
//         Alignment::Center => (text_area_width / 2).saturating_sub(line_width / 2),
//         Alignment::Right => text_area_width.saturating_sub(line_width),
//         Alignment::Left => 0,
//     }
// }

//TODO: Should `Lines` impl Deref so that lines[0].width() will work?
#[derive(Debug, Clone)]
pub struct Lines<'a> {
    pub lines: &'a [Text<'a>],
    pub block: Option<Block<'a>>,
    pub style: Option<Style>,
}

impl<'a> Lines<'a> {
    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        let area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        let mut lines_iter = self.lines.iter();
        for y in area.top()..area.bottom() {
            if let Some(line) = lines_iter.next() {
                let mut chars = line.text.chars();
                for x in area.left()..area.right() {
                    if let Some(char) = chars.next() {
                        if let Some(style) = self.style {
                            buf.get_mut(x, y).unwrap().set_char(char).set_style(style);
                        } else {
                            buf.get_mut(x, y)
                                .unwrap()
                                .set_char(char)
                                .set_style(line.style);
                        }
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
    pub fn draw_wrapping(&self, area: Rect, buf: &mut Buffer) {
        if self.lines.is_empty() {
            return;
        }

        let mut lines = self.lines.iter();
        let Some(mut line) = lines.next() else {
            return;
        };
        let mut chars = line.text.chars();

        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(char) = chars.next() {
                    if let Some(style) = self.style {
                        buf.get_mut(x, y).unwrap().set_char(char).set_style(style);
                    } else {
                        buf.get_mut(x, y)
                            .unwrap()
                            .set_char(char)
                            .set_style(line.style);
                    }
                } else {
                    if let Some(l) = lines.next() {
                        line = l;
                        chars = line.text.chars();
                    } else {
                        return;
                    }
                    break;
                }
            }
        }
    }
    pub fn height(&self) -> usize {
        self.lines.len()
    }
}

#[macro_export]
macro_rules! lines {
    ($lines:expr) => {
        Lines {
            lines: $lines,
            block: None,
            style: None,
        }
    };
    ($lines:expr, $block:expr) => {
        Lines {
            lines: $lines,
            block: Some($block),
            style: None,
        }
    };
    ($lines:expr, $block:expr, $style:expr) => {
        Lines {
            lines: $lines,
            block: Some($block),
            style: Some($style),
        }
    };
}

#[derive(Debug, Clone)]
pub struct Text<'a> {
    pub text: Cow<'a, str>,
    pub style: Style,
}

impl<'a> Text<'a> {
    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        let mut chars = self.text.chars();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if let Some(char) = chars.next() {
                    buf.get_mut(x, y)
                        .unwrap()
                        .set_char(char)
                        .set_style(self.style);
                } else {
                    return;
                }
            }
        }
    }
    pub fn width(&self) -> usize {
        self.text.width()
    }
}

///Keep in mind wide characters must be formatted with spaces. FIXME: This is not needed in block titles.
/// `う ず ま き` instead of `うずまき`
#[macro_export]
macro_rules! text {
    ($text:expr) => {
        Text {
            text: std::borrow::Cow::from($text),
            style: Style::default(),
        }
    };
    ($text:expr, $style:expr) => {
        Text {
            text: std::borrow::Cow::from($text),
            style: $style,
        }
    };
}
