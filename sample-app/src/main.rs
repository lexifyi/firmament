use std::net::Ipv4Addr;

use firmament::web;

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut server = web::Server::new();

    server.set_listen_ip(Ipv4Addr::LOCALHOST);
    server.set_listen_port(12345);

    let task = server.start().await?;

    task.join().await;

    Ok(())
}
