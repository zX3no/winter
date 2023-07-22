macro_rules! stylize {
    ($type:ty) => {
        impl Stylize for $type {
            fn bold(self) -> String {
                format!("\x1b[1m{}", self)
            }
            fn dim(self) -> String {
                format!("\x1b[2m{}", self)
            }
            fn italic(self) -> String {
                format!("\x1b[3m{}", self)
            }
            fn underline(self) -> String {
                format!("\x1b[4m{}", self)
            }
            fn blinking(self) -> String {
                format!("\x1b[5m{}", self)
            }
            ///Swaps background and foreground color
            fn invert(self) -> String {
                format!("\x1b[7m{}", self)
            }
            fn hidden(self) -> String {
                format!("\x1b[8m{}", self)
            }
            fn strikethrough(self) -> String {
                format!("\x1b[9m{}", self)
            }

            fn black(self) -> String {
                format!("\x1b[30m{}", self)
            }
            fn red(self) -> String {
                format!("\x1b[31m{}", self)
            }
            fn green(self) -> String {
                format!("\x1b[32m{}", self)
            }
            fn yellow(self) -> String {
                format!("\x1b[33m{}", self)
            }
            fn blue(self) -> String {
                format!("\x1b[34m{}", self)
            }
            fn magenta(self) -> String {
                format!("\x1b[35m{}", self)
            }
            fn cyan(self) -> String {
                format!("\x1b[36m{}", self)
            }
            fn white(self) -> String {
                format!("\x1b[37m{}", self)
            }

            fn bright_black(self) -> String {
                format!("\x1b[90m{}", self)
            }
            fn bright_red(self) -> String {
                format!("\x1b[91m{}", self)
            }
            fn bright_green(self) -> String {
                format!("\x1b[92m{}", self)
            }
            fn bright_yellow(self) -> String {
                format!("\x1b[93m{}", self)
            }
            fn bright_blue(self) -> String {
                format!("\x1b[94m{}", self)
            }
            fn bright_magenta(self) -> String {
                format!("\x1b[95m{}", self)
            }
            fn bright_cyan(self) -> String {
                format!("\x1b[96m{}", self)
            }
            fn bright_white(self) -> String {
                format!("\x1b[97m{}", self)
            }
        }
    };
}

stylize!(String);
stylize!(&str);

//https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences

///```rs
/// "Test".bold().underline().red()
/// ```
pub trait Stylize {
    fn bold(self) -> String;
    fn dim(self) -> String;
    fn italic(self) -> String;
    fn underline(self) -> String;
    fn blinking(self) -> String;
    ///Swaps background and foreground color
    fn invert(self) -> String;
    fn hidden(self) -> String;
    fn strikethrough(self) -> String;

    fn black(self) -> String;
    fn red(self) -> String;
    fn green(self) -> String;
    fn yellow(self) -> String;
    fn blue(self) -> String;
    fn magenta(self) -> String;
    fn cyan(self) -> String;
    fn white(self) -> String;

    //TODO: I THINK THESE ARE JUST BOLD COLORS.
    fn bright_black(self) -> String;
    fn bright_red(self) -> String;
    fn bright_green(self) -> String;
    fn bright_yellow(self) -> String;
    fn bright_blue(self) -> String;
    fn bright_magenta(self) -> String;
    fn bright_cyan(self) -> String;
    fn bright_white(self) -> String;
}
