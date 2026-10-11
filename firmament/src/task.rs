pub struct Task<T> {
    inner: Option<tokio::task::JoinHandle<T>>,
}

impl<T> Task<T> {
    #[must_use = "Tasks are aborted when dropped. Call .detach() to run a task in the background."]
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        Self {
            inner: Some(tokio::task::spawn(future)),
        }
    }

    #[inline]
    pub fn abort(self) {
        drop(self);
    }

    #[inline]
    pub fn detach(mut self) {
        self.inner = None;
    }

    #[inline]
    pub async fn join(mut self) -> T {
        match self.inner.take().unwrap().await {
            Ok(out) => out,
            Err(err) => panic!(
                "{}",
                match err.is_cancelled() {
                    true => "task aborted",
                    false => "task panicked",
                }
            ),
        }
    }
}

impl<T> Drop for Task<T> {
    #[inline]
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            inner.abort();
        }
    }
}
