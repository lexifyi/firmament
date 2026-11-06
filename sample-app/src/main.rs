use firmament::{log, web};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    message: String,
}

#[derive(Serialize)]
struct Response<'a> {
    message: &'a str,
}

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut server = web::Server::new();

    server.listen_address.set_port(12834);
    server.api.add_socket("echo", async |mut socket| {
        while let Some(text) = socket.recv::<Request>().await {
            log::info!("{:?}", text.message);

            socket
                .send(&Response {
                    message: &text.message,
                })
                .await;
        }
    });

    server.start().await?.join().await;

    Ok(())
}
