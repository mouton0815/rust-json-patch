//
// This program demonstrates how to compose messages of person events and "send" them as strings.
// The consumer parses the message strings and reconstructs the person events. It uses the events
// to maintain person "database" by applying an insert/update/delete semantics using JsonTernary
// fields. In particular, person records can be deleted if the person event is represented as
// JsonTernary::Null.
//
use std::collections::{HashMap, LinkedList};
use serde::{Deserialize, Serialize};
use serde_json_ternary::json_ternary::JsonTernary;

// An update event to be send in JSON format
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")] // Follow JSON conventions
struct PersonEvent {
    person_id: u32,
    // Instruct serde to
    // * not write familyName into the JSON document if the value is absent
    // * use the default deserializer if  familyName is absent (which constructs JsonTernary::Absent)
    #[serde(skip_serializing_if = "JsonTernary::is_absent")]
    #[serde(default)]
    first_name: JsonTernary<String>,

    #[serde(skip_serializing_if = "JsonTernary::is_absent")]
    #[serde(default)]
    family_name: JsonTernary<String>,

    #[serde(skip_serializing_if = "JsonTernary::is_absent")]
    #[serde(default)]
    spouse_id: JsonTernary<u32>
}

// A message to be send in JSON format
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Message {
    person_id: u32,

    #[serde(skip_serializing_if = "JsonTernary::is_absent")]
    #[serde(default)]
    person_data: JsonTernary<PersonEvent>
}

// A "database" record holding person data
#[derive(Debug)]
struct PersonRecord {
    first_name: String,
    family_name: String,
    spouse_id: Option<u32>
}

type PersonDB = HashMap<u32, PersonRecord>;

fn main() {
    let mut person_db: PersonDB = HashMap::new();
    let mut messages = produce_messages();
    consume_messages(&mut messages, &mut person_db);
    print_person_db(&person_db);
}

fn produce_messages() -> LinkedList<String> {
    let mut messages = LinkedList::new();

    // Birth of John
    let event = PersonEvent {
        person_id: 1,
        first_name: JsonTernary::Value(String::from("John")),
        family_name: JsonTernary::Value(String::from("Doe")),
        spouse_id: JsonTernary::Absent
    };
    messages.push_back(build_update_message(event));

    // Birth of Jane
    let event = PersonEvent {
        person_id: 2,
        first_name: JsonTernary::Value(String::from("Jane")),
        family_name: JsonTernary::Value(String::from("Deer")),
        spouse_id: JsonTernary::Absent
    };
    messages.push_back(build_update_message(event));

    // John marries Jane
    let event = PersonEvent {
        person_id: 1,
        first_name: JsonTernary::Absent,  // First name ...
        family_name: JsonTernary::Absent, // ... and family name stay
        spouse_id: JsonTernary::Value(2)
    };
    messages.push_back(build_update_message(event));

    // Jane marries John (and changes her family name)
    let event = PersonEvent {
        person_id: 2,
        first_name: JsonTernary::Absent,
        family_name: JsonTernary::Value(String::from("Doe")),
        spouse_id: JsonTernary::Value(1)
    };
    messages.push_back(build_update_message(event));

    // John gets divorced
    let event = PersonEvent {
        person_id: 1,
        first_name: JsonTernary::Absent,
        family_name: JsonTernary::Absent,
        spouse_id: JsonTernary::Null
    };
    messages.push_back(build_update_message(event));

    // And Jane too, but she keeps the family name
    let event = PersonEvent {
        person_id: 2,
        first_name: JsonTernary::Absent,
        family_name: JsonTernary::Absent,
        spouse_id: JsonTernary::Null
    };
    messages.push_back(build_update_message(event));

    // John dies :-(
    messages.push_back(build_delete_message(1));

    messages
}

fn build_update_message(event: PersonEvent) -> String {
    let message = Message {
        person_id: event.person_id,
        person_data: JsonTernary::Value(event)
    };
    serde_json::to_string(&message).unwrap()
}

fn build_delete_message(person_id: u32) -> String {
    let message = Message {
        person_id,
        person_data: JsonTernary::Null
    };
    serde_json::to_string(&message).unwrap()
}

fn consume_messages(messages: &mut LinkedList<String>, person_db: &mut PersonDB) {
    while let Some(message_string) = messages.pop_front() {
        let message : Message = serde_json::from_str(message_string.as_str()).unwrap();
        println!("{:?}", message);
        if let JsonTernary::Value(event) = message.person_data {
            if let Some(record) = person_db.get(&message.person_id) {
                // Use aux variable to avoid https://github.com/rust-lang/rust/issues/59159
                let merged_record = merge_person_record(record, &event);
                person_db.insert(message.person_id, merged_record);
            } else {
                person_db.insert(message.person_id, create_person_record(&event));
            }
        } else {
            person_db.remove(&message.person_id);
        }
    }
}

fn create_person_record(event: &PersonEvent) -> PersonRecord {
    PersonRecord {
        first_name : match &event.first_name {
            JsonTernary::Value(name) => name.clone(),
            _ => String::new()
        },
        family_name : match &event.family_name {
            JsonTernary::Value(name) => name.clone(),
            _ => String::new()
        },
        spouse_id : match event.spouse_id {
            JsonTernary::Value(id) => Option::Some(id),
            _ => Option::None
        }
    }
}

fn merge_person_record(record: &PersonRecord, event: &PersonEvent) -> PersonRecord {
    PersonRecord {
        first_name: match &event.first_name {
            JsonTernary::Value(name) => name.clone(),
            JsonTernary::Null => String::new(),
            JsonTernary::Absent => record.first_name.clone()
        },
        family_name: match &event.family_name {
            JsonTernary::Value(name) => name.clone(),
            JsonTernary::Null => String::new(),
            JsonTernary::Absent => record.family_name.clone()
        },
        spouse_id: match event.spouse_id {
            JsonTernary::Value(id) => Option::Some(id),
            JsonTernary::Null => Option::None,
            JsonTernary::Absent => record.spouse_id
        }
    }
}

fn print_person_db(person_db: &PersonDB) {
    for (id, person) in person_db {
        println!("{} -> {:?}", id, person);
    }
}