use super::*;

#[test]
fn append_and_read() {
    let dir = tempfile::tempdir().unwrap();
    let history = History::new(dir.path());

    let c1 = history.append("first entry").unwrap();
    let c2 = history.append("second entry").unwrap();

    assert_eq!(c1, 1);
    assert_eq!(c2, 2);

    let all = history.read_all().unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].content, "first entry");
    assert_eq!(all[1].content, "second entry");
}

#[test]
fn read_since_cursor() {
    let dir = tempfile::tempdir().unwrap();
    let history = History::new(dir.path());

    history.append("a").unwrap();
    history.append("b").unwrap();
    history.append("c").unwrap();

    let since_1 = history.read_since(1).unwrap();
    assert_eq!(since_1.len(), 2);
    assert_eq!(since_1[0].content, "b");
    assert_eq!(since_1[1].content, "c");
}

#[test]
fn max_cursor_empty() {
    let dir = tempfile::tempdir().unwrap();
    let history = History::new(dir.path());
    assert_eq!(history.max_cursor().unwrap(), 0);
}

#[test]
fn compact_trims_old_entries() {
    let dir = tempfile::tempdir().unwrap();
    let history = History::new(dir.path());

    // 写入超过 MAX_ENTRIES 条
    for i in 0..1010 {
        history.append(&format!("entry {i}")).unwrap();
    }

    let all = history.read_all().unwrap();
    assert!(all.len() <= 1000, "should compact to <= 1000, got {}", all.len());
    // cursor 应该保持连续
    assert!(all.last().unwrap().cursor == 1010);
}
