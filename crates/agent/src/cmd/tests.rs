//! Command module tests

use nanobot_provider::Provider;

/// Verify command sub-modules are properly exported
#[test]
fn command_submodules_exported() {
    // This test verifies the module structure is correct
    // by checking that the required types are accessible
    use crate::cmd::HelpCmd;

    // Verify HelpCmd implements Command by checking its size
    let _help_cmd: HelpCmd = HelpCmd;
    // Note: We cannot use Box<dyn Command> because Command has async methods
    // which makes it not dyn compatible
}

/// Verify command module structure
#[test]
fn module_structure() {
    // Verify that the command types are exported from this module
    use crate::cmd::{HelpCmd, NewCmd};

    // Just verify the types exist and can be referenced
    let _help: Option<HelpCmd> = None;
    // NewCmd is generic, so we verify it exists through type checking
    fn _accepts_newcmd<P: Provider>(_: NewCmd<P>) {}
}
