use firmament::app;

struct App {}

impl firmament::App for App {
    async fn new() -> app::Result<Self> {
        Ok(App {})
    }

    async fn run(&'static self) -> app::Result {
        use std::time;

        log::info!("Hello world!");

        tokio::time::sleep(time::Duration::from_hours(1)).await;

        Ok(())
    }
}

firmament::main!(App);
