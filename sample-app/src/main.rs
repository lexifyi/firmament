use firmament::log;

#[firmament::main]
async fn main() -> firmament::app::Result {
    let project_dir = firmament::PROJECT_DIR.as_ref();
    let data_dir = firmament::DATA_DIR.as_ref();

    log::info!("{project_dir:?}");
    log::info!("{data_dir:?}");

    Ok(())
}
