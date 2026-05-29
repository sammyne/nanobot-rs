use super::*;

#[test]
fn init_creates_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory");

    let _store = GitStore::init(&path).unwrap();

    assert!(path.join(".git").exists(), ".git directory should exist");
    assert!(path.join(".gitignore").exists(), ".gitignore should exist");

    let gitignore = std::fs::read_to_string(path.join(".gitignore")).unwrap();
    assert!(gitignore.contains("!*.md"), ".gitignore should track *.md files");
}

#[test]
fn init_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory");

    let _store1 = GitStore::init(&path).unwrap();
    let _store2 = GitStore::init(&path).unwrap();

    assert!(path.join(".git").exists());

    // Should still have exactly one init commit.
    let log = _store2.log(10).unwrap();
    assert_eq!(log.len(), 1);
    assert_eq!(log[0].message, "Initialize memory repository");
}

#[test]
fn commit_and_log() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory");
    let store = GitStore::init(&path).unwrap();

    std::fs::write(path.join("MEMORY.md"), "# Memory\nFact 1\n").unwrap();
    store.commit("Add initial memory").unwrap();

    let log = store.log(10).unwrap();
    assert!(log.len() >= 2, "should have init + our commit");
    assert_eq!(log[0].message, "Add initial memory");
    assert!(!log[0].sha.is_empty());
    assert!(!log[0].timestamp.is_empty());
}

#[test]
fn commit_noop_without_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory");
    let store = GitStore::init(&path).unwrap();

    // Commit with no changes should succeed as no-op.
    store.commit("nothing changed").unwrap();

    let log = store.log(10).unwrap();
    assert_eq!(log.len(), 1, "no extra commit should be created");
}

#[test]
fn diff_shows_changes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory");
    let store = GitStore::init(&path).unwrap();

    std::fs::write(path.join("MEMORY.md"), "version 1\n").unwrap();
    store.commit("v1").unwrap();

    std::fs::write(path.join("MEMORY.md"), "version 2\n").unwrap();
    store.commit("v2").unwrap();

    let log = store.log(1).unwrap();
    let diff = store.diff(&log[0].sha).unwrap();

    assert!(diff.contains("-version 1"), "diff should show removed line");
    assert!(diff.contains("+version 2"), "diff should show added line");
}

#[test]
fn revert_restores_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("memory");
    let store = GitStore::init(&path).unwrap();

    // Commit v1
    std::fs::write(path.join("MEMORY.md"), "version 1\n").unwrap();
    store.commit("v1").unwrap();

    let log = store.log(1).unwrap();
    let v1_sha = log[0].sha.clone();

    // Commit v2
    std::fs::write(path.join("MEMORY.md"), "version 2\n").unwrap();
    store.commit("v2").unwrap();

    assert_eq!(std::fs::read_to_string(path.join("MEMORY.md")).unwrap(), "version 2\n");

    // Revert to v1
    store.revert(&v1_sha).unwrap();

    let content = std::fs::read_to_string(path.join("MEMORY.md")).unwrap();
    assert_eq!(content, "version 1\n");

    // Log should show the revert commit.
    let log = store.log(1).unwrap();
    assert!(log[0].message.contains("Revert to"));
}
