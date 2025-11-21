use std::collections::HashMap;

/// Extract a value between single quotes at a specific occurrence
pub fn extract_quoted_value(msg: &str, occurrence: usize) -> Option<String> {
    msg.split('\'').nth(occurrence).map(|s| s.to_string())
}

/// Extract the first quoted value from a message
pub fn extract_first_quoted(msg: &str) -> Option<String> {
    extract_quoted_value(msg, 1)
}

/// Extract the second quoted value from a message
pub fn extract_second_quoted(msg: &str) -> Option<String> {
    extract_quoted_value(msg, 3)
}

/// Extract the third quoted value from a message
pub fn extract_third_quoted(msg: &str) -> Option<String> {
    extract_quoted_value(msg, 5)
}

/// Parse TS2322 error message to extract "from" and "to" types
pub fn parse_ts2322_error(msg: &str) -> Option<(String, String)> {
    let mut chars = msg.chars().peekable();

    fn read_quoted<I: Iterator<Item = char>>(chars: &mut std::iter::Peekable<I>) -> Option<String> {
        // Expect starting `'`
        if chars.next()? != '\'' {
            return None;
        }
        let mut out = String::new();
        while let Some(&c) = chars.peek() {
            chars.next();
            if c == '\'' {
                break;
            }
            out.push(c);
        }
        Some(out)
    }

    while let Some(_) = chars.next() {
        let mut lookahead = chars.clone();
        if lookahead.next()? == 'y'
            && lookahead.next()? == 'p'
            && lookahead.next()? == 'e'
            && lookahead.next()? == ' '
            && lookahead.next()? == '\''
        {
            for _ in 0..4 {
                chars.next();
            }
            let from = read_quoted(&mut chars)?;

            while let Some(c) = chars.next() {
                if c == '\'' {
                    let mut secondary = String::new();
                    for c2 in chars.by_ref() {
                        if c2 == '\'' {
                            break;
                        }
                        secondary.push(c2);
                    }
                    return Some((from, secondary));
                }
            }
        }
    }

    None
}

/// Parse error message to extract type name (looks for "type 'X'" pattern)
pub fn parse_property_missing_error(msg: &str) -> Option<String> {
    let type_marker = "type '";
    if let Some(start_index) = msg.rfind(type_marker) {
        let rest_of_msg = &msg[start_index + type_marker.len()..];
        if let Some(end_index) = rest_of_msg.find('\'') {
            return Some(rest_of_msg[..end_index].to_string());
        }
    }
    None
}

/// Parse TS2345 error to extract type mismatches in object properties
pub fn parse_ts2345_error(msg: &str) -> Option<Vec<(String, String, String)>> {
    // Extract the provided and expected object types from the first line
    let provided_obj = extract_object_type(msg, "Argument of type '")?;
    let expected_obj = extract_object_type(msg, "to parameter of type '")?;

    // Parse both object types to extract their properties
    let provided_props = parse_object_properties(&provided_obj);
    let expected_props = parse_object_properties(&expected_obj);

    // Find all mismatched properties
    let mut mismatches = Vec::new();
    for (key, expected_type) in &expected_props {
        if let Some(provided_type) = provided_props.get(key)
            && provided_type != expected_type {
                mismatches.push((key.clone(), provided_type.clone(), expected_type.clone()));
            }
    }

    Some(mismatches)
}

fn extract_object_type(msg: &str, marker: &str) -> Option<String> {
    let start = msg.find(marker)? + marker.len();
    let rest = &msg[start..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

fn parse_object_properties(obj_type: &str) -> HashMap<String, String> {
    let mut props = HashMap::new();

    // Remove leading/trailing braces and whitespace
    let obj_type = obj_type.trim();
    if !obj_type.starts_with('{') || !obj_type.ends_with('}') {
        return props;
    }

    let inner = &obj_type[1..obj_type.len() - 1];

    // Split by semicolons to get individual properties
    for prop in inner.split(';') {
        let prop = prop.trim();
        if prop.is_empty() {
            continue;
        }

        // Split by colon to get key and type
        if let Some(colon_pos) = prop.find(':') {
            let key = prop[..colon_pos].trim().to_string();
            let value = prop[colon_pos + 1..].trim().to_string();
            props.insert(key, value);
        }
    }

    props
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_quoted_value() {
        let msg = "Property 'foo' does not exist on type 'Bar'";
        assert_eq!(extract_first_quoted(msg), Some("foo".to_string()));
        assert_eq!(extract_second_quoted(msg), Some("Bar".to_string()));
    }

    #[test]
    fn test_parse_ts2322_error() {
        let msg = "Type 'string' is not assignable to type 'number'.";
        let result = parse_ts2322_error(msg);
        assert_eq!(result, Some(("string".to_string(), "number".to_string())));
    }

    #[test]
    fn test_parse_property_missing_error() {
        let msg = "Property 'x' is missing in type 'MyType' but required in type 'OtherType'.";
        assert_eq!(
            parse_property_missing_error(msg),
            Some("OtherType".to_string())
        );
    }
}
