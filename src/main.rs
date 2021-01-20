use std::fs::read_to_string;
use std::io::Stdout;
use std::io::{stdin, stdout, Write};
use std::process::exit;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::screen::*;
use termion::terminal_size;

#[derive(Debug)]
enum Mode {
    Insert,
    Normal,
}

#[derive(Debug)]
struct Editor {
    current_line: usize,
    cursor: (u16, u16),
    lines: Vec<String>,
    mode: Mode,
}

impl Editor {
    fn new() -> Self {
        Self {
            current_line: 0,
            cursor: (1, 1),
            lines: vec![],
            mode: Mode::Normal,
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
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        let mut screen = AlternateScreen::from(stdout().into_raw_mode().unwrap());
        self.update(&mut screen);
        loop {
            let stdin = stdin();
            for key in stdin.keys() {
                if let Ok(key) = key {
                    if (key == Key::Ctrl('c')) | (key == Key::Ctrl('z')) {
                        exit(0);
                    }
                    match self.mode {
                        Mode::Normal => match key {
                            Key::Char('h') => self.cursor.0 -= 1,
                            Key::Char('j') => self.cursor.1 += 1,
                            Key::Char('k') => self.cursor.1 -= 1,
                            Key::Char('l') => self.cursor.0 += 1,
                            Key::Char('i') | Key::Char('a') => {
                                self.mode = Mode::Insert;
                                write!(screen, "{}", termion::cursor::SteadyBar)
                                    .expect("Failed to switch cursor");
                                if key == Key::Char('a') {
                                    self.cursor.0 += 1;
                                }
                            }
                            _ => (),
                        },
                        Mode::Insert => match key {
                            Key::Char(c) => println!("{}", c),
                            Key::Esc => {
                                self.cursor.0 -= 1;
                                self.mode = Mode::Normal;
                                write!(screen, "{}", termion::cursor::SteadyBlock)
                                    .expect("Failed to switch cursor");
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
            .skip(self.current_line)
            .take(height)
            .enumerate()
        {
            let mut line = line.as_str();
            if line.len() > width {
                line = &line[..width];
            }
            write!(screen, "{}{}", termion::cursor::Goto(1, i as u16), line)
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
