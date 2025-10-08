// View implementations (Main, Diff, Status, etc.)

pub mod diff_view;
pub mod help_view;
pub mod main_view;
pub mod manager;
pub mod status_view;
pub mod view;

pub use diff_view::DiffView;
pub use help_view::HelpView;
pub use main_view::MainView;
pub use manager::ViewManager;
pub use status_view::StatusView;
pub use view::{Action, View, ViewType};
