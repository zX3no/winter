use crate::{buffer::Buffer, layout, lines, Block, Constraint, Direction, Lines, Rect, Style};
use std::borrow::Cow;

pub fn table_state(index: Option<usize>) -> TableState {
    TableState { selected: index }
}

#[derive(Debug, Clone)]
pub struct TableState {
    pub selected: Option<usize>,
}

pub fn row<'a>(cells: Vec<Lines<'a>>, style: Style, bottom_margin: u16) -> Row<'a> {
    Row {
        cells,
        //TODO: Why is exist?
        height: 1,
        style,
        bottom_margin,
    }
}

#[derive(Debug, Clone)]
pub struct Row<'a> {
    //Originally this was `cells: Vec<Cell<'a>>`
    //Which is Vec<Text> -> Vec<Vec<Spans>> -> Vec<VecVec<<Span>>>
    pub cells: Vec<Lines<'a>>,
    pub height: u16,
    pub style: Style,
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

    highlight_symbol: Option<&'a str>,
    rows: &'a [Row<'a>],

    style: Style,
    highlight_style: Style,
) -> Table<'a> {
    Table {
        block,
        style,
        widths,
        column_spacing: 1,
        highlight_style,
        highlight_symbol,
        header,
        rows,
        separator: false,
    }
}

#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub block: Option<Block<'a>>,
    pub style: Style,
    pub widths: &'a [Constraint],
    pub column_spacing: u16,
    pub highlight_style: Style,
    pub highlight_symbol: Option<&'a str>,
    pub header: Option<Row<'a>>,
    pub rows: &'a [Row<'a>],
    pub separator: bool,
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
        buf.set_style(area, self.style);
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

        //TODO: Add an underline
        // Draw header
        if let Some(ref header) = self.header {
            let max_header_height = table_area.height.min(header.total_height());
            buf.set_style(
                Rect {
                    x: table_area.left(),
                    y: table_area.top(),
                    width: table_area.width,
                    height: table_area.height.min(header.height),
                },
                header.style,
            );
            let mut col = table_area.left();
            if has_selection {
                col += (highlight_symbol.len() as u16).min(table_area.width);
            }
            for (width, cell) in columns_widths.iter().zip(header.cells.iter()) {
                draw_cell(
                    buf,
                    cell,
                    Rect {
                        x: col,
                        y: table_area.top(),
                        width: *width,
                        height: max_header_height,
                    },
                );

                col += *width + self.column_spacing;
            }
            if self.separator {
                let max: u16 = columns_widths.iter().sum();
                for i in table_area.left() + 3..max + table_area.left() + 4 {
                    draw_cell(
                        buf,
                        &lines!("─"),
                        Rect {
                            x: i,
                            y: table_area.top() + 1,
                            width: 1,
                            height: 1,
                        },
                    );
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
        for (i, table_row) in self.rows.iter().enumerate().skip(start).take(end - start) {
            let (row, col) = (table_area.top() + current_height, table_area.left());
            current_height += table_row.total_height();
            let table_row_area = Rect {
                x: col,
                y: row,
                width: table_area.width,
                height: table_row.height,
            };
            buf.set_style(table_row_area, table_row.style);
            let is_selected = state.selected.map_or(false, |s| s == i);
            let table_row_start_col = if has_selection {
                let symbol = if is_selected {
                    highlight_symbol
                } else {
                    &blank_symbol
                };
                let (col, _) =
                    buf.set_stringn(col, row, symbol, table_area.width as usize, table_row.style);
                col
            } else {
                col
            };
            let mut col = table_row_start_col;
            for (width, cell) in columns_widths.iter().zip(table_row.cells.iter()) {
                draw_cell(
                    buf,
                    cell,
                    Rect {
                        x: col,
                        y: row,
                        width: *width,
                        height: table_row.height,
                    },
                );
                col += *width + self.column_spacing;
            }
            if is_selected {
                buf.set_style(table_row_area, self.highlight_style);
            }
        }
    }
}

fn draw_cell(buf: &mut Buffer, cell: &Lines, area: Rect) {
    if let Some(style) = cell.style {
        buf.set_style(area, style);
    }

    for (i, line) in cell.lines.iter().enumerate() {
        if i as u16 >= area.height {
            break;
        }
        buf.set_stringn(
            area.x,
            area.y + i as u16,
            line,
            area.width as usize,
            line.style,
        );
    }
}
