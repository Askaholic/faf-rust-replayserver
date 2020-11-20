use std::error::Error;
use stop_token::StopSource;
use futures::{
    task::LocalSpawnExt,
    executor::LocalPool,
    join,
};

pub mod server;
pub mod config;

use crate::server::server::Server;
use crate::config::Config;
use crate::server::signal::hold_until_signal;

async fn run_server() {
    let config = Config { worker_threads: 4, port: "7878".to_string() };
    let server = Server::new(&config);
    let shutdown = StopSource::new();
    let token = shutdown.stop_token();
    let f1 = server.accept_until(token);
    let f2 = hold_until_signal(shutdown);
    join!(f1, f2);
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut pool = LocalPool::new();
    pool.spawner().spawn_local(run_server()).unwrap();
    pool.run();
    Ok(())
}
