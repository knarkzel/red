use std::io::{stdout, Stdout, Write};

use termion::{
    raw::{IntoRawMode, RawTerminal},
    screen::AlternateScreen,
};

pub struct TerminalScreen(pub AlternateScreen<RawTerminal<Stdout>>);

impl TerminalScreen {
    pub fn write<T: std::fmt::Display>(&mut self, argument: T) {
        write!(self.0, "{}", argument).expect("Failed writing to screen");
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
