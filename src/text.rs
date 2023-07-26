use crate::{block::Block, buffer::Buffer, layout::Rect, Style};
use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};
use unicode_width::UnicodeWidthStr;

//TODO: Text alignment?
// fn get_line_offset(line_width: u16, text_area_width: u16, alignment: Alignment) -> u16 {
//     match alignment {
//         Alignment::Center => (text_area_width / 2).saturating_sub(line_width / 2),
//         Alignment::Right => text_area_width.saturating_sub(line_width),
//         Alignment::Left => 0,
//     }
// }

//TODO: Split on \n so that each line has it's own item in the array.
//Otherwise Lines::height() will not work correctly.
//I think this is what graphemes are used for.

///Keep in mind wide characters must be formatted with spaces. FIXME: This is not needed in block titles.
/// `う ず ま き` instead of `うずまき`
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
                let mut chars = line.chars();
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
        let mut lines = self.lines.iter();
        let Some(mut line) = lines.next() else {
            return;
        };
        //Don't ask.
        let mut chars = line.chars();

        let area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

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
                    if let Some(new_line) = lines.next() {
                        line = new_line;
                        chars = line.chars();
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

impl<'a> Deref for Lines<'a> {
    type Target = &'a [Text<'a>];

    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}

pub fn lines<'a>(
    lines: &'a [Text<'a>],
    block: Option<Block<'a>>,
    style: Option<Style>,
) -> Lines<'a> {
    Lines {
        lines,
        block,
        style,
    }
}

///```rs
/// lines!["Text", "Text", "Text", "Text",]
/// ```
#[macro_export]
macro_rules! lines {
    ($($text:expr),*) => {
        Lines {
            lines: &[
                $(
                    crate::Text {
                        inner: Cow::from($text),
                        style: Style::default(),
                    }
                ),*
            ],
            block: None,
            style: None,
        }
    };
}

///```rs
/// lines_s!["Text", fg(Blue), "Text", fg(Red)]
/// ```
#[macro_export]
macro_rules! lines_s{
    ($($text:expr, $style:expr),*) => {
        Lines {
            lines: &[
                $(
                    crate::Text {
                        inner: Cow::from($text),
                        style: $style,
                    }
                ),*
            ],
            block: None,
            style: None,
        }
    };
}

#[derive(Debug, Clone)]
pub struct Text<'a> {
    pub inner: Cow<'a, str>,
    pub style: Style,
}

impl<'a> Text<'a> {
    pub fn width(&self) -> usize {
        self.inner.width()
    }
}

impl<'a> AsRef<str> for Text<'a> {
    fn as_ref(&self) -> &str {
        &self.inner as &str
    }
}

impl<'a> Deref for Text<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for Text<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a> Into<Text<'a>> for &'static str {
    fn into(self) -> Text<'a> {
        Text {
            inner: std::borrow::Cow::from(self),
            style: Style::default(),
        }
    }
}

// Dummy function to apply style to a string (or just return the style)
pub fn apply_style_internal<'a>(input: impl Into<StyledText<'a>>) -> StyledText<'a> {
    input.into()
}

// StyledText enum to wrap &str and Style
pub enum StyledText<'a> {
    Text(&'a str, Style),
    SingleStyle(Style),
}
