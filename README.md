# About

[JsonPatch](./src/json_patch.rs) is an `enum` to express the ternary logic of JSON properties. Let
```rust
struct Message {
    answer: JsonPatch<u32>
}
```
be a structure to be sent as JSON message. It covers three cases:

1. `Message { answer: JsonPatch::Value(42) }`:
    * The property `answer` exists with value `42`.
    * It serializes to `{ "answer": 42 }`.
2. `Message { answer: JsonPatch::Null }`:
    * The property `answer` exists but is `null`.
    * It serializes to `{ "answer": null }`.
3. `Message { answer: JsonPatch::Absent }`
    * The property `answer` is absent.
    * It serializes to `{ }`.

Ternary properties can be used to implement update logic in CRUD databases: 
1. `JsonPatch::Value(x)` overwrites the target value with `x`.
2. `JsonPatch::Null` deletes the target value.
2. `JsonPatch::Missing` leaves the target value unchanged.

See [json-patch-demo](./src/bin/json-patch-demo.rs) for a toy example.

Source: https://stackoverflow.com/a/44332837 (credits to [E_net4 the curator](https://stackoverflow.com/users/1233251/e-net4-the-curator) and [Shepmaster](https://stackoverflow.com/users/155423/shepmaster)).

# Running
```shell
$ cargo test
$ cargo run
```

