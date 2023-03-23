use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::io::Error;
use std::time::Duration;
use std::time::Instant;
use crossterm::event::{ KeyEvent, KeyCode, KeyModifiers };
use crossterm::style::Color;

const STATUS_FG_COLOR:Color = Color::Rgb { r: 63, g: 63, b: 63 };
const STATUS_BG_COLOR:Color = Color::Rgb { r: 239, g: 239, b: 239 };
const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone, Copy)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        let document = if let Some(file_name) = args.get(1) {
            let doc = Document::open(file_name);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self { 
            should_quit: false, 
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
        Ok(())
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Bye bye!\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name:Option<String> = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            KeyEvent { modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('q'), ..} => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changes. Press Ctrl-Q {} more times to quit.", 
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                }
                self.should_quit = true;
            },
            KeyEvent { modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('s'), .. } => self.save(),
            KeyEvent { modifiers: KeyModifiers::CONTROL, code: KeyCode::Char('f'), .. } => self.search(),
            KeyEvent { code: KeyCode::Enter, .. } => {
                self.document.insert(&self.cursor_position, '\n');
                self.cursor_position = Position{ x: 0, y: self.cursor_position.y + 1 };
            },
            KeyEvent { code: KeyCode::Char(c), .. } => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(KeyCode::Right);
            },
            KeyEvent { code: KeyCode::Delete, .. } => self.document.delete(&self.cursor_position),
            KeyEvent { code: KeyCode::Backspace, .. } => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(KeyCode::Left);
                    self.document.delete(&self.cursor_position);
                }
            },
            KeyEvent { 
                code: 
                KeyCode::Up 
                | KeyCode::Down 
                | KeyCode::Right 
                | KeyCode::Left 
                | KeyCode::PageUp
                | KeyCode::PageDown
                | KeyCode::Home
                | KeyCode::End, .. } => self.move_cursor(pressed_key.code),
            _ => (),
        }
        self.scroll();
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }
        Ok(())
    }

    fn scroll(&mut self) {
        let Position {x, y} = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }
        if x < offset.x { 
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key_code: KeyCode) {
        let terminal_height = self.terminal.size().height as usize;
        let Position {mut x, mut y} = self.cursor_position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key_code {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y.saturating_add(1) < height {
                    y = y.saturating_add(1)
                }
            },
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = if let Some(row) = self.document.row(y) {
                        row.len()
                    } else {
                        0
                    };
                }
            },
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y.saturating_add(1) < height {
                    y += 1;
                    x = 0;
                }
            },
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            },
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height.saturating_sub(1)
                }
            },
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y };
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("TTE -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding);
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            Terminal::clear_current_line();
            if let Some(row) = self.document.row(self.offset.y.saturating_add(terminal_row as usize)) {
                self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_status_bar(&self) {
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        let mut file_name = "[NO NAME]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }

        let mut status = format!(
            "{} - {} lines{}", 
            file_name, 
            self.document.len(),
            modified_indicator
        );
        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>, Error> 
    where
        C: FnMut(&mut Self, KeyEvent, &String)
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;
            let key_event = Terminal::read_key()?;
            match key_event {
                KeyEvent { modifiers: KeyModifiers::CONTROL, .. } => continue,
                KeyEvent { code: KeyCode::Enter, .. } => break,
                KeyEvent { code: KeyCode::Esc, .. } => {
                    result.truncate(0);
                    break;
                },
                KeyEvent { code: KeyCode::Backspace, .. } => result.truncate(result.len().saturating_sub(1)),
                KeyEvent { code: KeyCode::Char(c), .. } => result.push(c),
                _ => {}
            }
            callback(self, key_event, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt("Search (ESC to canel, Arrows to navigate): ", 
            |editor, key_event, query| {
                let mut moved = false;
                match key_event {
                    KeyEvent { code: KeyCode::Right | KeyCode::Down, .. } => {
                        direction = SearchDirection::Forward;
                        editor.move_cursor(KeyCode::Right);
                        moved = true;
                    },
                    KeyEvent { code: KeyCode::Left | KeyCode::Up, .. } => {
                        direction = SearchDirection::Backward;
                    },
                    _ => direction = SearchDirection::Forward,
                }

                if let Some(position) = 
                    editor
                        .document
                        .find(&query, &editor.cursor_position, direction)
                {
                    editor.cursor_position = position;
                    editor.scroll();
                } else if moved {
                    editor.move_cursor(KeyCode::Left);
                } else {
                    editor.cursor_position = old_position;
                    editor.scroll();
                }
            })
            .unwrap_or(None);
        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
    }
}

fn die(e: Error) {
    Terminal::clear_screen();
    panic!("{:?}", e);
}