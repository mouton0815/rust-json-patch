use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum JsonPatch<T> {
    Value(T), // Set the aggregate value to the value of T
    Null,     // Set the aggregate value to null
    Absent    // Do not change the aggregate value
}

impl<T> JsonPatch<T> {
    pub const fn is_value(&self) -> bool {
        matches!(*self, JsonPatch::Value(_))
    }
    pub const fn is_null(&self) -> bool {
        matches!(*self, JsonPatch::Null)
    }
    pub const fn is_absent(&self) -> bool {
        matches!(*self, JsonPatch::Absent)
    }
}

// https://stackoverflow.com/a/44332837
impl<T> Default for JsonPatch<T> {
    fn default() -> Self {
        JsonPatch::Absent
    }
}

impl<T> From<Option<T>> for JsonPatch<T> {
    fn from(opt: Option<T>) -> JsonPatch<T> {
        match opt {
            Some(v) => JsonPatch::Value(v),
            None => JsonPatch::Null,
        }
    }
}

impl<'de, T> Deserialize<'de> for JsonPatch<T> where T: Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Option::deserialize(deserializer).map(Into::into)
    }
}

// See https://serde.rs/impl-serialize.html
impl<T> Serialize for JsonPatch<T> where T: Serialize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            JsonPatch::Value(t) => serializer.serialize_some(t),
            JsonPatch::Null => serializer.serialize_none(),
            JsonPatch::Absent => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::json_patch::JsonPatch;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Record {
        #[serde(default)]
        #[serde(skip_serializing_if = "JsonPatch::is_absent")]
        a: JsonPatch<String>,

        #[serde(default)]
        #[serde(skip_serializing_if = "JsonPatch::is_absent")]
        b: JsonPatch<u32>,

        #[serde(default)]
        #[serde(skip_serializing_if = "JsonPatch::is_absent")]
        c: JsonPatch<Vec<i32>>
    }

    #[test]
    pub fn test_patch_value() {
        let t = JsonPatch::Value(String::from("123"));
        assert!(t.is_value());
        assert!(!t.is_null());
        assert!(!t.is_absent());

        let json = serde_json::to_string(&t);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from(r#""123""#));
    }

    #[test]
    pub fn test_patch_null() {
        let t: JsonPatch<u32> = JsonPatch::Null;
        assert!(!t.is_value());
        assert!(t.is_null());
        assert!(!t.is_absent());

        let json = serde_json::to_string(&t);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("null"));
    }

    #[test]
    pub fn test_patch_absent() {
        let t: JsonPatch<u32> = JsonPatch::Absent;
        assert!(!t.is_value());
        assert!(!t.is_null());
        assert!(t.is_absent());

        let json = serde_json::to_string(&t);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from("null"));
    }

    #[test]
    pub fn test_serde_record_value() {
        let record_ref = Record{
            a: JsonPatch::Value(String::from("Foo")),
            b: JsonPatch::Value(123),
            c: JsonPatch::Value(vec![3,-5, 7])
        };
        let json_ref = r#"{"a":"Foo","b":123,"c":[3,-5,7]}"#;
        serde_and_verify(&record_ref, json_ref);
    }

    #[test]
    pub fn test_serialize_record_null() {
        let record_ref = Record{
            a: JsonPatch::Null,
            b: JsonPatch::Null,
            c: JsonPatch::Null
        };
        let json_ref = r#"{"a":null,"b":null,"c":null}"#;
        serde_and_verify(&record_ref, json_ref);
    }

    #[test]
    pub fn test_serialize_record_absent() {
        let record_ref = Record{
            a: JsonPatch::Absent,
            b: JsonPatch::Absent,
            c: JsonPatch::Absent
        };
        let json_ref = r#"{}"#;
        serde_and_verify(&record_ref, json_ref);
    }

    fn serde_and_verify(record_ref: &Record, json_ref: &str) {
        let json = serde_json::to_string(&record_ref);
        assert!(json.is_ok());
        assert_eq!(json.unwrap(), String::from(json_ref));

        let record : Result<Record, serde_json::Error> = serde_json::from_str(json_ref);
        assert!(record.is_ok());
        assert_eq!(record.unwrap(), *record_ref);
    }
}