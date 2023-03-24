mod document;
mod editor;
mod filetype;
mod highlighting;
mod row;
mod terminal;

pub use row::Row;
pub use document::Document;
pub use editor::Editor;
pub use editor::SearchDirection;
pub use filetype::FileType;
pub use filetype::HighlightingOptions;
pub use terminal::Terminal;
pub use editor::Position;

fn main() -> Result<(), std::io::Error> {
    Editor::default().run()
}
