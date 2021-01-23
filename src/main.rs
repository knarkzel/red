// mods
pub mod cursor;
pub mod mode;
pub mod screen;

use std::{fs::read_to_string, io::stdin};
use termion::{clear, color, cursor::Goto, event::Key, input::TermRead, style, terminal_size};

use mode::*;

#[derive(Default)]
struct Editor {
    mode: Mode,
    file: String,
    status_bar: String,
    command: String,
    offset: (usize, usize),
    size: (u16, u16),
    lines: Vec<String>,
    cursor: cursor::Cursor,
    screen: screen::TerminalScreen,
}

impl Editor {
    fn new() -> Self {
        print!("{}{}", clear::All, Goto(1, 1));
        Self {
            lines: vec![String::new()],
            ..Self::default()
        }
    }
    fn load_file(mut self) -> Self {
        let file = std::env::args().skip(1).next();
        if let Some(file) = file {
            self.file = file.clone();
            self.lines = read_to_string(file)
                .expect("Failed to read file")
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
        }
        self
    }
    fn current_line(&self) -> usize {
        self.offset.1 + (self.cursor.1 - 1) as usize
    }
    fn get_line(&self, offset: isize) -> Option<&String> {
        self.lines
            .get((self.current_line() as isize + offset) as usize)
    }
    fn get_line_mut(&mut self, offset: isize) -> Option<&mut String> {
        let current_line = self.current_line() as isize;
        self.lines.get_mut((current_line + offset) as usize)
    }
    fn get_line_len(&self, offset: isize) -> usize {
        if let Some(line) = self.get_line(offset) {
            line.len()
        } else {
            0
        }
    }
    fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        match self.mode {
            Mode::Insert => self.screen.write(format!("{}", termion::cursor::SteadyBar)),
            Mode::Normal => self
                .screen
                .write(format!("{}", termion::cursor::SteadyBlock)),
            _ => (),
        }
    }
    fn run(mut self) {
        // start
        self.update();

        'outer: loop {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(key) = key {
                    self.screen.write(format!("{}", clear::All));
                    if (key == Key::Ctrl('c')) | (key == Key::Ctrl('z')) {
                        break 'outer;
                    }
                    // GLOBAL KEYS HERE PERHAPS?
                    match self.mode {
                        Mode::Normal => match key {
                            Key::Char('h') => self.cursor.move_left(&self.offset),
                            Key::Char('j') => {
                                self.cursor.move_down(
                                    &self.offset,
                                    self.lines.len(),
                                    self.get_line_len(1),
                                );
                            }
                            Key::Char('k') => {
                                self.cursor.move_up(&self.offset, self.get_line_len(-1))
                            }
                            Key::Char('l') => {
                                self.cursor.move_right(&self.offset, self.get_line_len(0))
                            }
                            Key::Char('i') => self.switch_mode(Mode::Insert),
                            Key::Char('a') => {
                                self.switch_mode(Mode::Insert);
                                if self.get_line_len(0) > 0 {
                                    self.cursor.0 += 1;
                                }
                            }
                            Key::Char('x') => {
                                let len = self.get_line_len(0);
                                let current_line = self.current_line();
                                if len > 0 {
                                    if self.cursor.1 > 1 {
                                        self.lines[current_line].remove(self.cursor.0 as usize - 1);
                                        if self.cursor.0 as usize > len {
                                            self.cursor.move_left(&self.offset);
                                        }
                                    }
                                }
                            }
                            Key::Char('G') => {
                                self.scroll_to(self.lines.len());
                            }
                            Key::Char('A') => {
                                self.switch_mode(Mode::Insert);
                                let len = self.get_line_len(0);
                                if len > self.size.0 as usize {
                                    self.offset.0 = len - self.size.0 as usize;
                                    self.cursor.0 = self.size.0;
                                } else {
                                    self.offset.0 = 0;
                                    self.cursor.0 = len as u16;
                                }
                                self.cursor.0 += 1;
                            }
                            Key::Char('I') => {
                                self.switch_mode(Mode::Insert);
                                self.offset.0 = 0;
                                self.cursor.0 = 1;
                            }
                            Key::Char('o') => {
                                self.switch_mode(Mode::Insert);
                                self.lines.insert(self.current_line() + 1, String::new());
                                // why is it zero? idk
                                self.cursor.0 = 0;
                                self.cursor.1 += 1;
                            }
                            Key::Char('O') => {
                                self.switch_mode(Mode::Insert);
                                self.lines.insert(self.current_line(), String::new());
                                // again, tiny hack
                                self.cursor.0 = 0;
                            }
                            Key::Char('0') => {
                                self.offset.0 = 0;
                                self.cursor.0 = 1;
                            }
                            Key::Char('$') => {
                                let len = self.get_line_len(0);
                                if len > self.size.0 as usize {
                                    self.offset.0 = len - self.size.0 as usize;
                                    self.cursor.0 = self.size.0;
                                } else {
                                    self.offset.0 = 0;
                                    self.cursor.0 = len as u16;
                                }
                            }
                            Key::Char(':') => self.mode = Mode::Command,
                            _ => (),
                        },
                        Mode::Insert => match key {
                            Key::Left => self.cursor.move_left(&self.offset),
                            Key::Down => self.cursor.move_down(
                                &self.offset,
                                self.lines.len(),
                                self.get_line_len(1),
                            ),
                            Key::Up => self.cursor.move_up(&self.offset, self.get_line_len(-1)),
                            Key::Right => {
                                self.cursor.move_right(&self.offset, self.get_line_len(0))
                            }
                            Key::Esc => {
                                if self.cursor.0 > 1 {
                                    self.cursor.0 -= 1;
                                }
                                self.switch_mode(Mode::Normal);
                            }
                            Key::Char(c) => {
                                let len = self.get_line_len(0);
                                let current_line = self.current_line();
                                if len > 0 {
                                    self.lines[current_line]
                                        .insert(self.offset.0 + self.cursor.0 as usize - 1, c);
                                } else {
                                    self.lines[current_line].push(c);
                                    if self.cursor.0 == 0 {
                                        self.cursor.0 += 1;
                                    }
                                }
                                self.cursor.0 += 1;
                            }
                            Key::Backspace => {
                                let mut join = false;
                                let len = self.get_line_len(0);
                                let current_line = self.current_line();
                                if self.cursor.0 <= 1 && self.cursor.1 > 1 {
                                    join = true;
                                } else if len > 0 && self.cursor.0 > 1 {
                                    self.lines[current_line]
                                        .remove(self.offset.0 + self.cursor.0 as usize - 2);
                                    self.cursor.move_left(&self.offset);
                                }
                                if join {
                                    let joined_line = self.get_line(0).map(|s| s.to_string());
                                    let mut split_point = 1;
                                    if let Some(line) = self.get_line_mut(-1) {
                                        split_point = line.len() as u16;
                                        line.push_str(&joined_line.unwrap());
                                    }
                                    self.lines.remove(self.current_line());
                                    self.cursor.1 -= 1;
                                    self.cursor.0 = split_point + 1;
                                }
                            }
                            _ => (),
                        },
                        Mode::Command => match key {
                            Key::Esc => {
                                self.mode = Mode::Normal;
                                self.command = String::new();
                            }
                            Key::Char('\n') => {
                                // parse then exit
                                self.mode = Mode::Normal;
                                self.command = String::new();
                            }
                            Key::Backspace => {
                                self.command.pop();
                            }
                            Key::Char(c) => {
                                self.command.push(c);
                            }
                            _ => (),
                        },
                    }
                    self.update();
                }
            }
        }
    }
    fn update(&mut self) {
        // size and scrolling
        self.size = terminal_size().unwrap();
        self.check_scroll();

        // draw self.screen.0
        let (width, height) = (self.size.0 as usize, self.size.1 as usize);
        for (i, line) in self
            .lines
            .iter()
            .skip(self.offset.1)
            .take(height.saturating_sub(2))
            .enumerate()
        {
            let temp = line.as_str();
            let slice = if temp.len() >= self.offset.0 {
                let (_, bound_x2) = (self.offset.0, width + self.offset.0);
                if temp.len() > bound_x2 {
                    temp.get(self.offset.0..(width + self.offset.0))
                } else {
                    temp.get(self.offset.0..)
                }
            } else {
                None
            };
            if let Some(slice) = slice {
                self.screen
                    .write(format!("{}{}", Goto(1, i as u16 + 1), slice));
            }
        }

        // status bar
        let status_mode = {
            let mode = match self.mode {
                Mode::Insert => "INSERT",
                Mode::Normal => "NORMAL",
                Mode::Command => "COMMAND",
            };
            format!(
                "{}{}{} {} {}{}{}",
                style::Bold,
                color::Bg(color::LightGreen),
                color::Fg(color::Black),
                mode,
                color::Bg(color::Reset),
                color::Fg(color::Reset),
                style::Reset,
            )
        };
        let status_file = {
            let file = &self.file;
            format!(
                "{}{} {} {}{}",
                color::Bg(color::LightBlue),
                color::Fg(color::Black),
                file,
                color::Bg(color::Reset),
                color::Fg(color::Reset)
            )
        };
        let status_position = {
            let col = self.cursor.0 as usize + self.offset.0;
            let line = self.cursor.1 as usize + self.offset.1;
            format!(
                "{}{} {}:{} {}{}",
                color::Bg(color::LightRed),
                color::Fg(color::Black),
                col,
                line,
                color::Bg(color::Reset),
                color::Fg(color::Reset)
            )
        };
        self.status_bar = format!("{}{}{}", status_mode, status_file, status_position);
        let status_bar_pos = height as u16 - 1;
        self.screen.write(format!(
            "{}{}{}{}",
            Goto(1, status_bar_pos),
            color::Bg(color::Rgb(0x35, 0x35, 0x35)),
            clear::CurrentLine,
            self.status_bar
        ));

        // move cursor to self.cursor
        if self.mode == Mode::Command {
            self.screen
                .write(format!("{}:{}", Goto(1, height as u16), self.command));
        } else {
            self.screen
                .write(format!("{}", Goto(self.cursor.0, self.cursor.1)))
        }
        self.screen.flush();
    }
    fn check_scroll(&mut self) {
        // check vertically
        let height = self.size.1 - 2;
        let increment = height / 2;
        if self.cursor.1 < 1 {
            self.offset.1 = self.offset.1.saturating_sub(increment as usize);
            self.cursor.1 = increment + 1;
        } else if self.cursor.1 > height {
            self.offset.1 += increment as usize;
            self.cursor.1 = height - increment;
        }

        // check horizontally
        if self.cursor.0 < 1 && self.offset.0 > 0 {
            self.offset.0 = self.offset.0.saturating_sub(1);
            self.cursor.0 += 1;
        }
        if self.cursor.0 > self.size.0 {
            self.offset.0 += 1;
            self.cursor.0 -= 1;
        }
    }
    fn scroll_to(&mut self, line: usize) {
        let height = (self.size.1 - 2) as usize;
        self.offset.1 = line.saturating_sub(height / 2);
        self.cursor.1 = line.saturating_sub(self.offset.1) as u16;
        self.cursor.0 = 1;
    }
}

fn main() {
    Editor::new().load_file().run();
}
