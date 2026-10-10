use firmament::log;

#[firmament::main]
async fn main() -> firmament::app::Result {
    let env = &*firmament::ENV;

    log::info!("Data dir: {:?}", env.data_dir);

    Ok(())
}
