use std::collections::HashMap;

/// Set of conversions into apple script input.
pub struct Input;

impl Input {
    pub fn quoted(x: &str) -> String {
        format!("\"{}\"", x)
    }

    pub fn integer_or_text(x: &str) -> String {
        if x.parse::<u8>().is_ok() {
            x.into()
        } else {
            Input::quoted(x)
        }
    }

    pub fn apple_script_list<S: Into<String>>(v: Vec<S>) -> String {
        if v.is_empty() {
            "{}".into()
        } else {
            let x = v
                .into_iter()
                .map(|x| {
                    let s = x.into();
                    Input::quoted(&s)
                })
                .reduce(|acc, x| format!("{},{}", acc, x))
                .unwrap();
            format!("{{{}}}", x)
        }
    }
}

/// Set of conversions from apple script output.
pub struct Output;

pub type StringMap = HashMap<String, String>;

impl Output {
    pub fn record(x: &str) -> StringMap {
        let xs: Vec<_> = x.trim_end().split(',').collect();
        let mut m = StringMap::new();
        for x in xs {
            if let Some((k, v)) = x.split_once(':') {
                let k = k.trim_start();
                m.insert(k.to_string(), v.to_string());
            }
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_output_record {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Output::record($input);
                assert_eq!($want, got);
            }
        };
    }

    test_output_record!(test_record_empty, "", StringMap::new());
    test_output_record!(
        test_record_one,
        "key:value",
        StringMap::from([("key".to_string(), "value".to_string())])
    );
    test_output_record!(
        test_record_two,
        "key:value, key 2:value 2",
        StringMap::from([
            ("key".to_string(), "value".to_string()),
            ("key 2".to_string(), "value 2".to_string()),
        ])
    );

    macro_rules! test_input_integer_or_text {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Input::integer_or_text($input);
                assert_eq!($want, got);
            }
        };
    }

    test_input_integer_or_text!(test_integer_or_text_1, "1", "1");
    test_input_integer_or_text!(test_integer_or_text_one, "one", "\"one\"");

    macro_rules! test_input_apple_script_list {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Input::apple_script_list($input);
                assert_eq!($want, got);
            }
        };
    }

    test_input_apple_script_list!(
        test_apple_script_list_empty,
        Vec::<String>::new(),
        "{}".to_string()
    );
    test_input_apple_script_list!(test_apple_script_list_one, vec!["a"], "{\"a\"}".to_string());
    test_input_apple_script_list!(
        test_apple_script_list_two,
        vec!["a", "b"],
        "{\"a\",\"b\"}".to_string()
    );
}
