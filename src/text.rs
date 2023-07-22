use crate::{buffer::Buffer, layout::Rect, Style};
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct Lines<'a> {
    pub lines: &'a [Text<'a>],
    pub style: Style,
}

impl<'a> Lines<'a> {
    pub fn draw(&self, style: Style, area: Rect, buf: &mut Buffer) {
        let mut lines_iter = self.lines.iter();
        for y in area.top()..area.bottom() {
            if let Some(line) = lines_iter.next() {
                let mut chars = line.text.chars();
                for x in area.left()..area.right() {
                    if let Some(char) = chars.next() {
                        buf.get_mut(x, y).set_char(char).set_style(style);
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }
}

pub fn lines<'a>(lines: &'a [Text<'a>], style: Style) -> Lines<'a> {
    Lines { lines, style }
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
