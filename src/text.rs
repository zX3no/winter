use crate::{block::Block, buffer::Buffer, layout::Rect, *};
use std::ops::Deref;
use unicode_width::UnicodeWidthStr;

//TODO: Split on \n so that each line has it's own item in the array.
//Otherwise Lines::height() will not work correctly.
//I think this is what graphemes are used for.

//TODO: Figure out style inheritence. Do we want it?

///Keep in mind wide characters must be formatted with spaces. FIXME: This is not needed in block titles.
/// `う ず ま き` instead of `うずまき`
#[derive(Debug, Clone, Default)]
pub struct Lines<'a> {
    pub lines: Box<[Text<'a>]>,
    pub block: Option<Block<'a>>,
    pub style: Option<Style>,
    pub alignment: Alignment,
    pub scroll: bool,
}

impl<'a> Lines<'a> {
    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }
    pub fn align(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    pub fn scroll(mut self) -> Self {
        self.scroll = true;
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
        let alignment = match self.alignment {
            Alignment::Center => area.width.saturating_sub(text_width) / 2,
            Alignment::Right => area.width.saturating_sub(text_width),
            Alignment::Left => 0,
        };

        buf.set_lines(area.x + alignment, area.y, self, area.width, self.scroll);
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

impl<'a> Into<Lines<'a>> for Text<'a> {
    fn into(self) -> Lines<'a> {
        Lines {
            style: Some(self.style),
            lines: Box::new([self]),
            block: None,
            alignment: Alignment::Left,
            scroll: false,
        }
    }
}

impl<'a> Into<Lines<'a>> for &'a [Text<'a>] {
    fn into(self) -> Lines<'a> {
        Lines {
            lines: self.into(),
            block: None,
            style: None,
            alignment: Alignment::Left,
            scroll: false,
        }
    }
}

/// Can take in anything that implements Into<Text<'a>>.
#[macro_export]
macro_rules! lines {
    () => {
        Lines::default()
    };
    ($($text:expr),*) => {
        Lines {
            lines: Box::new([
                $(
                    $text.into()
                ),*
            ]),
            block: None,
            style: None,
            alignment: Alignment::Left,
            scroll: false,
        }
    };
}

#[macro_export]
macro_rules! text {
    () => {
        //TODO: This is not const!
        Text::default()
    };
    ($($args:tt)*) => {
        Text {
            inner: format_args!($($args)*).to_string().into(),
            style: Style::default(),
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
    pub fn into_lines(self) -> Lines<'a> {
        self.into()
    }
}

// impl<'a> Into<&'a [Text<'a>]> for &'a [&'a str] {
//     fn into(self) -> &'a [Text<'a>] {
//         todo!()
//     }
// }
trait IntoText<'a> {
    fn into_text(self) -> Vec<Text<'a>>;
}

impl<'a> IntoText<'a> for &'a [&'a str] {
    fn into_text(self) -> Vec<Text<'a>> {
        self.iter()
            .map(|s| Text {
                inner: std::borrow::Cow::Borrowed(*s),
                style: Style::default(), // You might need to adjust this depending on your use case
            })
            .collect()
    }
}

impl<'a> AsRef<str> for Text<'a> {
    fn as_ref(&self) -> &str {
        &self.inner as &str
    }
}

macro_rules! modifier {
    ($($type:ty),*; $($name:ident),*) => {
        macro_rules! modifier {
            () => {
                $(fn $name(self) -> Text<'a> {
                    Text {
                        inner: std::borrow::Cow::from(self),
                        style: $name(),
                    }
                })*
            };
        }

        macro_rules! modifier_text {
            () => {
                $(fn $name(mut self) -> Text<'a> {
                    //TODO: Maybe cleanup this.
                    self.style.modifier.insert($name().modifier);
                    self
                })*
            };
        }

        macro_rules! style {
            () => {
                fn fg(self, fg: Color) -> Text<'a> {
                    let mut text = Into::<Text>::into(self);
                    text.style.fg = fg;
                    text
                }
                fn bg(self, bg: Color) -> Text<'a> {
                    let mut text = Into::<Text>::into(self);
                    text.style.bg = bg;
                    text
                }
                fn style(self, style: Style) -> Text<'a> {
                    let mut text = Into::<Text>::into(self);
                    text.style = style;
                    text
                }
            }
        }

        impl<'a> Stylize<'a> for Text<'a> {
            style!();
            modifier_text!();
        }

        $(
            impl<'a> Stylize<'a> for $type {
                style!();
                modifier!();
            }
        )*
    };
}

pub trait Stylize<'a> {
    fn bold(self) -> Text<'a>;
    fn dim(self) -> Text<'a>;
    fn italic(self) -> Text<'a>;
    fn underlined(self) -> Text<'a>;
    fn fast_blink(self) -> Text<'a>;
    fn slow_blink(self) -> Text<'a>;
    fn invert(self) -> Text<'a>;
    fn hidden(self) -> Text<'a>;
    fn crossed_out(self) -> Text<'a>;
    fn fg(self, fg: Color) -> Text<'a>;
    fn bg(self, bg: Color) -> Text<'a>;
    fn style(self, style: Style) -> Text<'a>;
}

modifier! {
    String,
    &'a String,
    &'a str;
    bold,
    dim,
    italic,
    underlined,
    fast_blink,
    slow_blink,
    invert,
    hidden,
    crossed_out
}

macro_rules! impl_into {
    ($($t:ty),*) => {
        $(
            impl<'a> Into<Text<'a>> for $t {
                fn into(self) -> Text<'a> {
                    Text {
                        inner: std::borrow::Cow::from(self),
                        style: Style::default(),
                    }
                }
            }

            impl<'a> Into<Lines<'a>> for $t {
                fn into(self) -> Lines<'a> {
                    Lines {
                        lines: Box::new([self.into()]),
                        block: None,
                        style: None,
                        alignment: Alignment::Left,
                        scroll: false,
                    }
                }
            }
        )*
    };
}

impl_into! { String, std::borrow::Cow<'a, str>, &'a str, &'a String }
