use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;

use act_zero::*;
use async_trait::async_trait;
use futures::executor::LocalPool;
use tokio::process::{Child, Command};
use tokio::stream;
use tokio_util::codec::length_delimited;

#[derive(Default)]
struct StreamActor {
    addr: WeakAddr<Self>,
    child: Option<Child>,
}

impl StreamActor {}

#[async_trait]
impl Actor for StreamActor {
    async fn started(&mut self, addr: Addr<Self>) -> ActorResult<()> {
        println!("Hello, world!");
        self.addr = addr.downgrade();
        let mut cmd = Command::new("/bin/bash");
        cmd.arg("src/stream.bash");
        cmd.stdout(Stdio::piped());

        let mut child = cmd.spawn().expect("Failed to launch");
        let stdout = child
            .stdout
            .take()
            .expect("Failed to open stdout on child process.");
        let framed_stream = length_delimited::Builder::new().new_read(stdout);

        Ok(act_zero::Produces::Value(()))
    }
}

#[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    let mut pool = LocalPool::new();
    let spawner = pool.spawner();
    pool.run_until(async move {
        println!("Starting actor!");
        let actor_ref = Addr::new(&spawner, StreamActor::default())?;
        actor_ref.termination().await;
        Ok(())
    })
}
