use firmament::{Task, log};

#[firmament::main]
async fn main() -> firmament::app::Result {
    let task = Task::new(async {
        log::info!(
            "Data dir: {:?}",
            firmament::DATA_DIR
                .as_ref()
                .expect("failed to determine data dir")
        );
    });

    task.join().await;

    Ok(())
}
