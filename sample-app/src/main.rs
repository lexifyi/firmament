use firmament::log;

#[firmament::main]
async fn main() -> firmament::app::Result {
    log::debug!("Hello world!");

    Ok(())
}
