use firmament::{Email, base32, log};

#[firmament::main]
async fn main() -> firmament::app::Result {
    log::info!("{:?}", base32::encode("lolololo".as_bytes()));

    Ok(())
}
