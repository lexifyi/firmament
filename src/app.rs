pub mod error {
    pub use anyhow::{Error, bail, ensure};
    pub type Result<T = (), E = Error> = core::result::Result<T, E>;
}

use std::{cell::UnsafeCell, ops, process};

pub use self::error::{Error, Result};

#[allow(async_fn_in_trait)]
pub trait App: Sync + Sized + 'static {
    async fn new() -> Result<Self>;
    async fn run(&'static self) -> Result;
    async fn on_exit_request(&'static self) {}
}

#[cfg(debug_assertions)]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const DEFAULT_LOG_LEVEL: log::LevelFilter = log::LevelFilter::Info;

#[doc(hidden)]
pub unsafe fn __run<A: App>(instance: &'static Instance<A>, crate_name: &str) {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module(crate_name, DEFAULT_LOG_LEVEL)
        .init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        match A::new().await {
            Ok(app) => {
                let app = unsafe {
                    let cell = &mut *instance.cell.get();

                    debug_assert!(cell.is_none(), "app is already running");
                    *cell = Some(app);
                    cell.as_ref().unwrap_unchecked()
                };

                #[cfg(unix)]
                {
                    use std::thread;

                    use tokio::{
                        signal::unix::{SignalKind, signal},
                        task::LocalSet,
                    };

                    let rt = rt.handle().clone();

                    thread::spawn(move || {
                        let local = LocalSet::new();

                        rt.block_on(local.run_until(async move {
                            let Ok(mut signal) = signal(SignalKind::terminate()) else {
                                return;
                            };

                            loop {
                                signal.recv().await;
                                app.on_exit_request().await;
                            }
                        }));
                    });
                }

                if let Err(err) = app.run().await {
                    log::error!("Could not run {crate_name}. {err}");
                    process::exit(-125);
                }
            }

            Err(err) => {
                log::error!("Could not initialize {crate_name}. {err}");
                process::exit(-126);
            }
        }
    });
}

pub struct Instance<A: 'static> {
    cell: UnsafeCell<Option<A>>,
}

unsafe impl<A: App> Sync for Instance<A> {}

impl<A: App> Instance<A> {
    #[doc(hidden)]
    #[allow(clippy::new_without_default)]
    pub const unsafe fn new() -> Self {
        Self {
            cell: UnsafeCell::new(None),
        }
    }
}

impl<A: App> ops::Deref for Instance<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let cell = &*self.cell.get();

            debug_assert!(cell.is_some(), "app is not initialized");
            cell.as_ref().unwrap_unchecked()
        }
    }
}

#[macro_export]
macro_rules! main {
    ($APP:ident : $App:ty) => {
        static $APP: $crate::app::Instance<$App> = unsafe { $crate::app::Instance::new() };

        fn main() {
            unsafe {
                $crate::app::__run(
                    &$APP,
                    module_path!()
                        .split_once("::")
                        .map(|x| x.0)
                        .unwrap_or(module_path!()),
                )
            }
        }
    };
}
