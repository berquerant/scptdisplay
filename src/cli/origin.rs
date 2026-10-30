pub(crate) const ORIGIN_TAG: &str = "(via scptdisplay)";
pub(crate) const ORIGIN_NAME: &str = "scptdisplay";

pub(crate) fn append_origin_tag(text: &str) -> String {
    if text.contains(ORIGIN_TAG) {
        text.to_string()
    } else if text.is_empty() {
        ORIGIN_TAG.to_string()
    } else {
        format!("{}\n{ORIGIN_TAG}", text.trim_end())
    }
}

pub(crate) fn format_notification_origin(
    text: Option<&str>,
    subtitle: Option<&str>,
) -> (String, Option<String>) {
    let body = text.unwrap_or("");
    match subtitle {
        None => (body.to_string(), Some(format!("via {ORIGIN_NAME}"))),
        Some(s) if s.contains(ORIGIN_NAME) => (body.to_string(), Some(s.to_string())),
        Some(s) => (append_origin_tag(body), Some(s.to_string())),
    }
}

pub(crate) fn format_alert_message_origin(message: Option<&str>) -> Option<String> {
    match message {
        None | Some("") => Some(ORIGIN_TAG.to_string()),
        Some(m) if m.contains(ORIGIN_TAG) => Some(m.to_string()),
        Some(m) => Some(format!("{}\n{ORIGIN_TAG}", m.trim_end())),
    }
}

pub(crate) fn format_dialog_title_origin(title: Option<&str>) -> Option<String> {
    match title {
        None | Some("") => Some(ORIGIN_NAME.to_string()),
        Some(t) => Some(t.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_origin_tag() {
        assert_eq!(append_origin_tag(""), "(via scptdisplay)");
        assert_eq!(append_origin_tag("hello"), "hello\n(via scptdisplay)");
        assert_eq!(append_origin_tag("hello\n"), "hello\n(via scptdisplay)");
        assert_eq!(
            append_origin_tag("hello\n(via scptdisplay)"),
            "hello\n(via scptdisplay)"
        );
    }

    #[test]
    fn test_format_notification_origin() {
        assert_eq!(
            format_notification_origin(None, None),
            ("".to_string(), Some("via scptdisplay".to_string()))
        );
        assert_eq!(
            format_notification_origin(Some("hi"), None),
            ("hi".to_string(), Some("via scptdisplay".to_string()))
        );
        assert_eq!(
            format_notification_origin(Some("hi"), Some("scptdisplay alert")),
            ("hi".to_string(), Some("scptdisplay alert".to_string()))
        );
        assert_eq!(
            format_notification_origin(Some("hi"), Some("custom")),
            (
                "hi\n(via scptdisplay)".to_string(),
                Some("custom".to_string())
            )
        );
    }

    #[test]
    fn test_format_alert_message_origin() {
        assert_eq!(
            format_alert_message_origin(None),
            Some("(via scptdisplay)".to_string())
        );
        assert_eq!(
            format_alert_message_origin(Some("")),
            Some("(via scptdisplay)".to_string())
        );
        assert_eq!(
            format_alert_message_origin(Some("error")),
            Some("error\n(via scptdisplay)".to_string())
        );
        assert_eq!(
            format_alert_message_origin(Some("error\n")),
            Some("error\n(via scptdisplay)".to_string())
        );
        assert_eq!(
            format_alert_message_origin(Some("error\n(via scptdisplay)")),
            Some("error\n(via scptdisplay)".to_string())
        );
    }

    #[test]
    fn test_format_dialog_title_origin() {
        assert_eq!(
            format_dialog_title_origin(None),
            Some("scptdisplay".to_string())
        );
        assert_eq!(
            format_dialog_title_origin(Some("")),
            Some("scptdisplay".to_string())
        );
        assert_eq!(
            format_dialog_title_origin(Some("My App")),
            Some("My App".to_string())
        );
    }
}
