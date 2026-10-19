use std::{env, fmt};

use tokio::sync::OnceCell;
pub use tokio_postgres::{Client, Error, Statement};
use tokio_postgres::{Config, NoTls};

pub async fn connect() -> Result<Client, Error> {
    let config = CONFIG.get_or_init(create_config).await;
    let (client, connection) = config.connect(NoTls).await?;

    tokio::task::spawn(async {
        if let Err(err) = connection.await {
            log::error!("Connecttion error. {err}");
        }
    });

    Ok(client)
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn create_config() -> Config {
    let mut config = Config::new();

    if let Ok(host) = env::var("PGHOST") {
        config.host(host);
    }

    if let Ok(hostaddr) = env::var("PGHOSTADDR") {
        match hostaddr.parse() {
            Err(_) => log::warn!("PGHOSTADDR is not a valid IP address"),
            Ok(hostaddr) => {
                config.hostaddr(hostaddr);
            }
        }
    }

    if let Ok(user) = env::var("PGUSER") {
        config.user(user);
    }

    if let Ok(password) = env::var("PGPASSWORD") {
        config.password(password);
    }

    config
}
