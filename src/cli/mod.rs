pub(crate) mod args;
pub(crate) mod exec;
pub(crate) mod origin;
pub(crate) mod script;

pub use args::{AlertType, Cli, Commands};

use crate::cmd::Cmd;
use crate::response::{Data, Response};
use anyhow::Result;

impl Cli {
    pub fn cmd(&self) -> Cmd {
        let mut c = Cmd::new(&self.osascript);
        c.arg(Some("-s"));
        c.arg(Some("s"));
        c
    }
    pub fn script(&self) -> Cmd {
        self.command.cmd()
    }
    pub fn parse_stdout(&self, output: Vec<u8>) -> Result<Data> {
        self.command.parse_stdout(output)
    }
    pub fn execute(&self) -> Response {
        self.command.execute_response(&self.osascript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_cmd_wrapper() {
        let cli = Cli {
            osascript: "osascript".to_string(),
            command: Commands::Notification {
                text: Some("hi".to_string()),
                title: None,
                subtitle: None,
                sound_name: None,
            },
        };
        let c = cli.cmd();
        assert_eq!(String::from(c), "osascript -s s");
        let s = cli.script();
        assert_eq!(
            String::from(s),
            "display notification \"hi\" subtitle \"via scptdisplay\""
        );
    }
}
