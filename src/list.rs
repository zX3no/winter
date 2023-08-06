use crate::{buffer::Buffer, layout::Rect, *};

pub fn list<'a, B: Into<Box<[Lines<'a>]>>>(
    block: Option<Block<'a>>,
    items: B,
    selection_symbol: Option<&'a str>,
    selection_style: Option<Style>,
) -> List<'a> {
    List {
        block,
        items: items.into(),
        selection_symbol,
        selection_style,
        start_from_bottom: false,
    }
}

#[derive(Debug, Clone)]
pub struct List<'a> {
    block: Option<Block<'a>>,
    items: Box<[Lines<'a>]>,
    selection_symbol: Option<&'a str>,
    selection_style: Option<Style>,
    start_from_bottom: bool,
}

impl<'a> List<'a> {
    fn get_items_bounds(&self, selection: usize, terminal_height: usize) -> (usize, usize) {
        let mut real_end = 0;
        let mut height = 0;
        //Was `item.height()`
        let item_height = 1;
        for item in self.items.iter() {
            if height + item_height > terminal_height {
                break;
            }
            height += item_height;
            real_end += 1;
        }

        let selection = selection.min(self.items.len() - 1);

        let half = if height == 0 { 0 } else { (height - 1) / 2 };

        let start = selection.saturating_sub(half);

        let end = if selection <= half {
            real_end
        } else if height % 2 == 0 {
            selection + 2 + half
        } else {
            selection + 1 + half
        };

        if end > self.items.len() {
            (self.items.len() - height, self.items.len())
        } else {
            (start, end)
        }
    }
    pub fn draw(&self, area: Rect, buf: &mut Buffer, state: Option<usize>) {
        let list_area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        let selected = state.is_some();
        let selection = state.unwrap_or(0);

        if list_area.width < 1 || list_area.height < 1 || self.items.is_empty() {
            return;
        }

        let list_height = list_area.height as usize;

        let (start, end) = self.get_items_bounds(selection, list_height);

        let highlight_symbol = self.selection_symbol.unwrap_or("");
        let blank_symbol = " ".repeat(highlight_symbol.len());
        let mut current_height = 0;

        for (i, item) in self.items.iter().enumerate().skip(start).take(end - start) {
            //Was `item.height()`
            let height: u16 = 1;

            let (x, y) = if self.start_from_bottom {
                current_height += height;
                (list_area.left(), list_area.bottom() - current_height)
            } else {
                let pos = (list_area.left(), list_area.top() + current_height);
                current_height += height;
                pos
            };

            let area = Rect {
                x,
                y,
                width: list_area.width,
                height,
            };

            let is_selected = selected && selection == i;

            let symbol = if is_selected {
                highlight_symbol
            } else {
                &blank_symbol
            };

            //Set the selection symbol.
            let (elem_x, max_element_width) = if selected {
                //TODO: What about the symbol style?
                let (elem_x, _) = buf.set_stringn(x, y, symbol, list_area.width as usize, style());
                (elem_x, (list_area.width - (elem_x - x)))
            } else {
                (x, list_area.width)
            };

            //Set the item text.
            if let Some(style) = item.style {
                buf.set_style(area, style);
            }
            buf.set_lines(elem_x, y, item, max_element_width, false);

            //TODO: Maybe skip the symbol area and just style the list item?
            //Could have a symbol_style and a selection_style?
            if let Some(style) = self.selection_style {
                if is_selected {
                    buf.set_style(area, style);
                }
            }
        }
    }
}
