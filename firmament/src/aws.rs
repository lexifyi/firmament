pub use aws_config::SdkConfig as Config;
use tokio::sync::{OnceCell, SetOnce};

pub async fn get_config() -> &'static Config {
    static CONFIG: OnceCell<Config> = OnceCell::const_new();

    async fn load_config() -> Config {
        aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await
    }

    CONFIG.get_or_init(load_config).await
}

#[cfg(feature = "email")]
pub mod ses {
    pub use aws_sdk_ses::*;
    use tokio::sync::OnceCell;

    pub async fn client() -> &'static Client {
        static CLIENT: OnceCell<Client> = OnceCell::const_new();

        async fn init() -> Client {
            Client::new(super::get_config().await)
        }

        CLIENT.get_or_init(init).await
    }
}
