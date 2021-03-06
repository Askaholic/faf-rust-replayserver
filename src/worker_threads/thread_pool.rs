use tokio_util::sync::CancellationToken;

use crate::server::connection::Connection;
use super::worker::{ReplayWorkerThread, ThreadFn};

pub struct ReplayThreadPool
{
    replay_workers: Vec<ReplayWorkerThread>,
}

impl ReplayThreadPool {
    pub fn new(work: ThreadFn, count: u32, shutdown_token: CancellationToken) -> Self {
        let mut replay_workers = Vec::new();
        for _ in 0..count {
            let worker = ReplayWorkerThread::new(work, shutdown_token.clone());
            replay_workers.push(worker);
        }
        Self { replay_workers }
    }

    /* Cancellable. */
    pub async fn assign_connection(&self, conn: Connection) {
        let conn_info = conn.get_header();
        let worker_to_pick = (conn_info.id % self.replay_workers.len() as u64) as usize;
        self.replay_workers[worker_to_pick].dispatch(conn).await;
    }
}
