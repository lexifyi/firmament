#[cfg(debug_assertions)]
use dotenv::dotenv;
pub use firmament_proc_macros::main;

pub use self::error::{Error, Result};

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[doc(hidden)]
pub fn __run<F>(crate_name: &str, main: fn() -> F) -> Result
where
    F: Future<Output = Result>,
{
    #[cfg(debug_assertions)]
    let _ = dotenv();

    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module(crate_name, DEFAULT_LOG_LEVEL)
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async { main().await })
}

pub mod error {
    pub use anyhow::{Error, bail, ensure};
    pub type Result<T = (), E = Error> = core::result::Result<T, E>;
}
