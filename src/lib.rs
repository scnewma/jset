use serde_json::map::Map;
use serde_json::Value;

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

fn union(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (&Value::Object(ref a), &Value::Object(ref b)) => {
            let mut m = a.clone();
            for (k, v) in b {
                let mut v = v.clone();
                if m.contains_key(k) {
                    let mv = m.get(k).unwrap();
                    // insert the recursive union if one can be found, otherwise insert the value
                    // from b directly since b overrides a
                    if let Some(uv) = union(mv, &v) {
                        v = uv.clone();
                    }
                }
                m.insert(k.to_string(), v);
            }
            Some(Value::Object(m))
        }
        // TODO: This is n^2, but serde_json::Value isn't hashable so need to figure out a better
        // was to do this
        // https://github.com/serde-rs/json/issues/747
        (&Value::Array(ref a), &Value::Array(ref b)) => {
            let mut m = a.clone();
            for v in b {
                if !m.contains(v) {
                    m.push(v.clone());
                }
            }
            Some(Value::Array(m))
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
    fn intersect_two_values() {
        let v = json!({"a": "b"});
        let v2 = json!({"a": "b", "c": "d"});
        assert_eq!(Some(v.clone()), intersect(&v, &v2));
    }

    #[test]
    fn no_intersection() {
        let v = json!({"a": "b"});
        let v2 = json!({"c": "d"});
        assert_eq!(Some(json!({})), intersect(&v, &v2));
    }

    #[test]
    fn nested_object_intersection() {
        let v = json!({"a": "b", "c": {"d": "e"}});
        let v2 = json!({"c": {"d": "e"}});
        assert_eq!(Some(json!({"c": {"d": "e"}})), intersect(&v, &v2));
    }

    #[test]
    fn array_intersection() {
        let v = json!(["a", "b"]);
        let v2 = json!(["a", "c"]);
        assert_eq!(Some(json!(["a"])), intersect(&v, &v2));
    }

    #[test]
    fn array_obj_intersection() {
        let v = json!([{"a": "b"}]);
        let v2 = json!([{"a": "b"}]);
        assert_eq!(Some(json!([{"a": "b"}])), intersect(&v, &v2));
    }

    #[test]
    fn array_obj_no_intersection() {
        // arrays are intersected based on their equality so objects are not intersected within the
        // array (how would you know which index of the array to intersect with which other index?)
        let v = json!([{"a": "b", "c": "d"}]);
        let v2 = json!([{"a": "b", "e": "f"}]);
        assert_eq!(Some(json!([])), intersect(&v, &v2));
    }

    #[test]
    fn array_no_intersection() {
        let v = json!(["a", "b"]);
        let v2 = json!(["c", "d"]);
        assert_eq!(Some(json!([])), intersect(&v, &v2));
    }

    #[test]
    fn string_intersection() {
        let v = json!("a");
        let v2 = json!("a");
        assert_eq!(Some(json!("a")), intersect(&v, &v2));
    }

    #[test]
    fn string_no_intersection() {
        let v = json!("a");
        let v2 = json!("b");
        assert_eq!(None, intersect(&v, &v2));
    }

    #[test]
    fn bool_intersection() {
        let v = json!(true);
        let v2 = json!(true);
        assert_eq!(Some(json!(true)), intersect(&v, &v2));
    }

    #[test]
    fn bool_no_intersection() {
        let v = json!(true);
        let v2 = json!(false);
        assert_eq!(None, intersect(&v, &v2));
    }

    #[test]
    fn null_intersection() {
        let v = json!(null);
        let v2 = json!(null);
        assert_eq!(Some(json!(null)), intersect(&v, &v2));
    }

    #[test]
    fn null_no_intersection() {
        let v = json!(null);
        let v2 = json!("a");
        assert_eq!(None, intersect(&v, &v2));
    }

    #[test]
    fn num_intersection() {
        let v = json!(1);
        let v2 = json!(1);
        assert_eq!(Some(json!(1)), intersect(&v, &v2));
    }

    #[test]
    fn num_no_intersection() {
        let v = json!(1);
        let v2 = json!(2);
        assert_eq!(None, intersect(&v, &v2));
    }

    #[test]
    fn union_objects() {
        let v = json!({"a": "b"});
        let v2 = json!({"c": "d"});
        let _union = json!({"a": "b", "c": "d"});
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_nested_objects() {
        let v = json!({"a": {"b": "c"}});
        let v2 = json!({"a": {"c": "d"}});
        let _union = json!({"a": {"b": "c", "c": "d"}});
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_objects_conflicting_values_takes_right() {
        let v = json!({"a": "b"});
        let v2 = json!({"a": "c"});
        let _union = json!({"a": "c"});
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_arrays() {
        let v = json!(["a", "b"]);
        let v2 = json!(["c", "d"]);
        let _union = json!(["a", "b", "c", "d"]);
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_arrays_duplicate_elements() {
        let v = json!(["a", "b"]);
        let v2 = json!(["b", "c"]);
        let _union = json!(["a", "b", "c"]);
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_same_string() {
        let v = json!("a");
        let v2 = json!("a");
        let _union = json!("a");
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_diff_string() {
        let v = json!("a");
        let v2 = json!("b");
        assert_eq!(None, union(&v, &v2));
    }

    #[test]
    fn union_same_bool() {
        let v = json!(true);
        let v2 = json!(true);
        let _union = json!(true);
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_diff_bool() {
        let v = json!(true);
        let v2 = json!(false);
        assert_eq!(None, union(&v, &v2));
    }

    #[test]
    fn union_same_number() {
        let v = json!(1);
        let v2 = json!(1);
        let _union = json!(1);
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_diff_number() {
        let v = json!(1);
        let v2 = json!(2);
        assert_eq!(None, union(&v, &v2));
    }

    #[test]
    fn union_null() {
        let v = json!(null);
        let v2 = json!(null);
        let _union = json!(null);
        assert_eq!(Some(_union), union(&v, &v2));
    }

    #[test]
    fn union_diff_type() {
        let v = json!(1);
        let v2 = json!("a");
        assert_eq!(None, union(&v, &v2));
    }
}
