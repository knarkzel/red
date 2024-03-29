pub mod marker;
pub mod mode;
pub mod screen;

use mode::*;
use std::{
    fs::{read_to_string, File},
    io::{self, prelude::*, stdin},
};
use termion::{clear, color, cursor::Goto, event::Key, input::TermRead, style, terminal_size};

fn main() {
    Editor::new().load_file().run();
}

const NUMBERS_PADDING: usize = 4;

#[derive(Default)]
struct Editor {
    lines: Vec<String>,
    file: String,
    status_bar: String,
    command: String,
    size: (usize, usize),
    mode: mode::Mode,
    offset: marker::Marker,
    cursor: marker::Marker,
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
            self.lines = read_to_string(&file)
                .expect("Failed to read file")
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<_>>();
            self.file = file;
        }
        self
    }

    fn current_line(&self) -> usize {
        self.cursor.1 + self.offset.1
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
            Mode::Spatial(_) => self
                .screen
                .write(format!("{}", termion::cursor::SteadyUnderline)),
            _ => (),
        }
    }

    fn save(&self) -> io::Result<()> {
        let mut file = File::create(&self.file)?;
        for line in self.lines.iter() {
            file.write(line.as_bytes())?;
            file.write(&['\n' as u8])?;
        }
        Ok(())
    }

    fn align_scroll(&mut self) {
        // check vertically VVV
        if self.cursor.1 > self.size.1.saturating_sub(3) {
            self.offset.increase_y(1);
            self.cursor.decrease_y(1);
        } else if self.cursor.1 == 0 && self.offset.1 > 0 {
            self.offset.decrease_y(1);
        }

        // check horizontally >>>
        if self.cursor.0 > self.size.0.saturating_sub(1) {
            self.offset.increase_x(1);
            self.cursor.decrease_x(1);
        } else if self.cursor.0 == 0 && self.offset.0 > 0 {
            self.offset.decrease_x(1);
        }
    }

    fn scroll_to(&mut self, line: usize) {
        self.offset.1 = line.saturating_sub(10);
        self.cursor.1 = line.saturating_sub(self.offset.1 + 1);
        self.cursor.0 = 0;
    }

    fn delete_end(&mut self) {
        let current_line = self.current_line();
        let _ = self.lines[current_line].split_off(self.cursor.0.saturating_sub(1) as usize);
    }

    fn reset_x(&mut self) {
        self.offset.0 = 0;
        self.cursor.0 = 0;
    }

    fn update(&mut self) {
        // size
        let term_size = terminal_size().unwrap();
        self.size = (term_size.0 as usize - NUMBERS_PADDING, term_size.1 as usize);

        // cursor bounds
        let mut len = (self.get_line_len(0), self.lines.len());
        if self.mode == Mode::Insert {
            len.0 += 1;
        }
        self.cursor.align_bounds(&self.offset, len);

        // scroll bounds
        self.align_scroll();

        // other draw
        self.draw_screen();
        self.render_status();

        // move cursor to self.cursor
        if self.mode == Mode::Command {
            self.screen
                .write(format!("{}:{}", Goto(1, self.size.1 as u16), self.command));
        } else {
            self.screen.write(format!(
                "{}",
                Goto(
                    2 + (self.cursor.0 + NUMBERS_PADDING) as u16,
                    1 + self.cursor.1 as u16
                )
            ))
        }
        self.screen.flush();
    }

    fn draw_screen(&mut self) {
        // draw contents to screen
        for (i, line) in self
            .lines
            .iter()
            .skip(self.offset.1)
            .take(self.size.1.saturating_sub(2))
            .enumerate()
        {
            let temp = line.as_str();
            let slice = if temp.len() + NUMBERS_PADDING >= self.offset.0 {
                let bound_x2 = self.offset.0 + self.size.0;
                if temp.len() > bound_x2 {
                    temp.get(self.offset.0..bound_x2)
                } else {
                    temp.get(self.offset.0..)
                }
            } else {
                None
            };
            // show relative numbers
            // TODO
            let relative_number = (i as isize - self.cursor.1 as isize).abs();
            self.screen.write(format!(
                "{}{}{:>4}{}{}",
                Goto(1, 1 + i as u16),
                color::Fg(color::LightBlack),
                relative_number,
                color::Fg(color::Reset),
                termion::cursor::Right(1)
            ));
            if self.cursor.1 == i {
                self.screen.write(format!(
                    "{}{}",
                    color::Bg(color::Rgb(0x30, 0x30, 0x30)),
                    clear::CurrentLine
                ));
            }

            // then draw text to screen
            if let Some(slice) = slice {
                self.screen
                    .write(format!("{}{}", slice, color::Bg(color::Reset)));
            }
        }
    }

    fn render_status(&mut self) {
        // status bar
        let status_mode = {
            let mode = match self.mode {
                Mode::Insert => "INSERT",
                Mode::Normal => "NORMAL",
                Mode::Command => "COMMAND",
                Mode::Spatial(_) => "SPATIAL",
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
        let status_bar_pos = self.size.1 as u16 - 1;
        self.screen.write(format!(
            "{}{}{}{}",
            Goto(1, status_bar_pos),
            color::Bg(color::Rgb(0x35, 0x35, 0x35)),
            clear::CurrentLine,
            self.status_bar
        ));
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
                    match self.mode {
                        Mode::Normal => self.handle_normal(key),
                        Mode::Insert => self.handle_insert(key),
                        Mode::Spatial(letter) => self.handle_spatial(letter, key),
                        Mode::Command => {
                            if self.handle_command(key) {
                                break 'outer;
                            }
                        }
                    }
                    self.update();
                }
            }
        }
    }

    fn handle_normal(&mut self, key: Key) {
        match key {
            Key::Char('h') => self.cursor.decrease_x(1),
            Key::Char('j') => self.cursor.increase_y(1),
            Key::Char('k') => self.cursor.decrease_y(1),
            Key::Char('l') => self.cursor.increase_x(1),
            Key::Char('i') => self.switch_mode(Mode::Insert),
            Key::Char('d') => self.mode = Mode::Spatial('d'),
            Key::Char('G') => self.scroll_to(self.lines.len()),
            Key::Char('a') => {
                self.switch_mode(Mode::Insert);
                self.cursor.increase_x(1);
            }
            Key::Char('x') => {
                let len = self.get_line_len(0);
                let current_line = self.current_line();
                if len > 0 {
                    self.lines[current_line].remove(self.cursor.0);
                    if self.cursor.0 + 2 > len {
                        self.cursor.decrease_x(1);
                    }
                }
            }
            Key::Char('A') => {
                self.switch_mode(Mode::Insert);
                let len = self.get_line_len(0);
                if len > self.size.0 {
                    self.offset.0 = len - self.size.0 + 2;
                    self.cursor.0 = self.size.0 - 2;
                } else {
                    self.offset.0 = 0;
                    self.cursor.0 = len;
                }
                self.cursor.increase_x(1);
            }
            Key::Char('I') => {
                self.switch_mode(Mode::Insert);
                self.reset_x();
            }
            Key::Char('o') => {
                self.switch_mode(Mode::Insert);
                self.lines.insert(self.current_line() + 1, String::new());
                // why is it zero? idk
                self.cursor.0 = 1;
                self.cursor.1 += 1;
            }
            Key::Char('O') => {
                self.switch_mode(Mode::Insert);
                self.lines.insert(self.current_line(), String::new());
                // again, tiny hack
                self.cursor.0 = 1;
            }
            Key::Char('0') => {
                self.reset_x();
            }
            Key::Char('S') => {
                self.switch_mode(Mode::Insert);
                let current_line = self.current_line();
                self.lines[current_line] = String::new();
                self.offset.1 = 0;
                self.cursor.0 = 1;
            }
            Key::Char('C') => {
                self.switch_mode(Mode::Insert);
                self.delete_end();
            }
            Key::Char('D') => {
                self.delete_end();
                self.cursor.0 = self.cursor.0.saturating_sub(1);
            }
            Key::Char('$') => {
                let len = self.get_line_len(0);
                if len > self.size.0 {
                    self.offset.0 = len - self.size.0;
                    self.cursor.0 = self.size.0;
                } else {
                    self.offset.0 = 0;
                    self.cursor.0 = len;
                }
            }
            Key::Char(':') => self.mode = Mode::Command,
            Key::Ctrl('u') => {
                let delta = self.size.1 / 2;
                if self.offset.1 == 0 {
                    self.cursor.decrease_y(delta);
                }
                self.offset.1 = self.offset.1.saturating_sub(delta);
            }
            Key::Ctrl('d') => {
                let delta = self.size.1 as usize / 2;
                let temp = self.offset.1 + self.cursor.1 as usize;
                if temp + (self.size.1 as usize) < self.lines.len() {
                    self.offset.1 += delta;
                } else {
                    self.offset.1 += self.lines.len().saturating_sub(temp);
                }
            }
            _ => (),
        }
    }

    fn handle_insert(&mut self, key: Key) {
        match key {
            Key::Char('\t') => {
                let len = self.get_line_len(0);
                let current_line = self.current_line();
                if len > 0 {
                    self.lines[current_line]
                        .insert_str(self.offset.0 + self.cursor.0 as usize - 1, "    ");
                } else {
                    self.lines[current_line].push_str("    ");
                    if self.cursor.0 == 0 {
                        self.cursor.0 += 1;
                    }
                }
                self.cursor.0 += 4;
            }
            Key::Char('\n') => {
                let len = self.get_line_len(0);
                let current_line = self.current_line();
                let newline = if len > 0 {
                    self.lines[current_line].split_off(self.cursor.0 as usize - 1)
                } else {
                    String::new()
                };
                self.lines.insert(current_line + 1, newline);
                self.cursor.0 = 1;
                self.cursor.1 += 1;
            }
            Key::Left => self.cursor.decrease_x(1),
            Key::Down => self.cursor.increase_y(1),
            Key::Up => self.cursor.decrease_y(1),
            Key::Right => self.cursor.increase_x(1),
            Key::Esc => {
                self.switch_mode(Mode::Normal);
                self.cursor.decrease_x(1);
            }
            Key::Backspace => {
                let mut join = false;
                let len = self.get_line_len(0);
                let current_line = self.current_line();
                if self.cursor.0 <= 1 && self.cursor.1 > 1 {
                    join = true;
                } else if len > 0 && self.cursor.0 > 1 {
                    self.lines[current_line].remove(self.offset.0 + self.cursor.0 - 1);
                    self.cursor.decrease_x(1);
                }
                if join {
                    let joined_line = self.get_line(0).map(|s| s.to_string());
                    let mut split_point = 0;
                    if let Some(line) = self.get_line_mut(-1) {
                        split_point = line.len();
                        line.push_str(&joined_line.unwrap());
                    }
                    self.lines.remove(self.current_line());
                    self.cursor.decrease_y(1);
                    self.cursor.0 = split_point;
                }
            }
            Key::Char(c) => {
                let current_line = self.current_line();
                if self.get_line_len(0) > 0 {
                    self.lines[current_line].insert(self.cursor.0 + self.offset.0, c);
                } else {
                    self.lines[current_line].push(c);
                }
                self.cursor.increase_x(1);
            }
            _ => (),
        }
    }

    fn handle_command(&mut self, key: Key) -> bool {
        // returns true when we want to exit application
        match key {
            Key::Esc => {
                self.mode = Mode::Normal;
                self.command = String::new();
            }
            Key::Char('\n') => {
                // parse then exit
                let args = self.command.split(" ").collect::<Vec<_>>();
                if let Some(command) = args.get(0) {
                    match *command {
                        "w" | "wq" | "write" => {
                            if let Err(e) = self.save() {
                                self.screen.echo(format!("{}", e));
                            } else {
                                self.screen.echo(format!("\"{}\" written", self.file));
                            }
                            if command == &"wq" {
                                return true;
                            }
                        }
                        "q" | "quit" => return true,
                        _ => (),
                    }
                }
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
        }
        false
    }

    fn handle_spatial(&mut self, letter: char, key: Key) {
        match letter {
            'd' => match key {
                Key::Char('d') => {
                    self.lines.remove(self.current_line());
                    let current_line = self.current_line();
                    let len = self.lines[current_line].len();
                    if len == 0 {
                        self.cursor.0 = 1;
                    } else if self.cursor.0 > len {
                        self.cursor.0 = len;
                    }
                }
                _ => (),
            },
            _ => (),
        }
        self.mode = Mode::Normal;
    }
}
