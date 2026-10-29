use std::collections::HashMap;

/// Set of conversions into apple script input.
pub struct Input;

impl Input {
    pub fn quoted(x: &str) -> String {
        let escaped = x.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    }

    pub fn integer_or_text(x: &str) -> String {
        if x.parse::<u8>().is_ok() {
            x.into()
        } else {
            Input::quoted(x)
        }
    }

    pub fn icon(x: &str) -> String {
        const CONSTANTS: &[&str] = &["stop", "note", "caution"];
        if let Some(&c) = CONSTANTS.iter().find(|&&c| c.eq_ignore_ascii_case(x)) {
            c.to_string()
        } else if x.parse::<u8>().is_ok() {
            x.into()
        } else {
            Input::quoted(x)
        }
    }

    pub fn apple_script_list<S: Into<String>>(v: Vec<S>) -> String {
        let x = v
            .into_iter()
            .map(|x| {
                let s = x.into();
                Input::quoted(&s)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{{x}}}")
    }
}

/// Set of conversions from apple script output.
pub struct Output;

pub type StringMap = HashMap<String, String>;

impl Output {
    pub fn record(x: &str) -> StringMap {
        let trimmed = x.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') && trimmed.len() >= 2 {
            Self::parse_source_record(&trimmed[1..trimmed.len() - 1])
        } else {
            Self::parse_legacy_record(x)
        }
    }

    fn parse_source_record(x: &str) -> StringMap {
        let mut m = StringMap::new();
        let chars: Vec<char> = x.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Skip leading whitespace and commas
            while i < len && (chars[i].is_whitespace() || chars[i] == ',') {
                i += 1;
            }
            if i >= len {
                break;
            }

            // Parse key up to ':'
            let key_start = i;
            while i < len && chars[i] != ':' {
                i += 1;
            }
            if i >= len {
                break;
            }
            let key = chars[key_start..i]
                .iter()
                .collect::<String>()
                .trim()
                .to_string();
            i += 1; // skip ':'

            // Skip whitespace after ':'
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= len {
                m.insert(key, String::new());
                break;
            }

            // Parse value
            if chars[i] == '"' {
                // Quoted string
                i += 1; // skip opening '"'
                let mut val = String::new();
                while i < len {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 1;
                        val.push(chars[i]);
                    } else if chars[i] == '"' {
                        i += 1; // skip closing '"'
                        break;
                    } else {
                        val.push(chars[i]);
                    }
                    i += 1;
                }
                m.insert(key, val);
            } else {
                // Unquoted literal (e.g. true, false, 123)
                let val_start = i;
                while i < len && chars[i] != ',' {
                    i += 1;
                }
                let val = chars[val_start..i]
                    .iter()
                    .collect::<String>()
                    .trim()
                    .to_string();
                m.insert(key, val);
            }
        }
        m
    }

    fn parse_legacy_record(x: &str) -> StringMap {
        const KNOWN_KEYS: &[&str] = &["button returned", "text returned", "gave up"];

        let xs: Vec<_> = x.trim_end().split(',').collect();
        let mut m = StringMap::new();
        let mut current_key: Option<String> = None;

        let has_known_keys = xs.iter().any(|part| {
            part.split_once(':')
                .map(|(k, _)| KNOWN_KEYS.contains(&k.trim_start()))
                .unwrap_or(false)
        });

        for part in xs {
            if let Some((k, v)) = part.split_once(':') {
                let trimmed_k = k.trim_start();
                if has_known_keys {
                    if KNOWN_KEYS.contains(&trimmed_k) {
                        current_key = Some(trimmed_k.to_string());
                        m.insert(trimmed_k.to_string(), v.to_string());
                    } else if let Some(ref prev_k) = current_key {
                        if let Some(val) = m.get_mut(prev_k) {
                            val.push(',');
                            val.push_str(part);
                        }
                    } else {
                        current_key = Some(trimmed_k.to_string());
                        m.insert(trimmed_k.to_string(), v.to_string());
                    }
                } else {
                    current_key = Some(trimmed_k.to_string());
                    m.insert(trimmed_k.to_string(), v.to_string());
                }
            } else if let Some(ref prev_k) = current_key {
                if let Some(val) = m.get_mut(prev_k) {
                    val.push(',');
                    val.push_str(part);
                }
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
    test_output_record!(
        test_record_dialog_with_commas,
        "button returned:OK, text returned:hello, world, gave up:false",
        StringMap::from([
            ("button returned".to_string(), "OK".to_string()),
            ("text returned".to_string(), "hello, world".to_string()),
            ("gave up".to_string(), "false".to_string()),
        ])
    );
    test_output_record!(
        test_record_dialog_with_colons_and_commas,
        "button returned:OK, text returned:Note: first, second: item, gave up:false",
        StringMap::from([
            ("button returned".to_string(), "OK".to_string()),
            (
                "text returned".to_string(),
                "Note: first, second: item".to_string()
            ),
            ("gave up".to_string(), "false".to_string()),
        ])
    );
    test_output_record!(
        test_record_source_format_standard,
        "{button returned:\"OK\", text returned:\"hello, world\", gave up:false}",
        StringMap::from([
            ("button returned".to_string(), "OK".to_string()),
            ("text returned".to_string(), "hello, world".to_string()),
            ("gave up".to_string(), "false".to_string()),
        ])
    );
    test_output_record!(
        test_record_source_format_injected_spoofing_attempt,
        "{button returned:\"Cancel\", text returned:\"evil, button returned:OK, gave up:true\", gave up:false}",
        StringMap::from([
            ("button returned".to_string(), "Cancel".to_string()),
            (
                "text returned".to_string(),
                "evil, button returned:OK, gave up:true".to_string()
            ),
            ("gave up".to_string(), "false".to_string()),
        ])
    );
    test_output_record!(
        test_record_source_format_escaped_quotes_and_backslashes,
        "{button returned:\"OK\\\"\", text returned:\"path\\\\to\\\\file and \\\"quotes\\\"\", gave up:false}",
        StringMap::from([
            ("button returned".to_string(), "OK\"".to_string()),
            (
                "text returned".to_string(),
                "path\\to\\file and \"quotes\"".to_string()
            ),
            ("gave up".to_string(), "false".to_string()),
        ])
    );
    test_output_record!(
        test_record_source_format_no_gave_up,
        "{button returned:\"OK\", text returned:\"hello\"}",
        StringMap::from([
            ("button returned".to_string(), "OK".to_string()),
            ("text returned".to_string(), "hello".to_string()),
        ])
    );

    macro_rules! test_input_quoted {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Input::quoted($input);
                assert_eq!($want, got);
            }
        };
    }

    test_input_quoted!(test_quoted_plain, "hello", "\"hello\"");
    test_input_quoted!(test_quoted_quotes, "foo\"bar", "\"foo\\\"bar\"");
    test_input_quoted!(test_quoted_backslash, "foo\\bar", "\"foo\\\\bar\"");
    test_input_quoted!(
        test_quoted_injection_attempt,
        "foo\" & (do shell script \"pwned\") & \"",
        "\"foo\\\" & (do shell script \\\"pwned\\\") & \\\"\""
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

    macro_rules! test_input_icon {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = Input::icon($input);
                assert_eq!($want, got);
            }
        };
    }

    test_input_icon!(test_icon_stop, "stop", "stop");
    test_input_icon!(test_icon_note, "note", "note");
    test_input_icon!(test_icon_caution, "caution", "caution");
    test_input_icon!(test_icon_note_uppercase, "NOTE", "note");
    test_input_icon!(test_icon_0, "0", "0");
    test_input_icon!(test_icon_1, "1", "1");
    test_input_icon!(test_icon_2, "2", "2");
    test_input_icon!(test_icon_custom, "custom.icns", "\"custom.icns\"");
    test_input_icon!(
        test_icon_injection,
        "note\" & (do shell script \"pwned\")",
        "\"note\\\" & (do shell script \\\"pwned\\\")\""
    );

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
