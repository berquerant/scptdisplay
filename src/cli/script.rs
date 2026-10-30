use super::args::Commands;
use super::origin::{
    append_origin_tag, format_alert_message_origin, format_dialog_title_origin,
    format_notification_origin,
};
use crate::cmd::Cmd;
use crate::parse::Input;

impl Commands {
    pub(crate) fn cmd(&self) -> Cmd {
        match self {
            Commands::Notification {
                text,
                title,
                subtitle,
                sound_name,
            } => {
                let mut c = Cmd::new("display notification");
                let (body, subtitle_with_origin) =
                    format_notification_origin(text.as_deref(), subtitle.as_deref());
                c.arg(Some(Input::quoted(&body)));
                c.pair("with title", title.as_deref().map(Input::quoted));
                c.pair(
                    "subtitle",
                    subtitle_with_origin.as_deref().map(Input::quoted),
                );
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
                let message_with_origin = format_alert_message_origin(message.as_deref());
                c.pair("message", message_with_origin.as_deref().map(Input::quoted));
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
                let text_with_origin = append_origin_tag(text);
                let title_with_origin = format_dialog_title_origin(title.as_deref());
                c.arg(Some(Input::quoted(&text_with_origin)));
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
                c.pair(
                    "with title",
                    title_with_origin.as_deref().map(Input::quoted),
                );
                c.pair("with icon", icon.as_deref().map(Input::icon));
                c.pair("giving up after", giving_up_after.map(|x| format!("{x}")));
                c
            }
            Commands::Mcp => Cmd::new("osascript"),
        }
    }

    pub(crate) fn script_for_logging(&self) -> String {
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
                let text_with_origin = append_origin_tag(text);
                let title_with_origin = format_dialog_title_origin(title.as_deref());
                c.arg(Some(Input::quoted(&text_with_origin)));
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
                c.pair(
                    "with title",
                    title_with_origin.as_deref().map(Input::quoted),
                );
                c.pair("with icon", icon.as_deref().map(Input::icon));
                c.pair("giving up after", giving_up_after.map(|x| format!("{x}")));
                String::from(c)
            }
            _ => String::from(self.cmd()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::AlertType;

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
            "display notification \"body\n(via scptdisplay)\" with title \"title\" subtitle \"subtitle\" sound name \"Ping\""
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
            "display notification \"\" with title \"title\" subtitle \"via scptdisplay\""
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
            "display alert \"alert text\" message \"alert message\n(via scptdisplay)\" as warning buttons {\"Cancel\",\"OK\"} default button 2 cancel button 1 giving up after 10"
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
            "display dialog \"enter name\n(via scptdisplay)\" default answer \"alice\" hidden answer buttons {\"OK\"} default button 1 with title \"Login\" with icon note giving up after 30"
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
            "display dialog \"test\n(via scptdisplay)\" with title \"scptdisplay\" with icon stop"
        );
        assert_eq!(
            String::from(make_cmd(Some("caution".to_string())).cmd()),
            "display dialog \"test\n(via scptdisplay)\" with title \"scptdisplay\" with icon caution"
        );
        assert_eq!(
            String::from(make_cmd(Some("CAUTION".to_string())).cmd()),
            "display dialog \"test\n(via scptdisplay)\" with title \"scptdisplay\" with icon caution"
        );
        assert_eq!(
            String::from(make_cmd(Some("1".to_string())).cmd()),
            "display dialog \"test\n(via scptdisplay)\" with title \"scptdisplay\" with icon 1"
        );
        assert_eq!(
            String::from(make_cmd(Some("app.icns".to_string())).cmd()),
            "display dialog \"test\n(via scptdisplay)\" with title \"scptdisplay\" with icon \"app.icns\""
        );
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
    fn test_notification_origin_variations() {
        // Subtitle already contains scptdisplay
        let cmd = Commands::Notification {
            text: Some("hello".to_string()),
            title: Some("Title".to_string()),
            subtitle: Some("scptdisplay notification".to_string()),
            sound_name: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display notification \"hello\" with title \"Title\" subtitle \"scptdisplay notification\""
        );

        // Subtitle is None -> subtitle becomes via scptdisplay
        let cmd = Commands::Notification {
            text: Some("hello".to_string()),
            title: Some("Title".to_string()),
            subtitle: None,
            sound_name: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display notification \"hello\" with title \"Title\" subtitle \"via scptdisplay\""
        );

        // Subtitle is Some without scptdisplay -> body gets (via scptdisplay)
        let cmd = Commands::Notification {
            text: Some("hello".to_string()),
            title: None,
            subtitle: Some("custom sub".to_string()),
            sound_name: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display notification \"hello\n(via scptdisplay)\" subtitle \"custom sub\""
        );
    }

    #[test]
    fn test_alert_origin_variations() {
        // message is None -> message becomes (via scptdisplay)
        let cmd = Commands::Alert {
            text: "alert".to_string(),
            message: None,
            alert_type: AlertType::Informational,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display alert \"alert\" message \"(via scptdisplay)\" as informational"
        );

        // message already contains (via scptdisplay) -> not duplicated
        let cmd = Commands::Alert {
            text: "alert".to_string(),
            message: Some("existing message\n(via scptdisplay)".to_string()),
            alert_type: AlertType::Informational,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display alert \"alert\" message \"existing message\n(via scptdisplay)\" as informational"
        );

        // message with trailing newline is trimmed
        let cmd = Commands::Alert {
            text: "alert".to_string(),
            message: Some("message with newline\n".to_string()),
            alert_type: AlertType::Informational,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display alert \"alert\" message \"message with newline\n(via scptdisplay)\" as informational"
        );
    }

    #[test]
    fn test_dialog_origin_variations() {
        // title is None -> title becomes scptdisplay
        let cmd = Commands::Dialog {
            text: "dialog text".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display dialog \"dialog text\n(via scptdisplay)\" with title \"scptdisplay\""
        );

        // title is empty string -> title falls back to scptdisplay
        let cmd = Commands::Dialog {
            text: "dialog text".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: Some("".to_string()),
            icon: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display dialog \"dialog text\n(via scptdisplay)\" with title \"scptdisplay\""
        );

        // text with trailing newline is trimmed
        let cmd = Commands::Dialog {
            text: "dialog text\n".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display dialog \"dialog text\n(via scptdisplay)\" with title \"scptdisplay\""
        );

        // text already contains (via scptdisplay) -> not duplicated
        let cmd = Commands::Dialog {
            text: "dialog text\n(via scptdisplay)".to_string(),
            default_answer: None,
            hidden_answer: false,
            buttons: vec![],
            default_button: None,
            cancel_button: None,
            title: Some("Custom Title".to_string()),
            icon: None,
            giving_up_after: None,
        };
        assert_eq!(
            String::from(cmd.cmd()),
            "display dialog \"dialog text\n(via scptdisplay)\" with title \"Custom Title\""
        );
    }
}
