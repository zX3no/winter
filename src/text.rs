use crate::{buffer::Buffer, layout::Rect, Style};
use std::borrow::Cow;

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
