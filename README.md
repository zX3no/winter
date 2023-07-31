## winter

### Terminal:
- [ ] Raw mode
- [ ] Input support
- [ ] Simplify main loop
- [x] Reset all styles on exit

### Widgets:
- [x] Table
- [x] Constraints
- [x] List
- [x] Guage
- [x] Block
- [x] Text
- [x] Layout

### Lines
- [x] Simplify the way text and lines work.
- [x] Fix styles

### Text
- [ ] Text alignment: Left, Center, Right
- [ ] Correctly handle multi-width characters
- [x] Fix modifiers

### Block
- [x] Offset titles
- [x] Text inside of blocks

### List
- [x] Only index 0 shows selection symbol
- [x] Fix styles

### Table
- [x] Only first line is displayed
- [x] Fix styles

### Layout
- [x] Cleanup

### Gauge
- [x] Fix styles

----

Okay so there's a table made up of rows, columns and a style.
Each row is made up `n_1` columns.
Each columns is made up of `n_2` lines and a style.
Each line is made up of `n_3` text and a style.
Each text item is made up of a string and a style.

Currently text styles overwrite all other styles.
Should you be able to set inherited styles for lines, rows and tables.

```rs
    let rows = &[
        //Row 1
        row![
            &[
                //Row 1 Column 1
                lines_s!(
                    "first item first row",
                    fg(Cyan),
                    " <-- there is a space here",
                    fg(Blue).underlined()
                ),
                //Row 1 Column 2
                lines!(
                    "second item",
                    " first row"
                ),
            ],
            style() //FIXME: Style is not being applied here.
        ],
        //Row 2
        row![
            &[
                //Row 2 Column 1
                lines_s!("first item second row", fg(Yellow)),
                //Row 2 Column 2
                lines!("second item second row"),
            ],
            fg(Yellow) //FIXME: Style is not being applied here.
        ],
    ];
```