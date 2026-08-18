mod status;
pub use status::StatusDisplay;

pub mod tab_view;
pub use tab_view::TabView;

pub mod command;
pub mod config;
pub mod events;
mod logs;
pub mod measurements;
pub mod state;
pub mod subscriptions;
pub mod transactions;

mod machines_view;
pub use machines_view::MachinesPage;
