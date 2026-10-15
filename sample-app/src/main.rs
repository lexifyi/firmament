use firmament::web;

#[firmament::main]
async fn main() -> firmament::app::Result {
    let mut server = web::Server::new();

    server.bind_address = "127.0.0.1:12345".parse().unwrap();

    let task = server.start().await?;

    task.join().await;

    Ok(())
}
