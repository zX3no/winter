use crate::{buffer::Buffer, *};
use std::borrow::Cow;

pub fn table_state(index: Option<usize>) -> TableState {
    TableState { selected: index }
}

#[derive(Debug, Clone)]
pub struct TableState {
    pub selected: Option<usize>,
}

#[macro_export]
macro_rules! row {
    ($columns:expr) => {
        row!($row, None, 0)
    };
    ($columns:expr, $style:expr) => {
        row!($columns, $style, 0)
    };
    ($columns:expr, $style:expr, $bottom_margin:expr) => {
        Row {
            columns: $columns,
            //TODO: Why is exist?
            height: 1,
            style: Some($style),
            bottom_margin: $bottom_margin,
        }
    };
}

#[derive(Debug, Clone)]
pub struct Row<'a> {
    //Originally this was `cells: Vec<Cell<'a>>`
    //Which is Vec<Text> -> Vec<Vec<Spans>> -> Vec<VecVec<<Span>>>
    pub columns: &'a [Lines<'a>],
    pub style: Option<Style>,
    pub height: u16,
    pub bottom_margin: u16,
}

impl<'a> Row<'a> {
    fn total_height(&self) -> u16 {
        self.height.saturating_add(self.bottom_margin)
    }
}

pub fn table_temp<'a>(rows: &'a [Row<'a>]) -> Table {
    Table {
        block: None,
        style: Style::default(),
        widths: &[],
        column_spacing: 1,
        highlight_style: Style::default(),
        highlight_symbol: None,
        header: None,
        rows,
        separator: false,
    }
}

pub fn table<'a>(
    header: Option<Row<'a>>,
    block: Option<Block<'a>>,
    widths: &'a [Constraint],

    rows: &'a [Row<'a>],
    style: Style,

    highlight_symbol: Option<&'a str>,
    //TODO: REMOVE ME
    highlight_style: Style,
) -> Table<'a> {
    Table {
        header,
        block,
        widths,
        rows,
        style,
        highlight_symbol,
        highlight_style,
        separator: false,
        //TODO: Why is this 1 and not 0?
        column_spacing: 1,
    }
}

#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub header: Option<Row<'a>>,
    pub block: Option<Block<'a>>,
    pub widths: &'a [Constraint],

    pub rows: &'a [Row<'a>],
    pub style: Style,

    pub highlight_symbol: Option<&'a str>,
    pub highlight_style: Style,

    //Puts a line underneath the table header.
    pub separator: bool,
    pub column_spacing: u16,
}

impl<'a> Table<'a> {
    fn get_columns_widths(&self, max_width: u16, has_selection: bool) -> Vec<u16> {
        let mut constraints = Vec::with_capacity(self.widths.len() * 2 + 1);
        if has_selection {
            let highlight_symbol_width = self.highlight_symbol.map_or(0, |s| s.len() as u16);
            constraints.push(Constraint::Length(highlight_symbol_width));
        }
        for constraint in self.widths {
            constraints.push(*constraint);
            constraints.push(Constraint::Length(self.column_spacing));
        }
        if !self.widths.is_empty() {
            constraints.pop();
        }
        let mut chunks = layout(
            Direction::Horizontal,
            (0, 0),
            constraints,
            Rect {
                x: 0,
                y: 0,
                width: max_width,
                height: 1,
            },
        );
        if has_selection {
            chunks.remove(0);
        }
        chunks.iter().step_by(2).map(|c| c.width).collect()
    }

    pub fn get_row_bounds(&self, selected: Option<usize>, terminal_height: u16) -> (usize, usize) {
        let mut real_end = 0;
        let mut height: usize = 0;
        let len = self.rows.len();
        let selection = selected.unwrap_or(0).min(len.saturating_sub(1));

        for item in self.rows {
            if height + item.height as usize > terminal_height as usize {
                break;
            }
            height += item.height as usize;
            real_end += 1;
        }

        if len <= height {
            (len - height, len)
        } else if height > 0 {
            let half = (height - 1) / 2;

            let (start, end) = if selection <= half {
                (0, real_end)
            } else if height % 2 == 0 {
                (selection - half, (selection + 2) + half)
            } else {
                (selection - half, (selection + 1) + half)
            };

            if end > len {
                (len - height, len)
            } else {
                (start, end)
            }
        } else {
            (0, 0)
        }
    }

    ///Used for finding which row to click on.
    /// ```rs
    ///let row_bounds = Some(t.get_row_bounds(ui_index, t.get_row_height(chunks[1])));
    ///
    /////Mouse support for the queue.
    ///if let Some((start, _)) = row_bounds {
    ///    //Check if you clicked on the header.
    ///    if y >= header_height {
    ///        let index = (y - header_height) as usize + start;

    ///        //Make sure you didn't click on the seek bar
    ///        //and that the song index exists.
    ///        if index < player.songs.len()
    ///            && ((size.height < 15 && y < size.height.saturating_sub(1))
    ///                || y < size.height.saturating_sub(3))
    ///        {
    ///            queue.ui.select(Some(index));
    ///        }
    ///    }
    ///}
    /// ```
    pub fn get_row_height(&self, area: Rect) -> u16 {
        let table_area = match &self.block {
            Some(b) => b.inner(area),
            None => area,
        };
        let mut rows_height = table_area.height;
        if let Some(ref header) = self.header {
            let max_header_height = table_area.height.min(header.total_height());
            rows_height = rows_height.saturating_sub(max_header_height);
        }
        rows_height
    }

    pub fn draw(&self, area: Rect, buf: &mut Buffer, state: &mut TableState) {
        if area.area() == 0 {
            return;
        }
        // buf.set_style(area, self.style);
        let table_area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        let has_selection = state.selected.is_some();
        let columns_widths = self.get_columns_widths(table_area.width, has_selection);
        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let blank_symbol = " ".repeat(highlight_symbol.len());
        let mut current_height = 0;
        let mut rows_height = table_area.height;

        // Draw header
        if let Some(ref header) = self.header {
            let max_header_height = table_area.height.min(header.total_height());
            let style = header.style.unwrap_or_default();
            buf.set_style(
                Rect {
                    x: table_area.left(),
                    y: table_area.top(),
                    width: table_area.width,
                    height: table_area.height.min(header.height),
                },
                style,
            );
            let mut col = table_area.left();
            if has_selection {
                col += (highlight_symbol.len() as u16).min(table_area.width);
            }
            for (width, cell) in columns_widths.iter().zip(header.columns.iter()) {
                if let Some(style) = cell.style {
                    buf.set_style(area, style);
                }
                buf.set_lines(col, table_area.top(), &cell, *width);
                col += *width + self.column_spacing;
            }
            if self.separator {
                let max: u16 = columns_widths.iter().sum();
                for i in table_area.left() + 3..max + table_area.left() + 4 {
                    let row = lines!("─");
                    if let Some(style) = row.style {
                        buf.set_style(area, style);
                    }
                    buf.set_lines(i, table_area.top() + 1, &row, 1);
                }
            }
            current_height += max_header_height;
            rows_height = rows_height.saturating_sub(max_header_height);
        }

        // Draw rows
        if self.rows.is_empty() {
            return;
        }
        let (start, end) = self.get_row_bounds(state.selected, rows_height);

        //TODO: Fix the table moving to the left when selected.
        //I don't think list has this problem?
        for (i, row) in self.rows.iter().enumerate().skip(start).take(end - start) {
            let (x, y) = (table_area.left(), table_area.top() + current_height);
            current_height += row.total_height();

            // let table_row_area = Rect {
            //     x,
            //     y,
            //     width: table_area.width,
            //     height: row.height,
            // };
            // buf.set_style(table_row_area, row.style);
            let row_style = row.style.unwrap_or_default();

            let selected = state.selected.map_or(false, |s| s == i);

            let mut x = if has_selection {
                let symbol = if selected {
                    highlight_symbol
                } else {
                    &blank_symbol
                };
                //TODO: Maybe replace with symbol_style or something?
                let (col, _) = buf.set_stringn(x, y, symbol, table_area.width as usize, row_style);
                col
            } else {
                x
            };

            for (width, column) in columns_widths.iter().zip(row.columns.iter()) {
                let area = Rect {
                    x,
                    y,
                    width: *width,
                    height: row.height,
                };

                // if let Some(style) = lines.style {
                // row_style = row_style.patch(style);
                // }

                // if let Some(style) = sub_row.style {
                //     if selected {
                //         buf.set_style(area, style.patch(self.highlight_style));
                //     } else {
                //         buf.set_style(area, style);
                //     }
                // } else if selected {
                //     buf.set_style(area, self.highlight_style)
                // };
                // buf.set_style(area, row_style);

                //FIXME: Set lines resets all the styles.
                //This is because in tui-rs fg and bg are both optional.
                //Should we change back or remove row styles?
                buf.set_lines(x, y, column, *width);
                x += width + self.column_spacing;
            }

            //TODO: This shold be removed.
            //The queue in gonk has a fake selection row so
            //it can have styles on the different parts of text.
            //An overall style like this is just not very useful.
            //If this is default it will undo the line style.

            // if is_selected {
            //     buf.set_style(table_row_area, self.highlight_style);
            // }
        }
    }
}
