use editor::Editor;

mod editor;
mod terminal;
pub use terminal::Terminal;
pub use editor::Position;

fn main() {
    Editor::default().run();
}