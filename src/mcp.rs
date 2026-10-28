use crate::cli::{AlertType, Commands};
use crate::response::Data;
use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    schemars, tool, tool_router, ErrorData,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ScptdisplayMcpServer {
    osascript: String,
}

impl ScptdisplayMcpServer {
    pub fn new() -> Self {
        Self {
            osascript: "osascript".to_string(),
        }
    }

    pub fn with_osascript(osascript: impl Into<String>) -> Self {
        Self {
            osascript: osascript.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct NotificationParams {
    #[schemars(description = "The body text of the notification")]
    pub text: Option<String>,
    #[schemars(description = "The title of the notification")]
    pub title: Option<String>,
    #[schemars(description = "The subtitle of the notification")]
    pub subtitle: Option<String>,
    #[schemars(
        description = "The name of a sound to play when the notification appears (installed in Library/Sounds)"
    )]
    pub sound_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct NotificationResult {
    #[schemars(description = "Whether the notification command completed successfully")]
    pub success: bool,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct AlertParams {
    #[schemars(description = "The alert text, which is displayed in emphasized system font")]
    pub text: String,
    #[schemars(
        description = "An explanatory message displayed below the alert text in smaller font"
    )]
    pub message: Option<String>,
    #[schemars(description = "The type of alert: 'informational', 'warning', or 'critical'")]
    pub alert_type: Option<AlertType>,
    #[schemars(description = "A list of up to three button names")]
    pub buttons: Option<Vec<String>>,
    #[schemars(description = "The name or 1-based index of the default button")]
    pub default_button: Option<String>,
    #[schemars(description = "The name or 1-based index of the cancel button")]
    pub cancel_button: Option<String>,
    #[schemars(description = "The number of seconds to wait before automatically dismissing")]
    pub giving_up_after: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct AlertResult {
    #[schemars(description = "The name of the button clicked by the user, if any")]
    pub button: Option<String>,
    #[schemars(description = "True if the alert timed out and gave up")]
    pub gave_up: bool,
    #[schemars(description = "Raw AppleScript record key-value pairs")]
    pub record: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Default)]
pub struct DialogParams {
    #[schemars(description = "The dialog prompt text displayed in emphasized system font")]
    pub text: String,
    #[schemars(
        description = "The initial contents of an editable text field. If omitted, no text field is displayed."
    )]
    pub default_answer: Option<String>,
    #[schemars(description = "If true, obscures entered text with bullets for passwords")]
    pub hidden_answer: Option<bool>,
    #[schemars(description = "A list of up to three button names")]
    pub buttons: Option<Vec<String>>,
    #[schemars(description = "The name or 1-based index of the default button")]
    pub default_button: Option<String>,
    #[schemars(description = "The name or 1-based index of the cancel button")]
    pub cancel_button: Option<String>,
    #[schemars(description = "The dialog window title")]
    pub title: Option<String>,
    #[schemars(
        description = "Icon type ('stop'/0, 'note'/1, 'caution'/2) or path to a .icns file"
    )]
    pub icon: Option<String>,
    #[schemars(description = "The number of seconds to wait before automatically dismissing")]
    pub giving_up_after: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct DialogResult {
    #[schemars(description = "The text entered by the user in the editable field")]
    pub text: Option<String>,
    #[schemars(description = "The name of the button clicked by the user, if any")]
    pub button: Option<String>,
    #[schemars(description = "True if the dialog timed out and gave up")]
    pub gave_up: bool,
    #[schemars(description = "Raw AppleScript record key-value pairs")]
    pub record: HashMap<String, String>,
}

#[tool_router(server_handler)]
impl ScptdisplayMcpServer {
    #[tool(description = "Posts a macOS desktop notification banner using Notification Center")]
    fn notification(
        &self,
        Parameters(params): Parameters<NotificationParams>,
    ) -> Result<Json<NotificationResult>, ErrorData> {
        if params.text.is_none() && params.title.is_none() {
            return Err(ErrorData::invalid_params(
                "At least one of 'text' or 'title' must be specified for notification",
                None,
            ));
        }

        let cmd = Commands::Notification {
            text: params.text,
            title: params.title,
            subtitle: params.subtitle,
            sound_name: params.sound_name,
        };

        cmd.execute(&self.osascript)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        Ok(Json(NotificationResult { success: true }))
    }

    #[tool(description = "Displays a standardized macOS alert dialog with 1 to 3 buttons")]
    fn alert(
        &self,
        Parameters(params): Parameters<AlertParams>,
    ) -> Result<Json<AlertResult>, ErrorData> {
        let cmd = Commands::Alert {
            text: params.text,
            message: params.message,
            alert_type: params.alert_type.unwrap_or(AlertType::Informational),
            buttons: params.buttons.unwrap_or_default(),
            default_button: params.default_button,
            cancel_button: params.cancel_button,
            giving_up_after: params.giving_up_after,
        };

        let data = cmd
            .execute(&self.osascript)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        match data {
            Data::Alert {
                record,
                button,
                gave_up,
                ..
            } => Ok(Json(AlertResult {
                button,
                gave_up,
                record,
            })),
            _ => Err(ErrorData::internal_error(
                "Unexpected alert response data",
                None,
            )),
        }
    }

    #[tool(
        description = "Displays an interactive macOS dialog with buttons and optional text field"
    )]
    fn dialog(
        &self,
        Parameters(params): Parameters<DialogParams>,
    ) -> Result<Json<DialogResult>, ErrorData> {
        let cmd = Commands::Dialog {
            text: params.text,
            default_answer: params.default_answer,
            hidden_answer: params.hidden_answer.unwrap_or(false),
            buttons: params.buttons.unwrap_or_default(),
            default_button: params.default_button,
            cancel_button: params.cancel_button,
            title: params.title,
            icon: params.icon,
            giving_up_after: params.giving_up_after,
        };

        let data = cmd
            .execute(&self.osascript)
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;

        match data {
            Data::Dialog {
                record,
                text,
                button,
                gave_up,
                ..
            } => Ok(Json(DialogResult {
                text,
                button,
                gave_up,
                record,
            })),
            _ => Err(ErrorData::internal_error(
                "Unexpected dialog response data",
                None,
            )),
        }
    }
}

pub async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_mcp_server_with_osascript("osascript").await
}

pub async fn run_mcp_server_with_osascript(
    osascript: impl Into<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use rmcp::serve_server;
    use rmcp::transport::io::stdio;
    let server = ScptdisplayMcpServer::with_osascript(osascript);
    let service = serve_server(server, stdio()).await?;
    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_params_validation() {
        let server = ScptdisplayMcpServer::with_osascript("true");
        let params = NotificationParams {
            text: None,
            title: None,
            subtitle: None,
            sound_name: None,
        };
        assert!(server.notification(Parameters(params)).is_err());
    }

    #[test]
    fn test_notification_execution_mock() {
        let server = ScptdisplayMcpServer::with_osascript("true");
        let params = NotificationParams {
            text: Some("hello".to_string()),
            title: Some("world".to_string()),
            subtitle: None,
            sound_name: None,
        };
        let res = server.notification(Parameters(params)).unwrap();
        assert_eq!(res.0, NotificationResult { success: true });
    }

    #[test]
    fn test_alert_execution_mock() {
        let server = ScptdisplayMcpServer::with_osascript("echo");
        let params = AlertParams {
            text: "test".to_string(),
            message: None,
            alert_type: Some(AlertType::Warning),
            buttons: Some(vec!["OK".to_string()]),
            default_button: None,
            cancel_button: None,
            giving_up_after: None,
        };
        let res = server.alert(Parameters(params)).unwrap();
        assert_eq!(res.0.gave_up, false);
    }

    #[test]
    fn test_dialog_execution_mock() {
        let server = ScptdisplayMcpServer::with_osascript("echo");
        let params = DialogParams {
            text: "prompt".to_string(),
            default_answer: Some("alice".to_string()),
            hidden_answer: None,
            buttons: Some(vec!["OK".to_string()]),
            default_button: None,
            cancel_button: None,
            title: None,
            icon: None,
            giving_up_after: None,
        };
        let res = server.dialog(Parameters(params)).unwrap();
        assert_eq!(res.0.gave_up, false);
    }
}
