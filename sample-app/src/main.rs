use firmament::{log, postgres};

#[firmament::main]
async fn main() -> firmament::app::Result {
    let pg = postgres::connect().await?;
    let message: String = pg.query_one_scalar("SELECT 'Hello world.'", &[]).await?;

    log::info!("{message:?}");

    Ok(())
}
