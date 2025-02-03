use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    result: Code,
    code: Option<i32>,
    error: Option<String>,
    data: Option<Data>,
}

impl Response {
    pub fn from_err_with_code(from: Error, code: Option<i32>) -> Self {
        Response {
            result: Code::Err,
            code,
            error: Some(format!("{}", from)),
            data: None,
        }
    }
}

impl From<Error> for Response {
    fn from(from: Error) -> Self {
        Response {
            result: Code::Err,
            code: None,
            error: Some(format!("{}", from)),
            data: None,
        }
    }
}

impl From<Data> for Response {
    fn from(from: Data) -> Self {
        Response {
            result: Code::Ok,
            code: Some(0),
            error: None,
            data: Some(from),
        }
    }
}

impl From<Result<Data>> for Response {
    fn from(from: Result<Data>) -> Self {
        match from {
            Ok(x) => x.into(),
            Err(x) => x.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Code {
    #[serde(rename(serialize = "ok"))]
    Ok,
    #[serde(rename(serialize = "error"))]
    Err,
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Code::Ok => "ok",
            Code::Err => "error",
        };
        write!(f, "{}", s)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Data {
    #[serde(rename(serialize = "notification"))]
    Notification {},
    #[serde(rename(serialize = "dialog"))]
    Dialog {
        raw: String,
        record: HashMap<String, String>,
        text: Option<String>,
        button: Option<String>,
        gave_up: bool,
    },
    #[serde(rename(serialize = "alert"))]
    Alert {
        raw: String,
        record: HashMap<String, String>,
        button: Option<String>,
        gave_up: bool,
    },
}
