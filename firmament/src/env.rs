use std::{
    env::{home_dir, var_os},
    path::PathBuf,
    sync::LazyLock,
};

/// Shared instance of [`Env`].
pub static ENV: LazyLock<Env> = LazyLock::new(Env::new);

/// Contains information about the execution environment.
pub struct Env {
    pub data_dir: Option<PathBuf>,
}

impl Env {
    fn new() -> Self {
        let data_dir = var_os("FIRMAMENT_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| {
                var_os("CARGO_MANIFEST_DIR")
                    .map(PathBuf::from)
                    .map(|mut dir| {
                        dir.push(".local/share");
                        dir
                    })
            })
            .or_else(|| var_os("XDG_DATA_DIR").map(PathBuf::from))
            .or_else(|| {
                home_dir().map(|mut dir| {
                    dir.push(".local/share");
                    dir
                })
            });

        Self { data_dir }
    }
}
