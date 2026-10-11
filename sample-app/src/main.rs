use std::time::Duration;

use firmament::{log, repeat_every};

#[firmament::main]
async fn main() -> firmament::app::Result {
    repeat_every(Duration::from_secs(1), async || {
        log::info!("Hello world!");
    })
    .await;
}
