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

#[cfg(test)]
mod tests {
    use super::*;

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
}
