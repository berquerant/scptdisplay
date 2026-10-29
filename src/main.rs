use clap::Parser;
use log::debug;
use scptdisplay::cli::{Cli, Commands};
use scptdisplay::mcp;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Cli::parse();

    match &args.command {
        Commands::Mcp => {
            if let Err(e) = mcp::run_mcp_server_with_osascript(&args.osascript).await {
                eprintln!("MCP server error: {e}");
                process::exit(1);
            }
        }
        _ => {
            debug!("args: {args:?}");

            let result = args.execute();

            match serde_json::to_string(&result) {
                Ok(json) => println!("{json}"),
                Err(err) => {
                    eprintln!("Failed to serialize response: {err}");
                    process::exit(1);
                }
            }
            if result.is_err() {
                process::exit(1);
            }
        }
    }
}
