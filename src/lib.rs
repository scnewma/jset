use serde_json::map::Map;
use serde_json::Value;

pub fn fold_intersect(values: Vec<Value>) -> Option<Value> {
    if values.is_empty() {
        return None;
    }

    let mut acc = values[0].clone();
    for val in values.iter().skip(1) {
        acc = intersect(&acc, val)?;
    }
    Some(acc)
}

fn intersect(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (&Value::Object(ref a), &Value::Object(ref b)) => {
            let mut m = Map::new();
            for (k, v) in a {
                if !b.contains_key(k) {
                    continue;
                }
                let bv = b.get(k).unwrap();
                if let Some(v) = intersect(v, bv) {
                    m.insert(k.to_string(), v);
                }
            }
            Some(Value::Object(m))
        }
        (&Value::Array(ref a), &Value::Array(ref b)) => {
            let mut v = a.clone();
            v.retain(|x| b.contains(x));
            Some(Value::Array(v))
        }
        (a, b) => {
            if a == b {
                Some(a.clone())
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn intersect_no_values() {
        assert_eq!(None, fold_intersect(vec![]));
    }

    #[test]
    fn intersect_one_value() {
        let v = json!({"a": "b"});
        assert_eq!(Some(v.clone()), fold_intersect(vec![v]));
    }

    #[test]
    fn intersect_two_values() {
        let v = json!({"a": "b"});
        let v2 = json!({"a": "b", "c": "d"});
        assert_eq!(Some(v.clone()), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn intersect_three_values() {
        let v = json!({"a": "b"});
        let v2 = json!({"a": "b", "c": "d"});
        let v3 = json!({"a": "b", "c": "d", "e": "f"});
        assert_eq!(Some(v.clone()), fold_intersect(vec![v, v2, v3]));
    }

    #[test]
    fn no_intersection() {
        let v = json!({"a": "b"});
        let v2 = json!({"c": "d"});
        assert_eq!(Some(json!({})), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn nested_object_intersection() {
        let v = json!({"a": "b", "c": {"d": "e"}});
        let v2 = json!({"c": {"d": "e"}});
        assert_eq!(Some(json!({"c": {"d": "e"}})), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn array_intersection() {
        let v = json!(["a", "b"]);
        let v2 = json!(["a", "c"]);
        assert_eq!(Some(json!(["a"])), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn array_obj_intersection() {
        let v = json!([{"a": "b"}]);
        let v2 = json!([{"a": "b"}]);
        assert_eq!(Some(json!([{"a": "b"}])), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn array_obj_no_intersection() {
        // arrays are intersected based on their equality so objects are not intersected within the
        // array (how would you know which index of the array to intersect with which other index?)
        let v = json!([{"a": "b", "c": "d"}]);
        let v2 = json!([{"a": "b", "e": "f"}]);
        assert_eq!(Some(json!([])), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn array_no_intersection() {
        let v = json!(["a", "b"]);
        let v2 = json!(["c", "d"]);
        assert_eq!(Some(json!([])), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn string_intersection() {
        let v = json!("a");
        let v2 = json!("a");
        assert_eq!(Some(json!("a")), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn string_no_intersection() {
        let v = json!("a");
        let v2 = json!("b");
        assert_eq!(None, fold_intersect(vec![v, v2]));
    }

    #[test]
    fn bool_intersection() {
        let v = json!(true);
        let v2 = json!(true);
        assert_eq!(Some(json!(true)), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn bool_no_intersection() {
        let v = json!(true);
        let v2 = json!(false);
        assert_eq!(None, fold_intersect(vec![v, v2]));
    }

    #[test]
    fn null_intersection() {
        let v = json!(null);
        let v2 = json!(null);
        assert_eq!(Some(json!(null)), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn null_no_intersection() {
        let v = json!(null);
        let v2 = json!("a");
        assert_eq!(None, fold_intersect(vec![v, v2]));
    }

    #[test]
    fn num_intersection() {
        let v = json!(1);
        let v2 = json!(1);
        assert_eq!(Some(json!(1)), fold_intersect(vec![v, v2]));
    }

    #[test]
    fn num_no_intersection() {
        let v = json!(1);
        let v2 = json!(2);
        assert_eq!(None, fold_intersect(vec![v, v2]));
    }
}
