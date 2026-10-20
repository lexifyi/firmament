use std::{env, io, net::IpAddr, str::FromStr};

use serde::Deserialize;
use tokio::{fs, sync::OnceCell};
pub use tokio_postgres::{Client, Error, Statement};
use tokio_postgres::{Config, NoTls};

use crate::FIRMAMENT_HOME;

pub async fn connect() -> Result<Client, Error> {
    let config = CONFIG.get_or_init(create_config).await;
    let (client, connection) = config.connect(NoTls).await?;

    tokio::task::spawn(async {
        if let Err(err) = connection.await {
            log::error!("Connection error. {err}");
        }
    });

    Ok(client)
}

static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn create_config() -> Config {
    let mut config = Config::new();
    let toml_config_path = FIRMAMENT_HOME.join("postgres.toml");

    match fs::read_to_string(&toml_config_path).await {
        Ok(toml) => match toml::from_str::<TomlConfig>(&toml) {
            Ok(toml) => {
                config.user(toml.user);
                config.password(toml.password);

                match IpAddr::from_str(&toml.host) {
                    Ok(ip) => {
                        config.hostaddr(ip);
                    }

                    _ => {
                        config.host(toml.host);
                    }
                }
            }

            Err(err) => {
                log::error!("Failed to parse `{}`. {err}", toml_config_path.display());
            }
        },

        Err(err) if err.kind() == io::ErrorKind::NotFound => {}

        Err(err) => {
            log::error!("Failed to read `{}`. {err}", toml_config_path.display());
        }
    }

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

#[derive(Debug, Deserialize)]
struct TomlConfig {
    host: String,
    user: String,
    password: String,
}
