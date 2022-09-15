use anyhow::{Context, Result};
use crate::proto;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::{io, task};

pub async fn client_mode(
    stream: TcpStream,
    server_path: Option<String>,
    server_args: Vec<String>,
) -> Result<()> {
    let server_path = match server_path {
        Some(p) => p,
        None => {
            // TODO "$(rustc --print sysroot)/rust-analyzer" and only default to plain
            // "rust-analyzer" if it fails
            "rust-analyzer".to_owned()
        },
    };

    let (mut read_stream, mut write_stream) = stream.into_split();

    { // protocol initialization
        let proto_init = proto::Init::new(server_path, server_args);
        let mut proto_init = serde_json::to_vec(&proto_init).context("sending proto init")?;
        proto_init.push(b'\0');

        write_stream
            .write_all(&proto_init)
            .await
            .context("sending proto init")?;
    }

    let input = task::spawn(async move {
        io::copy(&mut io::stdin(), &mut write_stream)
            .await
            .context("io error")
    });
    let output = task::spawn(async move {
        io::copy(&mut read_stream, &mut io::stdout())
            .await
            .context("io error")
    });
    tokio::select! {
        res = input => res,
        res = output => res,
    }
    .context("join")??;
    Ok(())
}
