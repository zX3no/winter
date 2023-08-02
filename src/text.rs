use crate::{block::Block, buffer::Buffer, layout::Rect, *};
use std::ops::{Deref, DerefMut};
use unicode_width::UnicodeWidthStr;

//TODO: Split on \n so that each line has it's own item in the array.
//Otherwise Lines::height() will not work correctly.
//I think this is what graphemes are used for.

//TODO: Do we ever actually use lines.style?
//None of the macros use it. So I guess it's just None all the time.

///Keep in mind wide characters must be formatted with spaces. FIXME: This is not needed in block titles.
/// `う ず ま き` instead of `うずまき`
#[derive(Debug, Clone, Default)]
pub struct Lines<'a> {
    pub lines: Box<[Text<'a>]>,
    pub block: Option<Block<'a>>,
    pub style: Option<Style>,
    pub alignment: Alignment,
}

impl<'a> Lines<'a> {
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        let area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        let text_width = self.lines.iter().map(|text| text.width()).sum::<usize>() as u16;
        let x_offset = match self.alignment {
            Alignment::Center => area.width.saturating_sub(text_width) / 2,
            Alignment::Right => area.width.saturating_sub(text_width),
            Alignment::Left => 0,
        };

        buf.set_lines(area.x + x_offset, area.y, self, area.width);
    }
    pub fn draw_wrapping(&self, area: Rect, buf: &mut Buffer) {
        let mut lines = self.lines.iter();
        let Some(mut line) = lines.next() else {
            return;
        };
        //Don't ask.
        let mut chars = line.inner.chars();

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
                        chars = line.inner.chars();
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
    type Target = Box<[Text<'a>]>;

    fn deref(&self) -> &Self::Target {
        &self.lines
    }
}

//TODO: Rework lines macros. They don't support blocks.
pub fn lines<'a, B: Into<Box<[Text<'a>]>>>(
    lines: B,
    block: Option<Block<'a>>,
    style: Option<Style>,
) -> Lines<'a> {
    Lines {
        lines: lines.into(),
        block,
        style,
        alignment: Alignment::Left,
    }
}

/// Use `lines_s` for styled text.
///```rs
/// lines!["Text", "Text", "Text", "Text"]
/// ```
#[macro_export]
macro_rules! lines {
    () => {
        Lines::default()
    };
    ($($text:expr),*) => {
        Lines {
            lines: Box::new([
                $(
                    $crate::Text {
                        inner: std::borrow::Cow::from($text),
                        style: Style::default(),
                    }
                ),*
            ]),
            block: None,
            style: None,
            alignment: Alignment::Left,
        }
    };
}

/// Use `lines` for un-styled text.
///```rs
/// lines_s!["Text", fg(Blue), "Text", fg(Red)]
/// ```
#[macro_export]
macro_rules! lines_s{
    ($($text:expr, $style:expr),*) => {
        Lines {
            lines: Box::new([
                $(
                    $crate::Text {
                        inner: std::borrow::Cow::from($text),
                        style: $style,
                    }
                ),*
            ]),
            block: None,
            style: None,
            alignment: Alignment::Left,
        }
    };
}

//TODO: Things like this `text!(format!("{pct}%"))` seem dumb.
//Maybe add format args or something.
#[macro_export]
macro_rules! text {
    () => {
        Text::default()
    };
    ($text:expr) => {
        Text {
            inner: $text.into(),
            style: Style::default(),
        }
    };
    ($text:expr, $style:expr) => {
        Text {
            inner: $text.into(),
            style: $style,
        }
    };
}

#[derive(Debug, Clone, Default)]
pub struct Text<'a> {
    pub inner: std::borrow::Cow<'a, str>,
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

macro_rules! impl_lines {
    ($($t:ty),*) => {
        $(
            impl<'a> Into<Lines<'a>> for $t {
                fn into(self) -> Lines<'a> {
                    Lines {
                        lines: Box::new([self.into()]),
                        block: None,
                        style: None,
                        alignment: Alignment::Left,
                    }
                }
            }
        )*
    };
}

macro_rules! impl_text {
    ($($t:ty),*) => {
        impl_lines!($($t),*);
        $(
            impl<'a> Into<Text<'a>> for $t {
                fn into(self) -> Text<'a> {
                    Text {
                        inner: std::borrow::Cow::from(self),
                        style: Style::default(),
                    }
                }
            }
        )*
    };
}

impl_lines! { Text<'a> }
impl_text! { String, std::borrow::Cow<'a, str>, &'static str }
