use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    Percentage(u16),
    Length(u16),
    Fill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

static mut RECTS: Vec<Rect> = Vec::new();

#[inline(always)]
pub fn layout(area: Rect, direction: Direction, cons: &'_ [Constraint]) -> Vec<Rect> {
    layout_margin(area, direction, cons, (0, 0)).unwrap()
}

//This should never panic. Area is always clamped to be inside the input area.
pub fn layout_margin(
    area: Rect,
    direction: Direction,
    cons: &'_ [Constraint],
    margin: (u16, u16),
) -> Result<Vec<Rect>, &'static str> {
    unsafe {
        RECTS.clear();
        let area = area.inner(margin.0, margin.1)?;
        let mut x = area.x;
        let mut y = area.y;

        //Fill can only be used at the start or end.
        //There cannot be more than one.
        //TODO: Add to result.
        let count = cons.iter().filter(|con| **con == Constraint::Fill).count();

        if count > 1 {
            return Err("Cannot have multiple fill constraints.");
        }

        let start = cons.starts_with(&[Constraint::Fill]);
        let end = cons.ends_with(&[Constraint::Fill]);

        if !(start || end) && count == 1 {
            return Err("Fill constraint must be at the beginning or end.");
        }

        for con in cons {
            match con {
                Constraint::Percentage(p) => match direction {
                    Direction::Horizontal => {
                        let width = (area.width as f32 * (*p as f32 / 100.0)).round() as u16;
                        let width = if width + x >= area.right() {
                            area.right().saturating_sub(x)
                        } else {
                            width
                        };
                        RECTS.push(Rect::new(x, y, width, area.height));
                        x += width;
                    }
                    Direction::Vertical => {
                        let height = (area.height as f32 * (*p as f32 / 100.0)).round() as u16;
                        let height = if height + y >= area.bottom() {
                            area.bottom().saturating_sub(y)
                        } else {
                            height
                        };
                        RECTS.push(Rect::new(x, y, area.width, height));
                        y += height;
                    }
                },
                Constraint::Length(l) => match direction {
                    Direction::Horizontal => {
                        let l = if x + l >= area.right() {
                            area.right().saturating_sub(x)
                        } else {
                            *l
                        };
                        RECTS.push(Rect::new(x, y, l, area.height));
                        x += l;
                    }
                    Direction::Vertical => {
                        let l = if y + l >= area.bottom() {
                            area.bottom().saturating_sub(y)
                        } else {
                            *l
                        };
                        RECTS.push(Rect::new(x, y, area.width, l));
                        y += l;
                    }
                },
                _ => {}
            }
        }

        if start {
            let rem_y = area.height - y;
            let rem_x = area.width - x;
            match direction {
                Direction::Horizontal => {
                    for RECT in &mut RECTS {
                        RECT.x += rem_x;
                    }
                }
                Direction::Vertical => {
                    for RECT in &mut RECTS {
                        RECT.y += rem_y;
                    }
                }
            }

            RECTS.insert(0, Rect::new(0, 0, rem_x, rem_y));
        }

        if end {
            //TODO: Overflow?
            RECTS.push(Rect::new(x, y, area.width - x, area.height - y))
        }

        Ok(std::mem::take(&mut RECTS))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    /// Creates a new rect, with width and height limited to keep the area under max u16.
    /// If clipped, aspect ratio will be preserved.
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Rect {
        let max_area = u16::max_value();
        let (clipped_width, clipped_height) = if width as u32 * height as u32 > max_area as u32 {
            let aspect_ratio = width as f64 / height as f64;
            let max_area_f = max_area as f64;
            let height_f = (max_area_f / aspect_ratio).sqrt();
            let width_f = height_f * aspect_ratio;
            (width_f as u16, height_f as u16)
        } else {
            (width, height)
        };
        Rect {
            x,
            y,
            width: clipped_width,
            height: clipped_height,
        }
    }

    pub const fn centered(&self, width: u16, height: u16) -> Result<Rect, &'static str> {
        let v = self.height / 2;
        let h = self.width / 2;
        match self.inner(h.saturating_sub(width / 2), v.saturating_sub(height / 2)) {
            Ok(mut rect) => {
                rect.width = width;
                rect.height = height;

                if rect.bottom() > self.bottom() {
                    Err("Centered rectangle bounds exceed it's parent. Reduce the height.")
                } else if rect.right() > self.right() {
                    Err("Centered rectangle bounds exceed it's parent. Reduce the width.")
                } else {
                    Ok(rect)
                }
            }
            Err(err) => Err(err),
        }
    }

    pub const fn area(self) -> u16 {
        self.width * self.height
    }

    pub const fn left(self) -> u16 {
        self.x
    }

    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub const fn top(self) -> u16 {
        self.y
    }

    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub const fn inner(self, w: u16, h: u16) -> Result<Rect, &'static str> {
        if self.width < 2 * w {
            Err("Inner area exceeded outside area. Reduce margin width.")
        } else if self.height < 2 * h {
            Err("Inner area exceeded outside area. Reduce margin height.")
        } else {
            Ok(Rect {
                x: self.x + w,
                y: self.y + h,
                width: self.width - 2 * w,
                height: self.height - 2 * h,
            })
        }
    }

    pub fn union(self, other: Rect) -> Rect {
        let x1 = min(self.x, other.x);
        let y1 = min(self.y, other.y);
        let x2 = max(self.x + self.width, other.x + other.width);
        let y2 = max(self.y + self.height, other.y + other.height);
        Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    pub fn intersection(self, other: Rect) -> Rect {
        let x1 = max(self.x, other.x);
        let y1 = max(self.y, other.y);
        let x2 = min(self.x + self.width, other.x + other.width);
        let y2 = min(self.y + self.height, other.y + other.height);
        Rect {
            x: x1,
            y: y1,
            width: x2 - x1,
            height: y2 - y1,
        }
    }

    pub fn intersects(self, other: Rect) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }
}
