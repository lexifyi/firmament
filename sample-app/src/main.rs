use firmament::{log, web};

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut server = web::Server::new();

    server.listen_address.set_port(12834);
    server.api.add_socket("echo", async |mut socket| {
        log::info!("Connected.");

        while let Some(text) = socket.recv_text().await {
            log::info!("{:?}", text);
            socket.send_text(text).await;
        }

        log::info!("Disconnected.");
    });

    server.start().await?.join().await;

    Ok(())
}
