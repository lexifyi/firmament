pub mod error {
    pub use anyhow::{Error, bail, ensure};
    pub type Result<T = (), E = Error> = core::result::Result<T, E>;
}

use dotenv::dotenv;

pub use self::error::{Error, Result};

#[allow(async_fn_in_trait)]
pub trait App: Sync + Sized + 'static {
    async fn new() -> Result<Self>;
    async fn run(&'static self) -> Result;
}

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[doc(hidden)]
pub fn __run<A: App>(crate_name: &str) -> Result {
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

    rt.block_on(async {
        let app = Box::leak(Box::new(A::new().await?));

        app.run().await
    })
}

#[macro_export]
macro_rules! main {
    ($App:ty) => {
        fn main() -> std::process::ExitCode {
            let module_path = module_path!();
            let crate_name = module_path.split_once("::").map_or(module_path, |x| x.0);

            match $crate::app::__run::<$App>(crate_name) {
                Ok(_) => std::process::ExitCode::SUCCESS,

                Err(err) => {
                    $crate::log::error!("{err}");

                    std::process::ExitCode::FAILURE
                }
            }
        }
    };
}
