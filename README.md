## winter

### Terminal:
- [x] Raw mode
- [x] Input support
- [x] Simplify main loop
- [ ] Simplify main loop even more
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

### Example

```rs
pub fn browser(area: Rect, buf: &mut Buffer) {
    let size = area.width / 3;
    let rem = area.width % 3;

    let chunks = layout!(
        area,
        Direction::Horizontal,
        Constraint::Length(size),
        Constraint::Length(size),
        Constraint::Length(size + rem)
    );

    let a = lines!["Artist 1", "Artist 2", "Artist 3"];
    let b = lines!["Album 1", "Album 2", "Album 3"];
    let c = lines!["Song 1", "Song 2", "Song 3"];

    fn browser_list<'a>(title: &'static str, content: Lines<'a>, use_symbol: bool) -> List<'a> {
        let block = block(
            Some(text!(title, bold())),
            1,
            Borders::ALL,
            BorderType::Rounded,
            style(),
        );
        let symbol = if use_symbol { ">" } else { " " };
        list(Some(block), content, Some(symbol), style())
    }

    let artists = browser_list("Aritst", a, false);
    let albums = browser_list("Album", b, false);
    let songs = browser_list("Song", c, true);

    artists.draw(chunks[0], buf, &mut list_state(Some(0)));
    albums.draw(chunks[1], buf, &mut list_state(Some(0)));
    songs.draw(chunks[2], buf, &mut list_state(Some(0)));
}
```