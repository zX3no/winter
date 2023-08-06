use crate::{buffer::Buffer, *};

#[macro_export]
///It's just a row with a 1px bottom margin.
/// ```rs
/// let header = header![
///         text!(),
///         text!("#", bold()),
///         text!("Title", bold()),
///         text!("Album", bold()),
///         text!("Artist", bold())
///     ];
/// ```
macro_rules! header {
    ($($column:expr),*)  => {
        Row {
            columns: Box::new([$($column.into()),*]),
            height: 1,
            bottom_margin: 1,
        }
    };
}

#[macro_export]
///Create a row with all it's columns.
///```rs
///let rows = &[
///    //Row 1
///    row![
///        //Row 1 Column 1
///        lines_s!(
///            text,
///            // "first item first row",
///            fg(Cyan),
///            " <-- there is a space here",
///            fg(Blue).underlined()
///        ),
///        //Row 1 Column 2
///        lines!("second item", " first row")
///    ],
///    //Row 2
///    row![
///        //Row 2 Column 1
///        lines_s!("first item second row", fg(Yellow)),
///        //Row 2 Column 2
///        lines!("second item second row")
///    ],
///];
///```
macro_rules! row {
    ($($column:expr),*) => {
        Row {
            columns: Box::new([$($column.into()),*]),
            height: 1,
            bottom_margin: 0,
        }
    };
}

#[derive(Debug, Clone)]
pub struct Row<'a> {
    pub columns: Box<[Lines<'a>]>,
    //TODO: Why is exist?
    pub height: u16,
    pub bottom_margin: u16,
}

impl<'a> Row<'a> {
    fn total_height(&self) -> u16 {
        self.height.saturating_add(self.bottom_margin)
    }
}

pub fn table<'a, B: Into<Box<[Row<'a>]>>>(
    header: Option<Row<'a>>,
    block: Option<Block<'a>>,
    widths: &'a [Constraint],
    rows: B,
    highlight_symbol: Option<&'a str>,
    highlight_style: Style,
) -> Table<'a> {
    Table {
        header,
        block,
        widths,
        rows: rows.into(),
        highlight_symbol,
        highlight_style,
        separator: false,
        column_spacing: 0,
    }
}

#[derive(Debug, Clone)]
pub struct Table<'a> {
    pub header: Option<Row<'a>>,
    pub block: Option<Block<'a>>,
    pub widths: &'a [Constraint],
    pub rows: Box<[Row<'a>]>,
    pub highlight_symbol: Option<&'a str>,
    //TODO: REMOVE ME and replace with highlight_line or something?
    pub highlight_style: Style,
    //Puts a line underneath the table header.
    pub separator: bool,
    //Moves the columns apart.
    pub column_spacing: u16,
}

impl<'a> Table<'a> {
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.column_spacing = spacing;
        self
    }
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
        let mut chunks = layout!(
            Rect {
                x: 0,
                y: 0,
                width: max_width,
                height: 1,
            },
            Direction::Horizontal,
            constraints
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

        for item in self.rows.iter() {
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

    pub fn draw(&self, area: Rect, buf: &mut Buffer, state: Option<usize>) {
        if area.area() == 0 {
            return;
        }
        let table_area = if let Some(block) = &self.block {
            block.draw(area, buf);
            block.inner(area)
        } else {
            area
        };

        let has_selection = state.is_some();
        let columns_widths = self.get_columns_widths(table_area.width, has_selection);
        let highlight_symbol = self.highlight_symbol.unwrap_or("");
        let blank_symbol = " ".repeat(highlight_symbol.len());
        let mut current_height = 0;
        let mut rows_height = table_area.height;

        // Draw header
        if let Some(ref header) = self.header {
            let max_header_height = table_area.height.min(header.total_height());
            let mut col = table_area.left();
            if has_selection {
                col += (highlight_symbol.len() as u16).min(table_area.width);
            }
            for (width, column) in columns_widths.iter().zip(header.columns.iter()) {
                if let Some(style) = column.style {
                    buf.set_style(
                        Rect {
                            x: col,
                            y: table_area.top(),
                            width: *width,
                            height: max_header_height,
                        },
                        style,
                    );
                }
                buf.set_lines(col, table_area.top(), column, *width, false);
                col += *width + self.column_spacing;
            }
            if self.separator {
                let max: u16 = columns_widths.iter().sum();
                for i in table_area.left() + 3..max + table_area.left() + 4 {
                    buf.set_lines(i, table_area.top() + 1, &lines!("─"), 1, false);
                }
            }
            current_height += max_header_height;
            rows_height = rows_height.saturating_sub(max_header_height);
        }

        // Draw rows
        if self.rows.is_empty() {
            return;
        }
        let (start, end) = self.get_row_bounds(state, rows_height);

        //TODO: Fix the table moving to the left when selected.
        //I don't think list has this problem?
        for (i, row) in self.rows.iter().enumerate().skip(start).take(end - start) {
            let (x, y) = (table_area.left(), table_area.top() + current_height);
            current_height += row.total_height();
            let selected = state.map_or(false, |s| s == i);

            let mut x = if has_selection {
                let symbol = if selected {
                    highlight_symbol
                } else {
                    &blank_symbol
                };

                //TODO: Maybe replace style() with symbol_style or something?
                let (col, _) = buf.set_stringn(x, y, symbol, table_area.width as usize, style());
                col
            } else {
                x
            };

            for (width, column) in columns_widths.iter().zip(row.columns.iter()) {
                if let Some(style) = column.style {
                    buf.set_style(
                        Rect {
                            x,
                            y,
                            width: *width,
                            height: row.height,
                        },
                        style,
                    );
                }
                buf.set_lines(x, y, column, *width, false);
                x += width + self.column_spacing;
            }
        }
    }
}
