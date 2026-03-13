use super::*;

#[test]
fn is_env_set_existing() {
    // HOME is usually set on Unix systems
    #[cfg(unix)]
    {
        assert!(is_env_set("HOME") || is_env_set("USER"));
    }
}

#[test]
fn is_env_set_nonexistent() {
    assert!(!is_env_set("DEFINITELY_NOT_SET_12345"));
}

#[test]
fn check_requirements_empty() {
    let requires = Requires::default();
    assert!(check_requirements(&requires));
}

#[test]
fn check_requirements_missing_bin() {
    let requires = Requires { bins: vec!["definitely_not_a_real_command_12345".to_string()], env: vec![] };
    assert!(!check_requirements(&requires));
}

#[test]
fn get_missing_requirements_empty() {
    let requires = Requires::default();
    let missing = get_missing_requirements(&requires);
    assert!(missing.is_empty());
}

#[test]
fn get_missing_requirements_with_missing() {
    let requires = Requires { bins: vec!["fake_cmd_xyz".to_string()], env: vec!["FAKE_ENV_XYZ".to_string()] };
    let missing = get_missing_requirements(&requires);
    assert_eq!(missing.len(), 2);
    assert!(missing[0].contains("fake_cmd_xyz"));
    assert!(missing[1].contains("FAKE_ENV_XYZ"));
}
