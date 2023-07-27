use bitflags::bitflags;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,

    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,

    #[default]
    Reset,
}

impl Color {
    pub fn fg_code(self) -> &'static str {
        match self {
            Color::Black => "\x1B[30m",
            Color::Red => "\x1B[31m",
            Color::Green => "\x1B[32m",
            Color::Yellow => "\x1B[33m",
            Color::Blue => "\x1B[34m",
            Color::Magenta => "\x1B[35m",
            Color::Cyan => "\x1B[36m",
            Color::White => "\x1B[37m",

            Color::BrightBlack => "\x1B[90m",
            Color::BrightRed => "\x1B[91m",
            Color::BrightGreen => "\x1B[92m",
            Color::BrightYellow => "\x1B[93m",
            Color::BrightBlue => "\x1B[94m",
            Color::BrightMagenta => "\x1B[95m",
            Color::BrightCyan => "\x1B[96m",
            Color::BrightWhite => "\x1B[97m",

            Color::Reset => "\x1B[0m",
        }
    }
    pub fn bg_code(self) -> &'static str {
        match self {
            Color::Black => "\x1B[40m",
            Color::Red => "\x1B[41m",
            Color::Green => "\x1B[42m",
            Color::Yellow => "\x1B[43m",
            Color::Blue => "\x1B[44m",
            Color::Magenta => "\x1B[45m",
            Color::Cyan => "\x1B[46m",
            Color::White => "\x1B[47m",

            Color::BrightBlack => "\x1B[100m",
            Color::BrightRed => "\x1B[101m",
            Color::BrightGreen => "\x1B[102m",
            Color::BrightYellow => "\x1B[103m",
            Color::BrightBlue => "\x1B[104m",
            Color::BrightMagenta => "\x1B[105m",
            Color::BrightCyan => "\x1B[106m",
            Color::BrightWhite => "\x1B[107m",

            Color::Reset => "\x1B[49m",
        }
    }
}

//TODO: Fix modifiers and make them work.
bitflags! {
    #[derive(Debug, Clone, PartialEq, Eq, Copy, Default)]
    pub struct Modifier: u16 {
        const BOLD              = 0b0000_0000_0001;
        const DIM               = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINED        = 0b0000_0000_1000;
        const FAST_BLINK        = 0b0000_0001_0000;
        const SLOW_BLINK        = 0b0000_0010_0000;
        const INVERT            = 0b0000_0100_0000;
        const HIDDEN            = 0b0000_1000_0000;
        const CROSSED_OUT       = 0b0001_0000_0000;
    }
}

pub const BOLD: &'static str = "\x1b[1m";
pub const DIM: &'static str = "\x1b[2m";
pub const ITALIC: &'static str = "\x1b[3m";
pub const UNDERLINE: &'static str = "\x1b[4m";
pub const FAST_BLINKING: &'static str = "\x1b[5m;1m";
pub const SLOW_BLINKING: &'static str = "\x1b[5m;2m";
pub const INVERT: &'static str = "\x1b[7m";
pub const HIDDEN: &'static str = "\x1b[8m";
pub const STRIKETHROUGH: &'static str = "\x1b[9m";

// pub const NO_BOLD: &'static str = "\x1b[21m";
pub const NO_BOLD_OR_DIM: &'static str = "\x1b[22m";
pub const NO_ITALIC: &'static str = "\x1b[23m";
pub const NO_UNDERLINE: &'static str = "\x1b[24m";
//TODO: Test fast and slow blinking does "\x1b[25m;2" work?
//Or does this do both?
pub const NO_BLINKING: &'static str = "\x1b[25m";
pub const NO_INVERT: &'static str = "\x1b[27m";
pub const NO_HIDDEN: &'static str = "\x1b[28m";
pub const NO_STRIKETHROUGH: &'static str = "\x1b[29m";

pub mod test {
    pub const BOLD: u16 = 0b0000_0000_0001;
    pub const DIM: u16 = 0b0000_0000_0010;
    pub const ITALIC: u16 = 0b0000_0000_0100;
    pub const UNDERLINED: u16 = 0b0000_0000_1000;
    pub const SLOW_BLINK: u16 = 0b0000_0001_0000;
    pub const RAPID_BLINK: u16 = 0b0000_0010_0000;
    pub const REVERSED: u16 = 0b0000_0100_0000;
    pub const HIDDEN: u16 = 0b0000_1000_0000;
    pub const CROSSED_OUT: u16 = 0b0001_0000_0000;
}

//TODO: Should this be Copy?
#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub modifier: Modifier,
}

impl Style {
    pub fn fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }
    pub fn bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }
}

macro_rules! modifier_helper {
    ($($modifier:ident => $value:ident),*) => {
        $(
            pub fn $modifier() -> Style {
                Style {
                    fg: Color::Reset,
                    bg: Color::Reset,
                    modifier: Modifier::$value,
                }
            }
        )*
    };
}

macro_rules! modifier {
    ($($modifier:ident => $value:ident),*) => {
        modifier_helper!($($modifier => $value),*);
        impl Style {
            $(
                pub fn $modifier(mut self) -> Self {
                    self.modifier.insert(Modifier::$value);
                    self
                }
            )*
        }
    };
}

modifier! {
    bold => BOLD,
    dim => DIM,
    italic => ITALIC,
    underlined => UNDERLINED,
    fast_blink => FAST_BLINK,
    slow_blink => SLOW_BLINK,
    invert => INVERT,
    hidden => HIDDEN,
    crossed_out => CROSSED_OUT
}

pub fn style() -> Style {
    Style::default()
}

pub fn fg(fg: Color) -> Style {
    Style {
        fg,
        bg: Color::Reset,
        modifier: Modifier::empty(),
    }
}

pub fn bg(bg: Color) -> Style {
    Style {
        fg: Color::Reset,
        bg,
        modifier: Modifier::empty(),
    }
}
