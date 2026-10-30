use super::args::Commands;
use crate::parse::Output;
use crate::response::{Data, Response};
use anyhow::{Error, Result};

impl Commands {
    pub fn parse_stdout(&self, output: Vec<u8>) -> Result<Data> {
        match self {
            Commands::Mcp => Ok(Data::Notification {}),
            Commands::Notification { .. } => Ok(Data::Notification {}),
            Commands::Dialog { .. } => match String::from_utf8(output) {
                Err(err) => Err(Error::new(err)),
                Ok(x) => {
                    let r = Output::record(&x);
                    let text = r.get("text returned").cloned();
                    let button = r.get("button returned").cloned();
                    let gave_up = match r.get("gave up") {
                        None => false,
                        Some(x) => x == "true",
                    };
                    Ok(Data::Dialog {
                        raw: x,
                        record: r,
                        text,
                        button,
                        gave_up,
                    })
                }
            },
            Commands::Alert { .. } => match String::from_utf8(output) {
                Err(err) => Err(Error::new(err)),
                Ok(x) => {
                    let r = Output::record(&x);
                    let button = r.get("button returned").cloned();
                    let gave_up = match r.get("gave up") {
                        None => false,
                        Some(x) => x == "true",
                    };
                    Ok(Data::Alert {
                        raw: x,
                        record: r,
                        button,
                        gave_up,
                    })
                }
            },
        }
    }

    fn execute_process(&self, osascript: &str) -> std::io::Result<std::process::Output> {
        let script = String::from(self.cmd());
        log::debug!(
            "osascript: {osascript}, script: {}",
            self.script_for_logging()
        );

        let mut child = std::process::Command::new(osascript)
            .arg("-s")
            .arg("s")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(script.as_bytes());
        }

        child.wait_with_output()
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Commands::Alert { buttons, .. } if buttons.len() > 3 => {
                anyhow::bail!(
                    "Alert dialog supports at most 3 buttons, but {} were provided",
                    buttons.len()
                );
            }
            Commands::Dialog { buttons, .. } if buttons.len() > 3 => {
                anyhow::bail!(
                    "Dialog supports at most 3 buttons, but {} were provided",
                    buttons.len()
                );
            }
            _ => {}
        }
        Ok(())
    }

    pub fn execute(&self, osascript: &str) -> Result<Data> {
        if matches!(self, Commands::Mcp) {
            anyhow::bail!("Cannot execute osascript for Mcp command");
        }
        self.validate()?;
        let output = self.execute_process(osascript)?;
        if output.status.success() {
            self.parse_stdout(output.stdout)
        } else {
            let msg = String::from_utf8(output.stderr).unwrap_or_else(|e| e.to_string());
            anyhow::bail!("{msg}")
        }
    }

    pub fn execute_response(&self, osascript: &str) -> Response {
        if matches!(self, Commands::Mcp) {
            return anyhow::anyhow!("Cannot execute osascript for Mcp command").into();
        }
        if let Err(err) = self.validate() {
            return err.into();
        }
        match self.execute_process(osascript) {
            Err(err) => Error::new(err).into(),
            Ok(output) => {
                if output.status.success() {
                    Response::from(self.parse_stdout(output.stdout))
                } else {
                    let code = output.status.code();
                    match String::from_utf8(output.stderr) {
                        Ok(msg) => Response::from_err_with_code(anyhow::anyhow!(msg), code),
                        Err(err) => Response::from_err_with_code(Error::new(err), code),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::AlertType;
    use std::collections::HashMap;

    #[test]
    fn test_parse_stdout_alert_source_format() {
        let cmd = Commands::Alert {
            text: "hi".to_string(),
            message: None,
            alert_type: AlertType::Informational,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        let raw = b"{button returned:\"OK\", gave up:false}".to_vec();
        let data = cmd.parse_stdout(raw).unwrap();
        assert_eq!(
            data,
            Data::Alert {
                raw: "{button returned:\"OK\", gave up:false}".to_string(),
                record: HashMap::from([
                    ("button returned".to_string(), "OK".to_string()),
                    ("gave up".to_string(), "false".to_string()),
                ]),
                button: Some("OK".to_string()),
                gave_up: false,
            }
        );
    }

    #[test]
    fn test_parse_stdout_dialog_source_format_spoofing_defense() {
        let cmd = Commands::Dialog {
            text: "hi".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let raw =
            b"{button returned:\"Cancel\", text returned:\"evil, button returned:OK, gave up:true\", gave up:false}"
                .to_vec();
        let data = cmd.parse_stdout(raw).unwrap();
        assert_eq!(
            data,
            Data::Dialog {
                raw: "{button returned:\"Cancel\", text returned:\"evil, button returned:OK, gave up:true\", gave up:false}".to_string(),
                record: HashMap::from([
                    ("button returned".to_string(), "Cancel".to_string()),
                    (
                        "text returned".to_string(),
                        "evil, button returned:OK, gave up:true".to_string()
                    ),
                    ("gave up".to_string(), "false".to_string()),
                ]),
                text: Some("evil, button returned:OK, gave up:true".to_string()),
                button: Some("Cancel".to_string()),
                gave_up: false,
            }
        );
    }

    #[test]
    fn test_parse_stdout_notification() {
        let cmd = Commands::Notification {
            text: Some("hi".to_string()),
            title: None,
            subtitle: None,
            sound_name: None,
        };
        assert_eq!(
            cmd.parse_stdout(b"".to_vec()).unwrap(),
            Data::Notification {}
        );
    }

    #[test]
    fn test_parse_stdout_alert() {
        let cmd = Commands::Alert {
            text: "hi".to_string(),
            message: None,
            alert_type: AlertType::Informational,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        let raw = b"button returned:OK, gave up:false".to_vec();
        let data = cmd.parse_stdout(raw).unwrap();
        assert_eq!(
            data,
            Data::Alert {
                raw: "button returned:OK, gave up:false".to_string(),
                record: HashMap::from([
                    ("button returned".to_string(), "OK".to_string()),
                    ("gave up".to_string(), "false".to_string()),
                ]),
                button: Some("OK".to_string()),
                gave_up: false,
            }
        );
    }

    #[test]
    fn test_parse_stdout_dialog() {
        let cmd = Commands::Dialog {
            text: "hi".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let raw = b"button returned:OK, text returned:hello, world, gave up:false".to_vec();
        let data = cmd.parse_stdout(raw).unwrap();
        assert_eq!(
            data,
            Data::Dialog {
                raw: "button returned:OK, text returned:hello, world, gave up:false".to_string(),
                record: HashMap::from([
                    ("button returned".to_string(), "OK".to_string()),
                    ("text returned".to_string(), "hello, world".to_string()),
                    ("gave up".to_string(), "false".to_string()),
                ]),
                text: Some("hello, world".to_string()),
                button: Some("OK".to_string()),
                gave_up: false,
            }
        );
    }

    #[test]
    fn test_parse_stdout_dialog_gave_up() {
        let cmd = Commands::Dialog {
            text: "hi".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let raw = b"button returned:, text returned:, gave up:true".to_vec();
        let data = cmd.parse_stdout(raw).unwrap();
        assert_eq!(
            data,
            Data::Dialog {
                raw: "button returned:, text returned:, gave up:true".to_string(),
                record: HashMap::from([
                    ("button returned".to_string(), "".to_string()),
                    ("text returned".to_string(), "".to_string()),
                    ("gave up".to_string(), "true".to_string()),
                ]),
                text: Some("".to_string()),
                button: Some("".to_string()),
                gave_up: true,
            }
        );
    }

    #[test]
    fn test_parse_stdout_invalid_utf8() {
        let cmd = Commands::Alert {
            text: "hi".to_string(),
            message: None,
            alert_type: AlertType::Informational,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        assert!(cmd.parse_stdout(vec![0xff, 0xfe]).is_err());
    }

    #[test]
    fn test_execute_response_success_mock() {
        let cmd = Commands::Notification {
            text: Some("hi".to_string()),
            title: None,
            subtitle: None,
            sound_name: None,
        };
        let resp = cmd.execute_response("true");
        assert!(!resp.is_err());
    }

    #[test]
    fn test_execute_response_failure_mock() {
        let cmd = Commands::Notification {
            text: Some("hi".to_string()),
            title: None,
            subtitle: None,
            sound_name: None,
        };
        let resp = cmd.execute_response("false");
        assert!(resp.is_err());
    }

    #[test]
    fn test_execute_response_mcp_rejected() {
        let cmd = Commands::Mcp;
        let resp = cmd.execute_response("true");
        assert!(resp.is_err());
    }

    #[test]
    fn test_alert_validate_too_many_buttons() {
        let cmd = Commands::Alert {
            text: "test".to_string(),
            message: None,
            alert_type: AlertType::Informational,
            buttons: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        assert!(cmd.validate().is_err());
        assert!(cmd.execute_response("true").is_err());
    }

    #[test]
    fn test_dialog_validate_too_many_buttons() {
        let cmd = Commands::Dialog {
            text: "test".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec!["1".into(), "2".into(), "3".into(), "4".into()],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        assert!(cmd.validate().is_err());
        assert!(cmd.execute_response("true").is_err());
    }
}
