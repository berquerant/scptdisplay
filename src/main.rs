use anyhow::{anyhow, Error};
use clap::Parser;
use log::debug;
use scptdisplay::cli::{Cli, Commands};
use scptdisplay::mcp;
use scptdisplay::response::Response;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Cli::parse();

    match &args.command {
        Commands::Mcp => {
            if let Err(e) = mcp::run_mcp_server().await {
                eprintln!("MCP server error: {e}");
                process::exit(1);
            }
        }
        _ => {
            let cli_cmd = args.cmd();

            debug!("args: {args:?}");
            debug!("cmd: {cli_cmd:?}");

            let cmd = process::Command::from(cli_cmd)
                .stdin(process::Stdio::null())
                .stdout(process::Stdio::piped())
                .stderr(process::Stdio::piped())
                .spawn();

            let result: Response = match cmd {
                Err(err) => Error::new(err).into(),
                Ok(child) => match child.wait_with_output() {
                    Err(err) => Error::new(err).into(),
                    Ok(x) => {
                        if x.status.success() {
                            Response::from(args.parse_stdout(x.stdout))
                        } else {
                            let code = x.status.code();
                            match String::from_utf8(x.stderr) {
                                Ok(x) => Response::from_err_with_code(anyhow!(x), code),
                                Err(x) => Response::from_err_with_code(Error::new(x), code),
                            }
                        }
                    }
                },
            };

            println!("{}", serde_json::to_string(&result).unwrap());
            if result.is_err() {
                process::exit(1);
            }
        }
    }
}
