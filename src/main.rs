use anyhow::{Context, Result};
use config::Config;
use std::env;
use tokio::net::TcpStream;

pub mod client;
pub mod config;
pub mod proto;
pub mod server;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // TODO replace stderr with /tmp/ra-mux.PID.log
    let config = Config::load_or_default().await;

    let mut server_args = Vec::new();
    let mut server_path = env::var("MUX_SERVER_PATH").ok();

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "---server-path" => {
                if let Some(set_server_path) = args.next() {
                    server_path = Some(set_server_path);
                }
            }
            "--version" => {
                // print version and exit instead of trying to connect to a server
                // see <https://github.com/pr2502/ra-multiplex/issues/4>
                println!("ra-multiplex {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--help" => {
                println!(
                    "\
ra-multiplex
    Multiplexing proxy for rust-analyzer (and other LSP servers)

USAGE:
    ra-mux [OPTIONS]

OPTIONS:
    --server-path PATH      Override the default rust-analyzer server path.
                            Overrides the MUX_SERVER_PATH environment variable.
    --version               Print the ra-mux version and exit.
    --help                  Print this help message and exit.

    Any option not recognized as a ra-mux option will be passed to the rust-analyzer server if
    ra-mux starts in client mode.
"
                );
                return Ok(());
            }
            _ => {
                // pass any unknown arguments along to the server
                server_args.push(arg);
            }
        }
    }

    // try starting in client mode
    match TcpStream::connect(config.connect).await {
        Ok(stream) => {
            client::client_mode(stream, server_path, server_args)
                .await
                .context("client mode")?;
            return Ok(()); // exit
        }
        Err(err) => {
            log::error!("unable to connect to a server, starting in server mode {err}");
        }
    }

    // TODO daemonize (will it work in tokio::main ?

    Ok(())
}
