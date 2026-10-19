use std::{
    env::{self, var_os},
    path::PathBuf,
    sync::LazyLock,
};

/// Path to the data root directory.
pub static DATA_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(find_data_dir);

/// Path to the project root directory.
pub static PROJECT_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(find_project_dir);

fn find_data_dir() -> Option<PathBuf> {
    var_os("FIRMAMENT_DATA_DIR").map(PathBuf::from).or_else(|| {
        PROJECT_DIR.as_ref().cloned().map(|mut d| {
            d.push(".local/data");
            d
        })
    })
}

fn find_project_dir() -> Option<PathBuf> {
    let mut path = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) => PathBuf::from(dir),
        _ => match env::current_dir() {
            Ok(dir) => dir,
            _ => return None,
        },
    };

    loop {
        path.push(".git");
        let found = path.is_dir();
        path.pop();

        if found {
            return Some(path);
        }

        if !path.pop() {
            return None;
        }
    }
}
