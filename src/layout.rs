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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

static mut RECTS: Vec<Rect> = Vec::new();

pub fn layout(area: Rect, direction: Direction, cons: &'_ [Constraint]) -> Vec<Rect> {
    layout_margin(area, (0, 0), direction, cons)
}

pub fn layout_margin(
    area: Rect,
    margin: (u16, u16),
    direction: Direction,
    cons: &'_ [Constraint],
) -> Vec<Rect> {
    unsafe {
        RECTS.clear();
        let mut x = area.x + margin.0;
        let mut y = area.y + margin.1;

        match direction {
            Direction::Horizontal => {
                for con in cons {
                    match con {
                        Constraint::Percentage(p) => {
                            let width = (area.width as f32 * (*p as f32 / 100.0)).round() as u16;
                            let width = width.clamp(0, area.width - x - 1);
                            RECTS.push(Rect::new(x, y, width, area.height));
                            x += width;
                        }
                        Constraint::Length(l) => {
                            RECTS.push(Rect::new(x, y, *l, area.height));
                            x += l;
                        }
                    }
                }
            }
            Direction::Vertical => {
                for con in cons {
                    match con {
                        Constraint::Percentage(p) => {
                            let height = (area.height as f32 * (*p as f32 / 100.0)).round() as u16;
                            let height = height.clamp(0, area.height - y - 1);
                            RECTS.push(Rect::new(x, y, area.width, height));
                            y += height;
                        }
                        Constraint::Length(l) => {
                            RECTS.push(Rect::new(x, y, area.width, *l));
                            y += l;
                        }
                    }
                }
            }
        }

        std::mem::take(&mut RECTS)
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
        let (clipped_width, clipped_height) =
            if u32::from(width) * u32::from(height) > u32::from(max_area) {
                let aspect_ratio = f64::from(width) / f64::from(height);
                let max_area_f = f64::from(max_area);
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

    pub fn area(self) -> u16 {
        self.width * self.height
    }

    pub fn left(self) -> u16 {
        self.x
    }

    pub fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    pub fn top(self) -> u16 {
        self.y
    }

    pub fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    pub const fn inner(self, (v, h): (u16, u16)) -> Rect {
        if self.width < 2 * h || self.height < 2 * v {
            Rect {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
            }
        } else {
            Rect {
                x: self.x + h,
                y: self.y + v,
                width: self.width - 2 * h,
                height: self.height - 2 * v,
            }
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
