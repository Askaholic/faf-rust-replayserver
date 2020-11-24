use async_std::sync::Receiver;
use stop_token::StopToken;

use crate::server::connection::Connection;

pub struct Replays {
    shutdown_token: StopToken,
    connections: Receiver<Connection>,
}

impl Replays {
    pub fn new(shutdown_token: StopToken,
               connections: Receiver<Connection>) -> Self
    {
        Replays {shutdown_token, connections}
    }

    pub async fn lifetime(&mut self) {
        while let Ok(conn) = self.connections.recv().await {
            let _ = conn;
        }
    }
}