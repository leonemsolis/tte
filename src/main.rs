mod document;
mod editor;
mod highlight;
mod row;
mod terminal;

pub use row::Row;
pub use document::Document;
pub use editor::Editor;
pub use editor::SearchDirection;
pub use terminal::Terminal;
pub use editor::Position;

fn main() -> Result<(), std::io::Error> {
    Editor::default().run()
}
