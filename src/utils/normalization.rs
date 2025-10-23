use serde_json::Value;

pub fn delete_none_values(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            for v in map.values_mut() {
                delete_none_values(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                delete_none_values(v);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_delete_none_values() {
        let mut value = json!({
            "a": 1,
            "b": null,
            "c": {
                "d": 2,
                "e": null,
            },
            "f": [1, null, 3],
        });

        delete_none_values(&mut value);

        assert_eq!(
            value,
            json!({
                "a": 1,
                "c": {
                    "d": 2,
                },
                "f": [1, null, 3],
            })
        );
    }
}
