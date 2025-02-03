mod cli;
mod cmd;
mod parse;
mod response;
use crate::cli::Cli;
use crate::response::Response;
use anyhow::{anyhow, Error};
use clap::Parser;
use log::debug;
use std::env;
use std::process;

fn main() {
    env_logger::init();

    let args = Cli::parse();
    let cli_cmd = args.cmd();

    debug!("args: {:?}", args);
    debug!("cmd: {:?}", cli_cmd);

    let cmd = process::Command::from(cli_cmd)
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .env_clear()
        .envs(env::vars())
        .spawn();

    let result: Response = match cmd {
        Err(err) => Error::new(err).into(),
        Ok(cmd) => match cmd.wait_with_output() {
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
}
