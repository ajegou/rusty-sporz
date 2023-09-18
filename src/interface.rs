const COLORS_ENABLED: bool = true;

#[allow(dead_code)]
pub enum Color {
  Reset,
  Bright,
  Dim,
  Underscore,
  Blink,
  Reverse,
  Hidden,

  FgBlack,
  FgRed,
  FgGreen,
  FgYellow,
  FgBlue,
  FgMagenta,
  FgCyan,
  FgWhite,

  BgBlack,
  BgRed,
  BgGreen,
  BgYellow,
  BgBlue,
  BgMagenta,
  BgCyan,
  BgWhite,
}

impl Color {
  pub fn color(&self, text: &str) -> String {
    if COLORS_ENABLED {
      return format!("{}{}{}", self.get_characters(), text, Color::Reset.get_characters());
    } else {
      return String::from(text);
    }
  }
  fn get_characters(&self) -> &str {
    match self {
      Color::Reset => "\x1b[0m",
      Color::Bright => "\x1b[1m",
      Color::Dim => "\x1b[2m",
      Color::Underscore => "\x1b[4m",
      Color::Blink => "\x1b[5m",
      Color::Reverse => "\x1b[7m",
      Color::Hidden => "\x1b[8m",
    
      Color::FgBlack => "\x1b[30m",
      Color::FgRed => "\x1b[31m",
      Color::FgGreen => "\x1b[32m",
      Color::FgYellow => "\x1b[33m",
      Color::FgBlue => "\x1b[34m",
      Color::FgMagenta => "\x1b[35m",
      Color::FgCyan => "\x1b[36m",
      Color::FgWhite => "\x1b[37m",
    
      Color::BgBlack => "\x1b[40m",
      Color::BgRed => "\x1b[41m",
      Color::BgGreen => "\x1b[42m",
      Color::BgYellow => "\x1b[43m",
      Color::BgBlue => "\x1b[44m",
      Color::BgMagenta => "\x1b[45m",
      Color::BgCyan => "\x1b[46m",
      Color::BgWhite => "\x1b[47m",
    }
  }
}
