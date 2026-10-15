use std::{
    fmt,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use axum::{Json, Router, extract::Query};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Task, app};

pub struct Server {
    pub bind_address: SocketAddr,
    pub api: Api,
}

impl Server {
    pub fn new() -> Self {
        let mut server = Self {
            bind_address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 80)),
            api: Api::new(),
        };

        server.api.add_query("firmament/isUp", async |_: ()| true);

        server
    }

    pub async fn start(self) -> app::Result<Task> {
        let listener = tokio::net::TcpListener::bind(self.bind_address).await?;

        Ok(Task::new(async move {
            axum::serve(listener, self.api.router.unwrap())
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
