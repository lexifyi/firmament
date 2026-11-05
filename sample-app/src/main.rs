use firmament::{log, web};

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut server = web::Server::new();

    server.listen_address.set_port(0);
    server.api.add_socket("echo", async |mut socket| {
        while let Some(text) = socket.recv_raw().await {
            log::info!("{:?}", text.as_str());

            socket.send_raw(text).await;
        }
    });

    server.start().await?.join().await;

    Ok(())
}
