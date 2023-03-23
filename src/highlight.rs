use crossterm::style::Color;

#[derive(PartialEq)]
pub enum Type {
    None,
    Number,
}

impl Type {
    pub fn to_color(&self) -> Color {
        match self {
            Type::Number => Color::Rgb { r: 220, g: 163, b: 163 },
            _ => Color::Rgb { r: 255, g: 255, b: 255 },
        }
    }
}