mod editor;
mod terminal;

use editor::Editor;
pub use terminal::Terminal;
pub use editor::Position;

fn main() -> Result<(), std::io::Error> {
    Editor::default().run()
}
