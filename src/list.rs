use crate::{buffer::Buffer, layout::Rect, *};

#[derive(Debug, Clone, Default)]
pub struct ListState {
    selection: usize,
    selected: bool,
}

impl ListState {
    pub fn select(&mut self, index: Option<usize>) {
        if let Some(i) = index {
            self.selection = i;
            self.selected = true;
        } else {
            self.selected = false;
        }
    }
}

pub fn list<'a>(
    block: Option<Block<'a>>,
    items: Lines<'a>,

    highlight_symbol: Option<&'a str>,
    highlight_style: Style,
) -> List<'a> {
    List {
        block,
        items,
        highlight_symbol,
        highlight_style,
        start_from_bottom: false,
    }
}

pub fn list_state(index: Option<usize>) -> ListState {
    if let Some(i) = index {
        ListState {
            selection: i,
            selected: true,
        }
    } else {
        ListState {
            selection: 0,
            selected: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct List<'a> {
    block: Option<Block<'a>>,
    items: Lines<'a>,
    highlight_symbol: Option<&'a str>,
    highlight_style: Style,
    start_from_bottom: bool,
}

impl<'a> List<'a> {
    fn get_items_bounds(&self, selection: usize, terminal_height: usize) -> (usize, usize) {
        let mut real_end = 0;
        let mut height = 0;
        //Was `item.height()`
        let item_height = 1;
        for _item in self.items.lines {
            if height + item_height > terminal_height {
                break;
            }
            height += item_height;
            real_end += 1;
        }

        let selection = selection.min(self.items.lines.len() - 1);

        let half = if height == 0 { 0 } else { (height - 1) / 2 };

        let start = selection.saturating_sub(half);

        let end = if selection <= half {
            real_end
        } else if height % 2 == 0 {
            selection + 2 + half
        } else {
            selection + 1 + half
        };

        if end > self.items.lines.len() {
            (self.items.lines.len() - height, self.items.lines.len())
        } else {
            (start, end)
        }
    }
    pub fn draw(&self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        let list_area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        if list_area.width < 1 || list_area.height < 1 || self.items.lines.is_empty() {
            return;
        }

        let list_height = list_area.height as usize;

        let (start, end) = self.get_items_bounds(state.selection, list_height);

        let highlight_symbol = self.highlight_symbol.unwrap_or("");
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

            let is_selected = if state.selected && state.selection == i {
                true
            } else {
                false
            };

            let symbol = if is_selected {
                highlight_symbol
            } else {
                &blank_symbol
            };

            //Set the selection symbol.
            let (elem_x, max_element_width) = if state.selected {
                let (elem_x, _) =
                    buf.set_stringn(x, y, symbol, list_area.width as usize, item.style);
                (elem_x, (list_area.width - (elem_x - x)))
            } else {
                (x, list_area.width)
            };

            //Set the item text.
            buf.set_stringn(
                elem_x,
                y + 0 as u16,
                item,
                max_element_width as usize,
                item.style,
            );

            //TODO: Maybe skip the symbol area and just style the list item?
            //Could have a symbol_style and a selection_style?
            if is_selected {
                buf.set_style(area, self.highlight_style);
            }
        }
    }
}
