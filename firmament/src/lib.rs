pub mod app;
pub use self::app::main;

pub mod log {
    pub use log::{debug, error, info, warn};
}
