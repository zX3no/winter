use crate::{buffer::Buffer, layout::Rect, Style};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Lines<'a> {
    pub lines: &'a [Text<'a>],
    pub style: Option<Style>,
}

impl<'a> Lines<'a> {
    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        let mut lines_iter = self.lines.iter();
        for y in area.top()..area.bottom() {
            if let Some(line) = lines_iter.next() {
                let mut chars = line.text.chars();
                for x in area.left()..area.right() {
                    if let Some(char) = chars.next() {
                        if let Some(style) = self.style {
                            buf.get_mut(x, y).set_char(char).set_style(style);
                        } else {
                            buf.get_mut(x, y).set_char(char).set_style(line.style);
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
                        buf.get_mut(x, y).set_char(char).set_style(style);
                    } else {
                        buf.get_mut(x, y).set_char(char).set_style(line.style);
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
}

pub fn lines<'a>(lines: &'a [Text<'a>], style: Option<Style>) -> Lines<'a> {
    Lines { lines, style }
}

#[macro_export]
macro_rules! lines {
    ($lines:expr) => {
        Lines {
            lines: $lines,
            style: None,
        }
    };
    ($Lines:expr, $style:expr) => {
        Lines {
            lines,
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
                    buf.get_mut(x, y).set_char(char).set_style(self.style);
                } else {
                    return;
                }
            }
        }
    }
}

pub fn text<'a>(text: &'a str, style: Style) -> Text<'a> {
    Text {
        text: Cow::from(text),
        style,
    }
}

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
