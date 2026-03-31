use tokio::select;
use tokio::sync::watch;
use tokio::time::{Duration, Instant, sleep_until};

/// 给 watch::Receiver 增加“去抖接收”的能力
pub trait WatchDebounceExt<T> {
    /// 等到有变化后，静默 `duration`，期间若继续变化就重置计时；
    /// 静默期结束返回“最后一个值”。发送端全关返回 None。
    async fn recv_debounced<'a>(&'a mut self, duration: Duration) -> Result<watch::Ref<'a, T>, watch::error::RecvError>
    where
        T: 'a;

    /// 去抖 + 去重：与上次返回相同则继续等待（需要 PartialEq）
    async fn recv_debounced_distinct<'a, F>(
        &'a mut self,
        duration: Duration,
        accept: F,
    ) -> Result<watch::Ref<'a, T>, watch::error::RecvError>
    where
        T: 'a,
        F: FnMut(&watch::Ref<'a, T>) -> bool;
}

impl<T: Clone> WatchDebounceExt<T> for watch::Receiver<T> {
    async fn recv_debounced<'a>(&'a mut self, duration: Duration) -> Result<watch::Ref<'a, T>, watch::error::RecvError>
    where
        T: 'a,
    {
        // 等到第一次变化；若 sender 全关，返回 Err(Closed)
        self.changed().await?;
        let mut deadline = Instant::now() + duration;

        loop {
            select! {
                _ = async { self.changed().await.ok() } => {
                    deadline = Instant::now() + duration;
                }
                _ = sleep_until(deadline) => {
                    return Ok(self.borrow());
                }
            }
        }
    }

    async fn recv_debounced_distinct<'a, F>(
        &'a mut self,
        duration: Duration,
        mut accept: F,
    ) -> Result<watch::Ref<'a, T>, watch::error::RecvError>
    where
        T: 'a,
        F: FnMut(&watch::Ref<'a, T>) -> bool,
    {
        // 以当前快照为“上一次值”的基准
        let last = self.recv_debounced(duration).await?;
        loop {
            if accept(&last) {
                return Ok(last);
            }
        }
    }
}
