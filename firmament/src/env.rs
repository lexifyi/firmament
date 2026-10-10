use std::{
    env::{self, home_dir, var_os},
    path::PathBuf,
    sync::LazyLock,
};

/// Path to the data root directory.
pub static DATA_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(find_data_dir);

fn find_data_dir() -> Option<PathBuf> {
    var_os("FIRMAMENT_DATA_DIR")
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
        })
        .map(|dir| match env::current_dir() {
            Ok(mut cwd) => {
                cwd.push(dir);
                cwd
            }

            _ => dir,
        })
}
