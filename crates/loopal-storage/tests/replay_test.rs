use loopal_message::Message;
use loopal_storage::entry::{Marker, TaggedEntry};
use loopal_storage::replay;

fn msg(text: &str) -> TaggedEntry {
    TaggedEntry::Message(Message::user(text))
}

fn clear_marker() -> TaggedEntry {
    TaggedEntry::Marker(Marker::Clear {
        timestamp: "t".into(),
    })
}

fn compact_marker(keep_last: usize) -> TaggedEntry {
    TaggedEntry::Marker(Marker::CompactTo {
        keep_last,
        timestamp: "t".into(),
    })
}

#[test]
fn replay_empty_returns_empty() {
    assert!(replay(vec![]).is_empty());
}

#[test]
fn replay_messages() {
    let entries = vec![msg("a"), msg("b"), msg("c")];
    let result = replay(entries);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text_content(), "a");
    assert_eq!(result[2].text_content(), "c");
}

#[test]
fn clear_discards_preceding() {
    let entries = vec![msg("a"), msg("b"), clear_marker(), msg("c")];
    let result = replay(entries);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text_content(), "c");
}

#[test]
fn clear_at_end_yields_empty() {
    let entries = vec![msg("a"), msg("b"), clear_marker()];
    let result = replay(entries);
    assert!(result.is_empty());
}

#[test]
fn compact_keeps_last_n() {
    let entries = vec![
        msg("1"), msg("2"), msg("3"), msg("4"), msg("5"),
        compact_marker(2),
    ];
    let result = replay(entries);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text_content(), "4");
    assert_eq!(result[1].text_content(), "5");
}

#[test]
fn compact_with_more_keep_than_messages() {
    let entries = vec![msg("a"), compact_marker(10)];
    let result = replay(entries);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text_content(), "a");
}

#[test]
fn interleaved_markers() {
    let entries = vec![
        msg("a"), msg("b"),
        clear_marker(),
        msg("c"), msg("d"), msg("e"),
        compact_marker(1),
        msg("f"),
    ];
    let result = replay(entries);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text_content(), "e");
    assert_eq!(result[1].text_content(), "f");
}
