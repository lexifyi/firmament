use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    path::PathBuf,
};

use axum::{Json, Router, extract::Query};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Task, app};

pub struct Server {
    pub api: Api,
    pub listen_address: SocketAddr,
    pub public_dir: Option<PathBuf>,
}

impl Server {
    pub fn new() -> Self {
        let mut server = Self {
            api: Api::new(),
            listen_address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 80)),
            public_dir: None,
        };

        server.api.add_query("firmament/isUp", async |_: ()| true);

        server
    }

    pub fn set_listen_ip(&mut self, ip: impl Into<IpAddr>) {
        self.listen_address.set_ip(ip.into());
    }

    pub fn set_listen_port(&mut self, port: u16) {
        self.listen_address.set_port(port);
    }

    pub fn set_public_dir(&mut self, path: impl Into<PathBuf>) {
        self.public_dir = Some(path.into());
    }

    pub async fn start(self) -> app::Result<Task> {
        let mut service = self.api.router.unwrap();

        if let Some(public_dir) = self.public_dir {
            service = service.fallback_service(
                tower_http::services::ServeDir::new(public_dir).precompressed_br(),
            );
        }

        let listener = tokio::net::TcpListener::bind(self.listen_address).await?;
        let addr = listener.local_addr()?;

        log::info!("Listening on {}…", addr);

        Ok(Task::new(async move {
            axum::serve(listener, service)
                .await
                .expect("unexpected error from axum::serve");
        }))
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Api {
    router: Option<Router>,
}

impl Api {
    pub fn new() -> Self {
        Self {
            router: Some(Router::new()),
        }
    }

    pub fn add_call<I, O, F>(
        &mut self,
        name: impl fmt::Display,
        handler: impl FnOnce(I) -> F + Clone + Send + Sync + 'static,
    ) where
        I: DeserializeOwned + Send + 'static,
        O: Serialize,
        F: Future<Output = O> + Send,
    {
        let path = format!("/api/{}", name);
        let router = self.router.take().unwrap();

        self.router = Some(router.route(
            &path,
            axum::routing::post(async move |body: Json<I>| -> Json<O> {
                Json(handler(body.0).await)
            }),
        ));
    }

    pub fn add_query<I, O, F>(
        &mut self,
        name: impl fmt::Display,
        handler: impl FnOnce(I) -> F + Clone + Send + Sync + 'static,
    ) where
        I: DeserializeOwned + Send + 'static,
        O: Serialize,
        F: Future<Output = O> + Send,
    {
        let path = format!("/api/{}", name);
        let router = self.router.take().unwrap();

        self.router = Some(router.route(
            &path,
            axum::routing::get(async move |query: Query<I>| -> Json<O> {
                Json(handler(query.0).await)
            }),
        ));
    }

    pub fn include(&mut self, other: Api) {
        let router = self.router.take().unwrap();
        let other = other.router.unwrap();

        self.router = Some(router.merge(other));
    }
}

impl Default for Api {
    fn default() -> Self {
        Self::new()
    }
}
