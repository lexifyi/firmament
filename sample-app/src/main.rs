use firmament::{log, web};

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut server = web::Server::new();

    server.listen_address.set_port(12834);
    server.api.add_socket("echo", async |mut socket| {
        log::info!("Connected.");

        while let Some(text) = socket.recv_raw().await {
            log::info!("{:?}", text);
            socket.send_raw(text);
        }

        log::info!("Disconnected.");
    });

    server.start().await?.join().await;

    Ok(())
}
