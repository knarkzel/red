use std::{
    fs::read_to_string,
    io::{stdin, stdout, Stdout, Write},
};
use termion::{
    event::Key,
    input::TermRead,
    raw::{IntoRawMode, RawTerminal},
    screen::*,
    terminal_size,
};

mod cursor;

#[derive(Debug)]
enum Mode {
    Insert,
    Normal,
}

#[derive(Debug)]
struct Editor {
    mode: Mode,
    cursor: cursor::Cursor,
    offset: (usize, usize),
    lines: Vec<String>,
}

impl Editor {
    fn new() -> Self {
        Self {
            mode: Mode::Normal,
            cursor: cursor::Cursor::new(),
            offset: (0, 0),
            lines: vec![],
        }
    }
    fn load_file(self) -> Self {
        let file = std::env::args().skip(1).next();
        let lines = if let Some(file) = file {
            read_to_string(file)
                .expect("Failed to read file")
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            vec![]
        };
        Self { lines, ..self }
    }
    fn run(mut self) {
        // start
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
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
                    match self.mode {
                        Mode::Normal => match key {
                            Key::Char('h') => self.cursor.move_left(),
                            Key::Char('j') => {
                                self.cursor
                                    .move_down(&self.offset, self.lines.len(), get_line!(1))
                            }
                            Key::Char('k') => self.cursor.move_up(&self.offset, get_line!(-1)),
                            Key::Char('l') => self.cursor.move_right(&self.offset, get_line!(0)),
                            Key::Char('i') | Key::Char('a') => {
                                switch_insert!();
                                if key == Key::Char('a') {
                                    self.cursor.0 += 1;
                                }
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
                                self.cursor
                                    .move_down(&self.offset, self.lines.len(), get_line!(1))
                            }
                            Key::Char('O') => {
                                switch_insert!();
                                self.lines.insert(current_line!(), String::new());
                                // again, tiny hack
                                self.cursor.0 = 0;
                            }
                            Key::Char('0') => self.cursor.0 = 1,
                            Key::Char('$') => {
                                if let Some(line) = get_line!(0) {
                                    self.cursor.0 = (line.len() - self.offset.0) as u16;
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
                                        // tiny hack
                                        line.push(c);
                                        self.cursor.0 += 1;
                                    }
                                    self.cursor.0 += 1;
                                }
                            }
                            Key::Backspace => {
                                let mut delete = false;
                                if let Some(line) = get_line_mut!(0) {
                                    if line.len() > 0 {
                                        line.remove(self.cursor.0 as usize - 2);
                                        self.cursor.move_left();
                                    } else {
                                        delete = true;
                                    }
                                }
                                if delete {
                                    self.lines.remove(current_line!());
                                    self.cursor.move_up(&self.offset, get_line!(-1));
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
        write!(screen, "{}", termion::clear::All).expect("Failed to clear screen");
        let size = terminal_size().unwrap();
        let (width, height) = (size.0 as usize, size.1 as usize);
        for (i, line) in self
            .lines
            .iter()
            .skip(self.offset.1)
            .take(height)
            .enumerate()
        {
            let mut line = line.as_str();
            if line.len() > width {
                line = &line[..width];
            }
            write!(screen, "{}{}", termion::cursor::Goto(1, i as u16 + 1), line)
                .expect("Failed to print line");
        }

        write!(
            screen,
            "{}",
            termion::cursor::Goto(self.cursor.0, self.cursor.1)
        )
        .expect("Failed to move cursor");

        screen.flush().unwrap();
    }
}

fn main() {
    Editor::new().load_file().run();
}
