use std::{
    env::{self, var_os},
    path::PathBuf,
    sync::LazyLock,
};

/// Path to the local firamement data directory.
pub static FIRMAMENT_HOME: LazyLock<PathBuf> =
    LazyLock::new(|| find_firmament_home().expect("failed to determine FIRMAMENT_HOME"));

fn find_firmament_home() -> Option<PathBuf> {
    var_os("FIRMAMENT_HOME").map(PathBuf::from).or_else(|| {
        env::home_dir().map(|mut d| {
            d.push(".firmament");
            d
        })
    })
}

/// Path to the project root directory.
pub static PROJECT_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(find_project_dir);

fn find_project_dir() -> Option<PathBuf> {
    let mut path = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) => PathBuf::from(dir),
        _ => return None,
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
