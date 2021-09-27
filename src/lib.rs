use serde_json::map::Map;
use serde_json::Value;

pub fn intersect(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (&Value::Object(ref a), &Value::Object(ref b)) => {
            let mut m = Map::new();
            for (k, v) in a {
                if !b.contains_key(k) {
                    continue;
                }
                let bv = b.get(k).unwrap();
                if let Some(v) = intersect(v, bv) {
                    m.insert(k.to_owned(), v);
                }
            }
            Some(Value::Object(m))
        }
        // TODO: O(n^2) b/c serde_json::Value does not support Hash
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

pub fn union(a: &Value, b: &Value) -> Option<Value> {
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
                m.insert(k.to_owned(), v);
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

// returns all the fields that only exist in a (all fields in b are subtracted from a)
pub fn difference(a: &Value, b: &Value) -> Option<Value> {
    match (a, b) {
        (&Value::Object(ref a), &Value::Object(ref b)) => {
            let mut m = Map::new();
            for (k, v) in a {
                if !b.contains_key(k) {
                    m.insert(k.to_owned(), v.clone());
                    continue;
                }
                let bv = b.get(k).unwrap();
                if let Some(v) = difference(v, bv) {
                    m.insert(k.to_owned(), v);
                }
            }
            Some(Value::Object(m))
        }
        (&Value::Array(ref a), &Value::Array(ref b)) => {
            let mut v = a.clone();
            v.retain(|x| !b.contains(x));
            Some(Value::Array(v))
        }
        (a, b) => {
            if a == b {
                None
            } else {
                Some(a.clone())
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

    #[test]
    fn difference_empty_objects() {
        let a = json!({});
        let b = json!({});
        assert_eq!(Some(json!({})), difference(&a, &b));
    }

    #[test]
    fn difference_objects() {
        let a = json!({"a": "b", "c": "d"});
        let b = json!({});
        assert_eq!(Some(json!({"a": "b", "c": "d"})), difference(&a, &b));

        let a = json!({"a": "b", "c": "d"});
        let b = json!({"c": "d"});
        assert_eq!(Some(json!({"a": "b"})), difference(&a, &b));
    }

    #[test]
    fn difference_objects_nested() {
        let a = json!({"a": {"b": "c", "e": "f"}});
        let b = json!({"a": {"b": "c"}});
        assert_eq!(Some(json!({"a": {"e": "f"}})), difference(&a, &b));
    }

    #[test]
    fn difference_empty_arrays() {
        let a = json!([]);
        let b = json!([]);
        assert_eq!(Some(json!([])), difference(&a, &b));
    }

    #[test]
    fn difference_arrays() {
        let a = json!(["a", "b", "c"]);
        let b = json!([]);
        assert_eq!(Some(json!(["a", "b", "c"])), difference(&a, &b));

        let a = json!(["a", "b", "c"]);
        let b = json!(["b", "d"]);
        assert_eq!(Some(json!(["a", "c"])), difference(&a, &b));
    }

    #[test]
    fn difference_same_string() {
        let a = json!("abc");
        let b = json!("abc");
        assert_eq!(None, difference(&a, &b));
    }

    #[test]
    fn difference_string() {
        let a = json!("abc");
        let b = json!("cde");
        assert_eq!(Some(json!("abc")), difference(&a, &b));
    }

    #[test]
    fn difference_same_bool() {
        let a = json!(true);
        let b = json!(true);
        assert_eq!(None, difference(&a, &b));
    }

    #[test]
    fn difference_bool() {
        let a = json!(true);
        let b = json!(false);
        assert_eq!(Some(json!(true)), difference(&a, &b));
    }

    #[test]
    fn difference_same_num() {
        let a = json!(1);
        let b = json!(1);
        assert_eq!(None, difference(&a, &b));
    }

    #[test]
    fn difference_num() {
        let a = json!(1);
        let b = json!(10);
        assert_eq!(Some(json!(1)), difference(&a, &b));
    }

    #[test]
    fn difference_null() {
        let a = json!(null);
        let b = json!(null);
        assert_eq!(None, difference(&a, &b));
    }
}
