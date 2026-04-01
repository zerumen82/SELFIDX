// Terminal module

pub mod capsule;
pub mod tui;
pub mod tui_enhanced;
pub mod history_search;

pub use capsule::*;
pub use history_search::{CommandHistory, HistorySearchState};
pub use tui::run_tui;
pub use tui_enhanced::run_enhanced_tui;
