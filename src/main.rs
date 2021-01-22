use std::{
    fs::read_to_string,
    io::{stdin, stdout, Stdout, Write},
};
use termion::{
    *,
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::*,
};

mod cursor;

#[derive(Debug, PartialEq)]
enum Mode {
    Insert,
    Normal,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Normal
    }
}

#[derive(Debug, Default)]
struct Editor {
    mode: Mode,
    status_bar: String,
    file: String,
    cursor: cursor::Cursor,
    offset: (usize, usize),
    size: (u16, u16),
    lines: Vec<String>,
}

impl Editor {
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
    fn run(mut self) {
        // start
        print!("{}{}", clear::All, termion::cursor::Goto(1, 1));
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        self.update(&mut screen);

        // macros
        macro_rules! current_line {
            () => {
                self.offset.1 + (self.cursor.1 - 1) as usize
            };
        }
        macro_rules! get_line {
            ($offset:expr) => {
                self.lines
                    .get((current_line!() as isize + $offset) as usize)
            };
        }
        macro_rules! get_line_mut {
            ($offset:expr) => {
                self.lines
                    .get_mut((current_line!() as isize + $offset) as usize)
            };
        }
        macro_rules! switch_insert {
            () => {
                self.mode = Mode::Insert;
                write!(screen, "{}", termion::cursor::SteadyBar).expect("Failed to switch cursor");
            };
        }

        'outer: loop {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(key) = key {
                    if (key == Key::Ctrl('c')) | (key == Key::Ctrl('z')) {
                        break 'outer;
                    }
                    // GLOBAL KEYS HERE PERHAPS?
                    match self.mode {
                        Mode::Normal => match key {
                            Key::Char('h') => self.cursor.move_left(),
                            Key::Char('j') => self.cursor.move_down(&self.offset, self.lines.len(), get_line!(1)),
                            Key::Char('k') => self.cursor.move_up(&self.offset, get_line!(-1)),
                            Key::Char('l') => self.cursor.move_right(&self.offset, get_line!(0)),
                            Key::Char('i') => {
                                switch_insert!();
                            }
                            Key::Char('a') => {
                                switch_insert!();
                                if get_line!(0).map(|t| t.len()).unwrap_or(0) > 0 {
                                    self.cursor.0 += 1;
                                }
                            }
                            Key::Char('x') => {
                                if let Some(line) = get_line_mut!(0) {
                                    if !line.is_empty() && self.cursor.1 > 1 {
                                        line.remove(self.cursor.0 as usize - 1);
                                        if self.cursor.0 as usize > line.len() {
                                            self.cursor.move_left();
                                        }
                                    }
                                }
                            }
                            Key::Char('G') => {
                                self.scroll_to(self.lines.len());
                            }
                            Key::Char('A') => {
                                switch_insert!();
                                if let Some(line) = get_line!(0) {
                                    self.cursor.0 = (line.len() - self.offset.0) as u16 + 1;
                                }
                            }
                            Key::Char('I') => {
                                switch_insert!();
                                self.cursor.0 = 1;
                            }
                            Key::Char('o') => {
                                switch_insert!();
                                self.lines.insert(current_line!() + 1, String::new());
                                // why is it zero? idk
                                self.cursor.0 = 0;
                                self.cursor.1 += 1;
                            }
                            Key::Char('O') => {
                                switch_insert!();
                                self.lines.insert(current_line!(), String::new());
                                // again, tiny hack
                                self.cursor.0 = 0;
                            }
                            Key::Char('0') => {
                                self.offset.0 = 0;
                                self.cursor.0 = 1;
                            }
                            Key::Char('$') => {
                                if let Some(line) = get_line!(0) {
                                    if line.len() > self.size.0 as usize {
                                        self.offset.0 = line.len() - (self.size.0 / 2) as usize;
                                        self.cursor.0 = self.size.0 / 2;
                                    } else {
                                        self.offset.0 = 0;
                                        self.cursor.0 = line.len() as u16;
                                    }
                                }
                            }
                            _ => (),
                        },
                        Mode::Insert => match key {
                            Key::Left => self.cursor.move_left(),
                            Key::Down => {
                                self.cursor
                                    .move_down(&self.offset, self.lines.len(), get_line!(1))
                            }
                            Key::Up => self.cursor.move_up(&self.offset, get_line!(-1)),
                            Key::Right => self.cursor.move_right(&self.offset, get_line!(0)),
                            Key::Esc => {
                                if self.cursor.0 > 1 {
                                    self.cursor.0 -= 1;
                                }
                                self.mode = Mode::Normal;
                                write!(screen, "{}", termion::cursor::SteadyBlock)
                                    .expect("Failed to switch cursor");
                            }
                            Key::Char(c) => {
                                if let Some(line) = get_line_mut!(0) {
                                    if line.len() > 0 {
                                        line.insert(self.cursor.0 as usize - 1, c);
                                    } else {
                                        line.push(c);
                                        if self.cursor.0 == 0 {
                                            self.cursor.0 += 1;
                                        }
                                    }
                                    self.cursor.0 += 1;
                                }
                            }
                            Key::Backspace => {
                                let mut join = false;
                                if let Some(line) = get_line_mut!(0) {
                                    if self.cursor.0 <= 1 && self.cursor.1 > 1 {
                                        join = true;
                                    } else if line.len() > 0 && self.cursor.0 > 1 {
                                        line.remove(self.cursor.0 as usize - 2);
                                        self.cursor.move_left();
                                    }
                                }
                                if join {
                                    let joined_line = get_line!(0).map(|s| s.to_string());
                                    let mut split_point = 1;
                                    if let Some(line) = get_line_mut!(-1) {
                                        split_point = line.len() as u16;
                                        line.push_str(&joined_line.unwrap());
                                    }
                                    self.lines.remove(current_line!());
                                    self.cursor.1 -= 1;
                                    self.cursor.0 = split_point + 1;
                                }
                            }
                            _ => (),
                        },
                    }
                    self.update(&mut screen);
                }
            }
        }
    }
    fn update(&mut self, screen: &mut AlternateScreen<RawTerminal<Stdout>>) {
        write!(screen, "{}", clear::All).expect("Failed to clear screen");

        // size and scrolling
        self.size = terminal_size().unwrap();
        self.check_scroll();

        // draw screen
        let (width, height) = (self.size.0 as usize, self.size.1 as usize);
        for (i, line) in self.lines.iter().skip(self.offset.1).take(height.saturating_sub(2)).enumerate() {
            let temp = line.as_str();
            let slice = if temp.len() >= self.offset.0 {
                let (bound_x1, bound_x2) = (self.offset.0, width + self.offset.0);
                if temp.len() > bound_x2 {
                    temp.get(self.offset.0..(width + self.offset.0))
                } else {
                    temp.get(self.offset.0..)
                }
            } else {
                None
            };
            // let line = temp.get(self.offset.0..(width + self.offset.0));
            // if line.len() > width {
            //     line = &line[..width];
            // }
            if let Some(slice) = slice {
                write!(screen, "{}{}", termion::cursor::Goto(1, i as u16 + 1), slice).expect("Failed to print line");
            }
        }

        // status bar
        let status_mode = {
            let mode = match self.mode {
                Mode::Insert => "INSERT",
                Mode::Normal => "NORMAL",
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
        write!(
            screen,
            "{}{}{}{}",
            termion::cursor::Goto(1, status_bar_pos),
            // color::Bg(color::LightBlack),
            color::Bg(color::Rgb(0x35, 0x35, 0x35)),
            clear::CurrentLine,
            self.status_bar
        )
        .expect("Failed to print status_bar");

        // move cursor to self.cursor
        write!(
            screen,
            "{}",
            termion::cursor::Goto(self.cursor.0, self.cursor.1)
        )
        .expect("Failed to move cursor");
        screen.flush().unwrap();
    }
    fn check_scroll(&mut self) {
        // check vertically
        let height = self.size.1 - 2;
        let mut increment = height / 2;
        if self.cursor.1 < 1 {
            self.offset.1 = self.offset.1.saturating_sub(increment as usize);
            self.cursor.1 = increment + 1;
        } else if self.cursor.1 > height {
            self.offset.1 += increment as usize;
            self.cursor.1 = height - increment;
        }

        // check horizontally 
        let width = self.size.0;
        increment = width / 2;
        if self.cursor.0 <= 1 && self.offset.0 > 0 {
            self.offset.0 = self.offset.0.saturating_sub(increment as usize);
            self.cursor.0 = width;
        }
        if self.cursor.0 > self.size.0 {
            self.offset.0 += increment as usize;
            self.cursor.0 = width - increment;
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
    Editor::default().load_file().run();
}
