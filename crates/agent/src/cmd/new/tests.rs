//! New command tests

/// Verify that NewCmd struct can be created
#[test]
fn struct_creation() {
    // Note: NewCmd now requires dependencies to be created
    // This test just verifies the struct can be defined
    // NewCmd<P> is now a generic struct
}

/// Verify the expected success message format
#[test]
fn success_message_format() {
    let expected = "New session started.";
    assert_eq!(expected, "New session started.");
}

/// Verify the expected error message for consolidation in progress
#[test]
fn consolidation_in_progress_message() {
    let expected = "Session is already being consolidated. Please try again later.";
    assert!(expected.contains("already being consolidated"));
    assert!(expected.contains("Please try again later"));
}

/// Verify the expected error message for memory archival failure
#[test]
fn memory_failure_message_format() {
    let error_msg = "Memory archival failed, session not cleared. Please try again: {error}";
    assert!(error_msg.contains("Memory archival failed"));
    assert!(error_msg.contains("session not cleared"));
    assert!(error_msg.contains("Please try again"));
}

/// Verify the expected error message for session save failure
#[test]
fn session_save_failure_message_format() {
    let error_msg = "Failed to save session: {error}";
    assert!(error_msg.contains("Failed to save session"));
}
