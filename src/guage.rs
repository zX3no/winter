use crate::{block::Block, buffer::Buffer, layout::Rect, *};
use std::borrow::Cow;
use unicode_width::UnicodeWidthStr;

pub fn guage<'a>(
    block: Option<Block<'a>>,
    ratio: f64,
    label: Option<Cow<'a, str>>,
    label_style: Style,
    left_style: Style,
    right_style: Style,
) -> Gauge<'a> {
    Gauge {
        block,
        ratio,
        label,
        label_style,
        left_style,
        right_style,
    }
}

#[derive(Debug, Clone)]
pub struct Gauge<'a> {
    pub block: Option<Block<'a>>,
    pub ratio: f64,
    pub label: Option<Cow<'a, str>>,
    pub label_style: Style,
    pub left_style: Style,
    pub right_style: Style,
}

impl<'a> Gauge<'a> {
    pub fn draw(&self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.right_style);

        let area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        if area.height < 1 {
            return;
        }

        // compute label value and its position
        // label is put at the center of the gauge_area
        let pct = f64::round(self.ratio * 100.0);
        let text = Cow::from(format!("{pct}%"));
        let label = self.label.as_ref().unwrap_or_else(|| &text);

        let clamped_label_width = area.width.min(label.width() as u16);
        let label_col = area.left() + (area.width - clamped_label_width) / 2;
        let label_row = area.top() + area.height / 2;

        // the gauge will be filled proportionally to the ratio
        let filled_width = f64::from(area.width) * self.ratio;
        let end = area.left() + filled_width.round() as u16;
        for y in area.top()..area.bottom() {
            // render the filled area (left to end)
            for x in area.left()..end {
                // spaces are needed to apply the background styling
                buf.get_mut(x, y)
                    .unwrap()
                    .set_symbol(" ")
                    .set_fg(self.left_style.fg)
                    .set_bg(self.left_style.bg);
            }
        }
        // set the span
        buf.set_stringn(
            label_col,
            label_row,
            &label,
            clamped_label_width as usize,
            self.label_style,
        );
    }
}
