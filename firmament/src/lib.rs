pub mod app;
mod env;
pub use self::app::main;
pub use self::env::{ENV, Env};

pub mod log {
    pub use log::{debug, error, info, warn};
}
