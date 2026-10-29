use crate::cmd::Cmd;
use crate::parse::{Input, Output};
use crate::response::{Data, Response};
use anyhow::{Error, Result};
use clap::{self, Parser, Subcommand, ValueEnum};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, JsonSchema)]
pub enum AlertType {
    #[value(name = "informational")]
    #[serde(rename = "informational")]
    Informational,
    #[value(name = "warning")]
    #[serde(rename = "warning")]
    Warning,
    #[value(name = "critical")]
    #[serde(rename = "critical")]
    Critical,
}

impl AlertType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Informational => "informational",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

/// Display a notification, dialog or alert via AppleScript.
///
/// Requirements:
/// - osascript
///
/// Environment variables:
///   RUST_LOG
///     log level.
///     see https://docs.rs/env_logger/latest/env_logger/
#[derive(Debug, Parser)]
#[command(name = "scptdisplay")]
#[command(version, about)]
pub struct Cli {
    /// osascript command.
    #[arg(long = "osascript", default_value = "/usr/bin/osascript")]
    pub osascript: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Posts a notification using the Notification Center, containing a title, subtitle, and explanation, and optionally playing a sound.
    ///
    /// Output:
    ///   A json to stdout.
    ///     result(string): success for ok, failure for error.
    ///     code(int or null): exit status of invoked process.
    ///     error(string or null): stderr of invoked process or null if result is ok.
    ///     data(map or null): null if result is error.
    ///       notification(empty map): empty map.
    ///
    /// Exit status
    ///   0 successfully processed.
    ///   1 failed to process.
    ///
    /// See https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW224
    #[command(about, verbatim_doc_comment, arg_required_else_help = true, visible_aliases = &["n", "notify"])]
    Notification {
        /// The body text of the notification.
        /// At least one of this and the title must be specified.
        #[arg(
            value_name = "TEXT",
            required_unless_present = "title",
            verbatim_doc_comment
        )]
        text: Option<String>,
        /// The title of the notification.
        /// At least one of this and the body text must be specified.
        #[arg(
            short = 't',
            long = "title",
            required_unless_present = "text",
            verbatim_doc_comment
        )]
        title: Option<String>,
        /// The subtitle of the notification.
        #[arg(short = 's', long = "subtitle", verbatim_doc_comment)]
        subtitle: Option<String>,
        /// The name of a sound to play when the notification appears.
        /// This may be the base name of any sound installed in Library/Sounds.
        #[arg(long = "sound", verbatim_doc_comment)]
        sound_name: Option<String>,
    },
    /// Displays a standardized alert containing a message, explanation, and from one to three buttons.
    ///
    /// Output:
    ///   A json to stdout.
    ///     result(string): success for ok, failure for error.
    ///     code(int or null): exit status of invoked process.
    ///     error(string or null): stderr of invoked process or null if result is ok.
    ///     data(map or null): null if result is error.
    ///       alert(map):
    ///         raw(string): raw stdout.
    ///         record(map(string to string)): parsed stdout.
    ///         button(string or null): button returned.
    ///         gave_up(bool): if true, no button was returned and the command gave up.
    ///
    /// Exit status
    ///   0 successfully processed.
    ///   1 failed to process.
    ///
    /// Caution:
    ///   If you specify a string containing characters such as commas or colons in default_answer or buttons, it may not correctly identify the selected button or text.
    ///
    /// See https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW11
    #[command(about, verbatim_doc_comment, arg_required_else_help = true, visible_aliases = &["a"])]
    Alert {
        /// The alert text, which is displayed in emphasized system font.
        #[arg(value_name = "TEXT", num_args = 1, value_parser = clap::value_parser!(String), verbatim_doc_comment)]
        text: String,
        /// An explanatory message, which is displayed in small system font, below the alert text.
        #[arg(long = "message", verbatim_doc_comment)]
        message: Option<String>,
        /// The type of alert to show. You can specify one of the following alert types:
        ///   informational: the standard alert dialog
        ///   warning: the alert dialog dialog is badged with a warning icon
        ///   critical: currently the same as the standard alert dialog
        #[arg(
            short = 't',
            long = "as",
            default_value = "informational",
            value_enum,
            verbatim_doc_comment
        )]
        alert_type: AlertType,
        /// A list of up to three button names.
        /// If you supply one name, a button with that name serves as the default and is displayed on the right side of the alert dialog.
        /// If you supply two names, two buttons are displayed on the right, with the second serving as the default button.
        /// If you supply three names, the first is displayed on the left, and the next two on the right, as in the case with two buttons.
        /// Default:
        ///   {"OK"}: One button labeled “OK”, which is the default button.
        #[arg(long = "buttons", verbatim_doc_comment)]
        buttons: Vec<String>,
        /// The name or number of the default button.
        /// This may be the same as the cancel button.
        /// Default:
        ///   The rightmost button.
        #[arg(long = "default_button", verbatim_doc_comment)]
        default_button: Option<String>,
        /// The name or number of the cancel button.
        /// Default:
        ///   None; there is no cancel button.
        #[arg(long = "cancel_button", verbatim_doc_comment)]
        cancel_button: Option<String>,
        /// The number of seconds to wait before automatically dismissing the alert.
        /// Default:
        ///   None; the dialog will wait until the user clicks a button.
        #[arg(short = 'g', long = "giving_up_after", verbatim_doc_comment)]
        giving_up_after: Option<u8>,
    },
    /// Displays a dialog containing a message, one to three buttons, and optionally an icon and a ﬁeld in which the user can enter text.
    ///
    /// Output:
    ///   A json to stdout.
    ///     result(string): success for ok, failure for error.
    ///     code(int or null): exit status of invoked process.
    ///     error(string or null): stderr of invoked process or null if result is ok.
    ///     data(map or null): null if result is error.
    ///       dialog(map):
    ///         raw(string): raw stdout.
    ///         record(map(string to string)): parsed stdout.
    ///         text(string or null): text returned.
    ///         button(string or null): button returned.
    ///         gave_up(bool): if true, no button was returned and the command gave up.
    ///
    /// Exit status
    ///   0 successfully processed.
    ///   1 failed to process.
    ///
    /// Caution:
    ///   If you specify a string containing characters such as commas or colons in default_answer or buttons, it may not correctly identify the selected button or text.
    ///
    /// See https://developer.apple.com/library/archive/documentation/AppleScript/Conceptual/AppleScriptLangGuide/reference/ASLR_cmds.html#//apple_ref/doc/uid/TP40000983-CH216-SW12
    #[command(about, verbatim_doc_comment, arg_required_else_help = true, visible_aliases = &["d"])]
    Dialog {
        /// The dialog text, which is displayed in emphasized system font.
        #[arg(value_name = "TEXT", num_args = 1, value_parser = clap::value_parser!(String), verbatim_doc_comment)]
        text: String,
        /// The initial contents of an editable text field.
        /// This edit field is not present unless this parameter is present; to have the field present but blank, specify an empty string.
        /// Default:
        ///   None; there is no edit field.
        #[arg(long = "default_answer", verbatim_doc_comment)]
        default_answer: Option<String>,
        /// If true, any text in the edit field is obscured as in a password dialog: each character is displayed as a bullet.
        /// Default:
        ///   false: text in the edit field is shown in cleartext.
        #[arg(long = "hidden_answer", verbatim_doc_comment)]
        hidden_answer: bool,
        /// A list of up to three button names.
        /// Default:
        ///   If you don’t specify any buttons, by default, Cancel and OK buttons are shown, with the OK button set as the default button.
        ///   If you specify any buttons, there is no default or cancel button unless you use the following parameters to specify them.
        #[arg(long = "buttons", verbatim_doc_comment)]
        buttons: Vec<String>,
        /// The name or number of the default button. This button is highlighted, and will be pressed if the user presses the Return or Enter key.
        /// Default:
        ///   If there are no buttons specified using buttons, the OK button. Otherwise, there is no default button.
        #[arg(long = "default_button", verbatim_doc_comment)]
        default_button: Option<String>,
        /// The name or number of the cancel button. This button will be pressed if the user presses the Escape key or Command-period.
        /// Default:
        ///   If there are no buttons specified using buttons, the Cancel button. Otherwise, there is no cancel button.
        #[arg(long = "cancel_button", verbatim_doc_comment)]
        cancel_button: Option<String>,
        /// The dialog window title.
        /// Default:
        ///   None; no title is displayed.
        #[arg(short = 't', long = "title", verbatim_doc_comment)]
        title: Option<String>,
        /// The resource name or ID of the icon to display.
        /// The type of icon to show. You may specify one of the following constants:
        ///   stop (or 0): Shows a stop icon
        ///   note (or 1): Shows the application icon
        ///   caution (or 2): Shows a warning icon, badged with the application icon
        /// An alias or file specifier that specifies a .icns file.
        #[arg(long = "icon", verbatim_doc_comment)]
        icon: Option<String>,
        /// The number of seconds to wait before automatically dismissing the dialog.
        /// Default:
        ///   None; the dialog will wait until the user presses a button.
        #[arg(short = 'g', long = "giving_up_after", verbatim_doc_comment)]
        giving_up_after: Option<u8>,
    },
    /// Start Model Context Protocol (MCP) server over stdio.
    #[command(about)]
    Mcp,
}

impl fmt::Debug for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Commands::Notification {
                text,
                title,
                subtitle,
                sound_name,
            } => f
                .debug_struct("Notification")
                .field("text", text)
                .field("title", title)
                .field("subtitle", subtitle)
                .field("sound_name", sound_name)
                .finish(),
            Commands::Alert {
                text,
                message,
                alert_type,
                buttons,
                default_button,
                cancel_button,
                giving_up_after,
            } => f
                .debug_struct("Alert")
                .field("text", text)
                .field("message", message)
                .field("alert_type", alert_type)
                .field("buttons", buttons)
                .field("default_button", default_button)
                .field("cancel_button", cancel_button)
                .field("giving_up_after", giving_up_after)
                .finish(),
            Commands::Dialog {
                text,
                default_answer,
                hidden_answer,
                buttons,
                default_button,
                cancel_button,
                title,
                icon,
                giving_up_after,
            } => {
                let redacted_answer: Option<String> = if *hidden_answer && default_answer.is_some()
                {
                    Some("[REDACTED]".to_string())
                } else {
                    default_answer.clone()
                };
                f.debug_struct("Dialog")
                    .field("text", text)
                    .field("default_answer", &redacted_answer)
                    .field("hidden_answer", hidden_answer)
                    .field("buttons", buttons)
                    .field("default_button", default_button)
                    .field("cancel_button", cancel_button)
                    .field("title", title)
                    .field("icon", icon)
                    .field("giving_up_after", giving_up_after)
                    .finish()
            }
            Commands::Mcp => write!(f, "Mcp"),
        }
    }
}

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

impl Commands {
    fn cmd(&self) -> Cmd {
        match self {
            Commands::Notification {
                text,
                title,
                subtitle,
                sound_name,
            } => {
                let mut c = Cmd::new("display notification");
                let body = text.as_deref().unwrap_or("");
                c.arg(Some(Input::quoted(body)));
                c.pair("with title", title.as_deref().map(Input::quoted));
                c.pair("subtitle", subtitle.as_deref().map(Input::quoted));
                c.pair("sound name", sound_name.as_deref().map(Input::quoted));
                c
            }
            Commands::Alert {
                text,
                message,
                alert_type,
                buttons,
                default_button,
                cancel_button,
                giving_up_after,
            } => {
                let mut c = Cmd::new("display alert");
                c.arg(Some(Input::quoted(text)));
                c.pair("message", message.as_deref().map(Input::quoted));
                c.pair("as", Some(alert_type.as_str()));
                if !buttons.is_empty() {
                    c.pair("buttons", Some(Input::apple_script_list(buttons.clone())));
                }
                c.pair(
                    "default button",
                    default_button.as_deref().map(Input::integer_or_text),
                );
                c.pair(
                    "cancel button",
                    cancel_button.as_deref().map(Input::integer_or_text),
                );
                c.pair("giving up after", giving_up_after.map(|x| format!("{x}")));
                c
            }
            Commands::Dialog {
                text,
                default_answer,
                hidden_answer,
                buttons,
                default_button,
                cancel_button,
                title,
                icon,
                giving_up_after,
            } => {
                let mut c = Cmd::new("display dialog");
                c.arg(Some(Input::quoted(text)));
                c.pair(
                    "default answer",
                    default_answer.as_deref().map(Input::quoted),
                );
                c.arg(hidden_answer.then_some("hidden answer"));
                if !buttons.is_empty() {
                    c.pair("buttons", Some(Input::apple_script_list(buttons.clone())));
                }
                c.pair(
                    "default button",
                    default_button.as_deref().map(Input::integer_or_text),
                );
                c.pair(
                    "cancel button",
                    cancel_button.as_deref().map(Input::integer_or_text),
                );
                c.pair("with title", title.as_deref().map(Input::quoted));
                c.pair("with icon", icon.as_deref().map(Input::icon));
                c.pair("giving up after", giving_up_after.map(|x| format!("{x}")));
                c
            }
            Commands::Mcp => Cmd::new("osascript"),
        }
    }
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

    fn script_for_logging(&self) -> String {
        match self {
            Commands::Dialog {
                text,
                default_answer,
                hidden_answer: true,
                buttons,
                default_button,
                cancel_button,
                title,
                icon,
                giving_up_after,
            } if default_answer.is_some() => {
                let mut c = Cmd::new("display dialog");
                c.arg(Some(Input::quoted(text)));
                c.pair("default answer", Some(Input::quoted("[REDACTED]")));
                c.arg(Some("hidden answer"));
                if !buttons.is_empty() {
                    c.pair("buttons", Some(Input::apple_script_list(buttons.clone())));
                }
                c.pair(
                    "default button",
                    default_button.as_deref().map(Input::integer_or_text),
                );
                c.pair(
                    "cancel button",
                    cancel_button.as_deref().map(Input::integer_or_text),
                );
                c.pair("with title", title.as_deref().map(Input::quoted));
                c.pair("with icon", icon.as_deref().map(Input::icon));
                c.pair("giving up after", giving_up_after.map(|x| format!("{x}")));
                String::from(c)
            }
            _ => String::from(self.cmd()),
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
    use std::collections::HashMap;

    #[test]
    fn test_notification_cmd_full() {
        let cmd = Commands::Notification {
            text: Some("body".to_string()),
            title: Some("title".to_string()),
            subtitle: Some("subtitle".to_string()),
            sound_name: Some("Ping".to_string()),
        };
        let c = cmd.cmd();
        assert_eq!(
            String::from(c),
            "display notification \"body\" with title \"title\" subtitle \"subtitle\" sound name \"Ping\""
        );
    }

    #[test]
    fn test_notification_cmd_title_only() {
        let cmd = Commands::Notification {
            text: None,
            title: Some("title".to_string()),
            subtitle: None,
            sound_name: None,
        };
        let c = cmd.cmd();
        assert_eq!(
            String::from(c),
            "display notification \"\" with title \"title\""
        );
    }

    #[test]
    fn test_alert_cmd_full() {
        let cmd = Commands::Alert {
            text: "alert text".to_string(),
            message: Some("alert message".to_string()),
            alert_type: AlertType::Warning,
            buttons: vec!["Cancel".to_string(), "OK".to_string()],
            default_button: Some("2".to_string()),
            cancel_button: Some("1".to_string()),
            giving_up_after: Some(10),
        };
        let c = cmd.cmd();
        assert_eq!(
            String::from(c),
            "display alert \"alert text\" message \"alert message\" as warning buttons {\"Cancel\",\"OK\"} default button 2 cancel button 1 giving up after 10"
        );
    }

    #[test]
    fn test_dialog_cmd_full() {
        let cmd = Commands::Dialog {
            text: "enter name".to_string(),
            default_answer: Some("alice".to_string()),
            hidden_answer: true,
            buttons: vec!["OK".to_string()],
            default_button: Some("1".to_string()),
            cancel_button: None,
            title: Some("Login".to_string()),
            icon: Some("note".to_string()),
            giving_up_after: Some(30),
        };
        let c = cmd.cmd();
        assert_eq!(
            String::from(c),
            "display dialog \"enter name\" default answer \"alice\" hidden answer buttons {\"OK\"} default button 1 with title \"Login\" with icon note giving up after 30"
        );
    }

    #[test]
    fn test_dialog_cmd_icon_variants() {
        let make_cmd = |icon: Option<String>| Commands::Dialog {
            text: "test".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon,
            giving_up_after: None,
        };

        assert_eq!(
            String::from(make_cmd(Some("stop".to_string())).cmd()),
            "display dialog \"test\" with icon stop"
        );
        assert_eq!(
            String::from(make_cmd(Some("caution".to_string())).cmd()),
            "display dialog \"test\" with icon caution"
        );
        assert_eq!(
            String::from(make_cmd(Some("CAUTION".to_string())).cmd()),
            "display dialog \"test\" with icon caution"
        );
        assert_eq!(
            String::from(make_cmd(Some("1".to_string())).cmd()),
            "display dialog \"test\" with icon 1"
        );
        assert_eq!(
            String::from(make_cmd(Some("app.icns".to_string())).cmd()),
            "display dialog \"test\" with icon \"app.icns\""
        );
    }

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
        assert_eq!(String::from(s), "display notification \"hi\"");
    }

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
    fn test_commands_dialog_debug_redaction() {
        let cmd = Commands::Dialog {
            text: "prompt".to_string(),
            default_answer: Some("super_secret_password".to_string()),
            hidden_answer: true,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let debug_str = format!("{cmd:?}");
        assert!(!debug_str.contains("super_secret_password"));
        assert!(debug_str.contains("[REDACTED]"));

        let non_hidden = Commands::Dialog {
            text: "prompt".to_string(),
            default_answer: Some("public_value".to_string()),
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let non_hidden_debug = format!("{non_hidden:?}");
        assert!(non_hidden_debug.contains("public_value"));
    }

    #[test]
    fn test_commands_dialog_script_for_logging_redaction() {
        let cmd = Commands::Dialog {
            text: "prompt".to_string(),
            default_answer: Some("super_secret_password".to_string()),
            hidden_answer: true,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let script_log = cmd.script_for_logging();
        assert!(!script_log.contains("super_secret_password"));
        assert!(script_log.contains("[REDACTED]"));

        // Ensure the actual script still contains the real password
        let actual_script = String::from(cmd.cmd());
        assert!(actual_script.contains("super_secret_password"));
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
