use crate::Position;

use std::io::{Write, Error};
use crossterm::terminal::{enable_raw_mode, Clear, ClearType};
use crossterm::event::{read, Event, KeyEvent};
use crossterm::cursor::{MoveTo, Hide, Show};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        let size = crossterm::terminal::size()?;
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
        print!("{}", Clear(ClearType::All))
    }

    pub fn cursor_position(position: &Position) {
        let Position {x, y} = position;
        let x = *x as u16;
        let y = *y as u16;
        print!("{}", MoveTo(x, y));
    }

    pub fn flush() -> Result<(), Error> {
        std::io::stdout().flush()
    }

    pub fn read_key() -> Result<KeyEvent, Error> {
        loop {
            let event = read()?;
            if let Event::Key(key_event) = event {
                return Ok(key_event);
            }
        }
    }

    pub fn cursor_hide() {
        print!("{}", Hide);
    }

    pub fn cursor_show() {
        print!("{}", Show);
    }

    pub fn clear_current_line() {
        print!("{}", Clear(ClearType::CurrentLine));
    }
}