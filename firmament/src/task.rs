use tokio::time::{Duration, Instant, sleep};

#[must_use = "Tasks are aborted when dropped. Call .detach() to run a task in the background."]
pub struct Task<T> {
    inner: Option<tokio::task::JoinHandle<T>>,
}

impl<T> Task<T> {
    #[inline]
    pub fn new(future: impl Future<Output = T> + Send + 'static) -> Self
    where
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

pub async fn repeat_every<F>(duration: Duration, mut func: impl FnMut() -> F) -> !
where
    F: Future<Output = ()>,
{
    loop {
        let start = Instant::now();

        func().await;

        let remaining = duration.saturating_sub(start.elapsed());

        if !remaining.is_zero() {
            sleep(remaining).await;
        }
    }
}
