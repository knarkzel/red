use std::io::{stdout, Stdout, Write};

use termion::{
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
    terminal_size,
    cursor::Goto,
};

pub struct TerminalScreen(pub AlternateScreen<RawTerminal<Stdout>>);

impl TerminalScreen {
    pub fn write<T: std::fmt::Display>(&mut self, argument: T) {
        write!(self.0, "{}", argument).expect("Failed writing to screen");
    }
    pub fn echo<T: std::fmt::Display>(&mut self, argument: T) {
        let size = terminal_size().expect("Failed to get size");
        self.write(format!("{}{}", Goto(1, size.1), argument));
    }
    pub fn flush(&mut self) {
        self.0.flush().unwrap();
    }
}

impl Default for TerminalScreen {
    fn default() -> Self {
        Self(AlternateScreen::from(stdout().into_raw_mode().unwrap()))
    }
}
