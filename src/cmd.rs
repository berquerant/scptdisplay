use std::convert::{From, Into};
use std::process::Command;

#[derive(Debug, PartialEq, Clone)]
pub struct Cmd(Vec<String>);

impl Cmd {
    pub fn new<S: Into<String>>(program: S) -> Cmd {
        let v = vec![program.into()];
        Cmd(v)
    }
    /// Add an argument if arg is not None.
    pub fn arg<S: Into<String>>(&mut self, arg: Option<S>) {
        if let Some(x) = arg {
            self.0.push(x.into());
        }
    }
    /// Add key and value if value if not None.
    pub fn pair<S: Into<String>, T: Into<String>>(&mut self, key: S, value: Option<T>) {
        if let Some(x) = value {
            self.0.push(key.into());
            self.0.push(x.into());
        }
    }
}

impl From<Cmd> for String {
    fn from(from: Cmd) -> Self {
        from.0
            .into_iter()
            .reduce(|acc, x| format!("{} {}", acc, x))
            .unwrap()
    }
}

impl From<Cmd> for Command {
    fn from(from: Cmd) -> Self {
        let v = from.0;
        let program = &v[0];
        let mut c = Command::new(program);
        for x in v.iter().skip(1) {
            c.arg(x);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd_init() {
        let got = Cmd::new("p");
        let want = vec!["p"];
        assert_eq!(want, got.0);
    }

    #[test]
    fn test_cmd_pair() {
        let mut got = Cmd::new("p");
        got.pair("key", Some("value"));
        got.pair("key2".to_string(), None::<String>);
        let want = vec!["p", "key", "value"];
        assert_eq!(want, got.0);
    }

    #[test]
    fn test_cmd_arg() {
        let mut got = Cmd::new("p");
        got.arg(Some("1"));
        got.arg::<String>(None);
        let want = vec!["p", "1"];
        assert_eq!(want, got.0);
    }
}
