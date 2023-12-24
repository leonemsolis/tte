use crate::Position;
use std::io::{Write, stdout};
use crossterm::cursor::MoveTo;
use crossterm::terminal::{enable_raw_mode, Clear, ClearType};
use crossterm::event::{read, Event, KeyEvent, KeyEventKind};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        let size = crossterm::terminal::size()?;
        enable_raw_mode().unwrap();
        Ok(Self {
            size: Size {
                width: size.0,
                height: size.1,
            }
        })
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn clear_screen() {
        print!("{}", Clear(ClearType::All));
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
        let Position {mut x, mut y} = position;
        let x = x as u16;
        let y = y as u16;
        print!("{}", MoveTo(x, y));
    }

    pub fn flush() -> Result<(), std::io::Error> {
        stdout().flush()
    }

    pub fn read_pressed_key_event() -> Result<KeyEvent, std::io::Error> {
        loop {
            match read() {
                Err(error) => {
                    return Err(error);
                },
                Ok(event) => {
                    if let Event::Key(key_event) = event {
                        if key_event.kind == KeyEventKind::Press 
                        || key_event.kind == KeyEventKind::Repeat {
                            return Ok(key_event);
                        }
                    }
                    continue;
                }
            }
        }
    }

    pub fn cursor_hide() {
        print!("{}", crossterm::cursor::Hide{});
    }

    pub fn cursor_show() {
        print!("{}", crossterm::cursor::Show{});
    }

    pub fn clear_current_line() {
        print!("{}", Clear(ClearType::CurrentLine));
    }
}