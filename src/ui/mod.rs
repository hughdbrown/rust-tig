// UI components and terminal management

pub mod app;
pub mod event;
pub mod terminal;

pub use app::App;
pub use event::{Event, EventHandler};
pub use terminal::Tui;
