use std::collections::LinkedList;
use serde::{Deserialize, Serialize};
use learning_rust::json_patch::JsonPatch;

// A "database" record holding person data
#[derive(Debug)]
struct PersonRecord {
    name: String,
    family_name: String,
    spouse_name: String
}

// An update event to be send in JSON format
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")] // Follow JSON conventions
struct PersonEvent {
    name: String,

    // Instruct serde to
    // * not write familyName into the JSON document if the value is absent
    // * use the default deserializer if  familyName is absent (which constructs JsonPatch::Absent)
    #[serde(skip_serializing_if = "JsonPatch::is_absent")]
    #[serde(default)]
    family_name: JsonPatch<String>,

    #[serde(skip_serializing_if = "JsonPatch::is_absent")]
    #[serde(default)]
    spouse_name: JsonPatch<String>
}

fn main() {
    let mut queue = LinkedList::new();

    // Birth of John
    queue.push_back(serde_json::to_string(&PersonEvent {
        name: String::from("John"),
        family_name: JsonPatch::Value(String::from("Doe")),
        spouse_name: JsonPatch::Absent
    }).unwrap());

    // John marries Jane Deer
    queue.push_back(serde_json::to_string(&PersonEvent {
        name: String::from("John"),
        family_name: JsonPatch::Value(String::from("Deer")),
        spouse_name: JsonPatch::Value(String::from("Jane"))
    }).unwrap());

    // John gets divorced but keeps the family name
    queue.push_back(serde_json::to_string(&PersonEvent {
        name: String::from("John"),
        family_name: JsonPatch::Absent,
        spouse_name: JsonPatch::Null
    }).unwrap());

    // The "database" record
    let mut person_record = PersonRecord {
        name: String::new(),
        family_name: String::new(),
        spouse_name: String::new(),
    };

    while let Some(person_event) = queue.pop_front() {
        let person : PersonEvent = serde_json::from_str(person_event.as_str()).unwrap();
        person_record.name = person.name;
        match person.family_name {
            JsonPatch::Value(name) => person_record.family_name = name,
            JsonPatch::Null => person_record.family_name = String::new(),
            JsonPatch::Absent => ()
        }
        match person.spouse_name {
            JsonPatch::Value(name) => person_record.spouse_name = name,
            JsonPatch::Null => person_record.spouse_name = String::new(),
            JsonPatch::Absent => ()
        }
        println!("{:?}", person_record);
    }
}