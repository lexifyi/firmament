use firmament::log;

#[firmament::main]
async fn main() -> firmament::app::Result {
    log::info!(
        "Data dir: {:?}",
        firmament::DATA_DIR
            .as_ref()
            .expect("failed to determine data dir")
    );

    Ok(())
}
