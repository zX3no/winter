use crate::{
    buffer::Buffer,
    layout::{Corner, Rect},
    Block, Lines, Style,
};

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
    items: &'a [Lines<'a>],

    style: Style,
    start_corner: Corner,

    highlight_style: Style,
    highlight_symbol: Option<&'a str>,
) -> List<'a> {
    List {
        block,
        items,
        style,
        start_corner,
        highlight_style,
        highlight_symbol,
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
    items: &'a [Lines<'a>],

    style: Style,
    start_corner: Corner,

    highlight_style: Style,
    highlight_symbol: Option<&'a str>,
}

impl<'a> List<'a> {
    fn get_items_bounds(&self, selection: usize, terminal_height: usize) -> (usize, usize) {
        let mut real_end = 0;
        let mut height = 0;
        for item in self.items {
            if height + item.height() > terminal_height {
                break;
            }
            height += item.height();
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
    pub fn draw(&self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        buf.set_style(area, self.style);

        let list_area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        if list_area.width < 1 || list_area.height < 1 {
            return;
        }

        if self.items.is_empty() {
            return;
        }
        let list_height = list_area.height as usize;

        let (start, end) = self.get_items_bounds(state.selection, list_height);

        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let blank_symbol = " ".repeat(highlight_symbol.len());
        let mut current_height = 0;

        for (i, item) in self.items.iter().enumerate().skip(start).take(end - start) {
            let (x, y) = if self.start_corner == Corner::BottomLeft {
                current_height += item.height() as u16;
                (list_area.left(), list_area.bottom() - current_height)
            } else {
                let pos = (list_area.left(), list_area.top() + current_height);
                current_height += item.height() as u16;
                pos
            };

            let area = Rect {
                x,
                y,
                width: list_area.width,
                height: item.height() as u16,
            };

            // let item_style = self.style.patch(item.style);
            // buf.set_style(area, item_style);

            //check if the current index is selected
            let is_selected = if state.selected {
                state.selection == i
            } else {
                false
            };

            for (j, line) in item.lines.iter().enumerate() {
                // if the item is selected, we need to display the hightlight symbol:
                // - either for the first line of the item only,
                // - or for each line of the item if the appropriate option is set
                let symbol = if is_selected && (j == 0) {
                    highlight_symbol
                } else {
                    &blank_symbol
                };

                let (elem_x, max_element_width) = if state.selected {
                    let (elem_x, _) = buf.set_stringn(
                        x,
                        y + j as u16,
                        symbol,
                        list_area.width as usize,
                        line.style,
                        //TODO: item_style,
                    );
                    (elem_x, (list_area.width - (elem_x - x)))
                } else {
                    (x, list_area.width)
                };
                // buf.set_spans(elem_x, y + j as u16, line, max_element_width);
                buf.set_stringn(
                    elem_x,
                    y + j as u16,
                    &line.text,
                    max_element_width as usize,
                    line.style,
                );
            }

            //sets the style of the selection
            if state.selected {
                buf.set_style(area, self.highlight_style);
            }
        }
    }
}
