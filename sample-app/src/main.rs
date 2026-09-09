use std::time;

struct App {}

impl firmament::App for App {
    async fn new() -> firmament::app::Result<Self> {
        Ok(App {})
    }

    async fn run(&'static self) -> firmament::app::Result {
        tokio::time::sleep(time::Duration::from_hours(1)).await;
        Ok(())
    }

    async fn on_exit_request(&'static self) {
        log::debug!("Exit requested.");
    }
}

firmament::main!(APP: App);
