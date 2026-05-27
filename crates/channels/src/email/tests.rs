use super::*;

// ---- normalize_address ----

#[test]
fn normalize_plain_address() {
    assert_eq!(normalize_address("User@Example.COM"), "user@example.com");
}

#[test]
fn normalize_angle_bracket_address() {
    assert_eq!(normalize_address("John Doe <john@example.com>"), "john@example.com");
}

#[test]
fn normalize_bare_angle_brackets() {
    assert_eq!(normalize_address("<alice@test.org>"), "alice@test.org");
}

#[test]
fn normalize_empty_string() {
    assert_eq!(normalize_address(""), "");
    assert_eq!(normalize_address("   "), "");
}

#[test]
fn normalize_no_at_sign() {
    assert_eq!(normalize_address("not-an-email"), "");
}

// ---- html_to_text ----

#[test]
fn html_br_and_p_tags() {
    let html = "Hello<br>World<br/>End</p>Next";
    let text = html_to_text(html);
    assert!(text.contains("Hello\nWorld\nEnd\nNext"));
}

#[test]
fn html_strip_tags() {
    let html = "<b>bold</b> and <i>italic</i>";
    assert_eq!(html_to_text(html), "bold and italic");
}

#[test]
fn html_decode_entities() {
    let html = "&amp; &lt; &gt; &quot; &#39;";
    assert_eq!(html_to_text(html), "& < > \" '");
}

// ---- check_authentication_results ----

#[test]
fn auth_results_both_pass() {
    let raw = b"Authentication-Results: mx.example.com; spf=pass; dkim=pass\r\n\r\nBody";
    let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
    let (spf, dkim) = check_authentication_results(&msg);
    assert!(spf);
    assert!(dkim);
}

#[test]
fn auth_results_spf_fail() {
    let raw = b"Authentication-Results: mx.example.com; spf=fail; dkim=pass\r\n\r\nBody";
    let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
    let (spf, dkim) = check_authentication_results(&msg);
    assert!(!spf);
    assert!(dkim);
}

#[test]
fn auth_results_missing() {
    let raw = b"Subject: test\r\n\r\nBody";
    let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
    let (spf, dkim) = check_authentication_results(&msg);
    assert!(!spf);
    assert!(!dkim);
}

// ---- extract_text_body ----

#[test]
fn extract_plain_text_body() {
    let raw = b"Content-Type: text/plain\r\n\r\nHello world";
    let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
    assert_eq!(extract_text_body(&msg), "Hello world");
}

#[test]
fn extract_html_body_fallback() {
    let raw = b"Content-Type: text/html\r\n\r\n<p>Hello <b>world</b></p>";
    let msg = mail_parser::MessageParser::default().parse(raw).unwrap();
    let text = extract_text_body(&msg);
    assert!(text.contains("Hello world"));
}

// ---- is_allowed ----

#[test]
fn allowed_empty_list_allows_all() {
    assert!(is_allowed(&[], "anyone@example.com"));
}

#[test]
fn allowed_match_case_insensitive() {
    let list = vec!["User@Example.COM".to_string()];
    assert!(is_allowed(&list, "user@example.com"));
}

#[test]
fn allowed_no_match() {
    let list = vec!["allowed@example.com".to_string()];
    assert!(!is_allowed(&list, "other@example.com"));
}

// ---- collect_self_addresses ----

#[test]
fn self_addresses_dedup() {
    let config = EmailConfig {
        imap: nanobot_config::ImapConfig { username: "me@example.com".to_string(), ..Default::default() },
        smtp: nanobot_config::SmtpConfig {
            username: "me@example.com".to_string(),
            from_address: "Me <me@example.com>".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let addrs = collect_self_addresses(&config);
    assert_eq!(addrs.len(), 1);
    assert!(addrs.contains("me@example.com"));
}

// ---- remember_processed_uid ----

#[test]
fn uid_eviction_on_overflow() {
    let mut processed = HashSet::new();
    let mut cycle = HashSet::new();

    // 填充到超过上限
    for i in 0..MAX_PROCESSED_UIDS + 10 {
        remember_processed_uid(&i.to_string(), true, &mut cycle, &mut processed);
    }

    // 淘汰后应约为上限的一半
    assert!(processed.len() <= MAX_PROCESSED_UIDS / 2 + 10);
}

#[test]
fn uid_no_dedupe_skips_processed() {
    let mut processed = HashSet::new();
    let mut cycle = HashSet::new();

    remember_processed_uid("uid1", false, &mut cycle, &mut processed);
    assert!(cycle.contains("uid1"));
    assert!(processed.is_empty());
}

// ---- reply_subject ----

#[test]
fn reply_subject_adds_prefix() {
    assert_eq!(reply_subject("Hello", "Re: "), "Re: Hello");
}

#[test]
fn reply_subject_keeps_existing_re() {
    assert_eq!(reply_subject("Re: Hello", "Re: "), "Re: Hello");
    assert_eq!(reply_subject("RE: Hello", "Re: "), "RE: Hello");
}

#[test]
fn reply_subject_empty_uses_default() {
    assert_eq!(reply_subject("", "Re: "), "Re: nanobot reply");
}

// ---- format_imap_date ----

#[test]
fn imap_date_format() {
    let date = chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
    assert_eq!(format_imap_date(&date), "10-Mar-2026");
}

#[test]
fn imap_date_january() {
    let date = chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    assert_eq!(format_imap_date(&date), "05-Jan-2026");
}

// ---- is_stale_imap_error / is_missing_mailbox_error ----

#[test]
fn stale_error_detection() {
    let err = imap::Error::Bad("disconnected for inactivity".to_string());
    assert!(is_stale_imap_error(&err));
}

#[test]
fn missing_mailbox_detection() {
    let err = imap::Error::Bad("mailbox doesn't exist".to_string());
    assert!(is_missing_mailbox_error(&err));
}

#[test]
fn non_stale_error() {
    let err = imap::Error::Bad("authentication failed".to_string());
    assert!(!is_stale_imap_error(&err));
    assert!(!is_missing_mailbox_error(&err));
}
