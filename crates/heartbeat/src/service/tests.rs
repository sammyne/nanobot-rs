use super::*;

#[test]
fn action_skip_interop() {
    // enum → JSON → struct
    let enum_val = Action::Skip;
    let json = serde_json::to_value(&enum_val).unwrap();
    let struct_val: ActionSchema = serde_json::from_value(json).unwrap();
    assert_eq!(struct_val.action, "skip");
    assert!(struct_val.tasks.is_empty());

    // struct → JSON → enum
    let json = serde_json::to_value(&struct_val).unwrap();
    let roundtrip: Action = serde_json::from_value(json).unwrap();
    assert_eq!(roundtrip, Action::Skip);
}

#[test]
fn action_run_interop() {
    // enum → JSON → struct
    let enum_val = Action::Run { tasks: "Check pending PRs".to_string() };
    let json = serde_json::to_value(&enum_val).unwrap();
    let struct_val: ActionSchema = serde_json::from_value(json).unwrap();
    assert_eq!(struct_val.action, "run");
    assert_eq!(struct_val.tasks, "Check pending PRs");

    // struct → JSON → enum
    let json = serde_json::to_value(&struct_val).unwrap();
    let roundtrip: Action = serde_json::from_value(json).unwrap();
    assert_eq!(roundtrip, Action::Run { tasks: "Check pending PRs".to_string() });
}
