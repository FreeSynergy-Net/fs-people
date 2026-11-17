//! B4: the export is always vCard 4.0, also for a card that came in as 3.0 (or 2.1-looking text),
//! and the stored original stays exactly as it came.
//!
//! Pattern: Round trip with its own oracle (the exact input text, a fixed list of versions).
//! Mutant: copy the `VERSION` line of the original into the export: this test is red.

mod common;

use common::repo;

const V3: &str = "BEGIN:VCARD\r\nVERSION:3.0\r\nUID:urn:uuid:01a1aaef-ba80-7f7a-8dff-0000000000b4\r\nFN:Bea Quill\r\nN:Quill;Bea;;;\r\nEMAIL;TYPE=INTERNET,WORK:bea@example.org\r\nTEL;TYPE=CELL:+49 30 5550142\r\nEND:VCARD\r\n";

#[tokio::test]
async fn a_card_imported_as_3_0_is_exported_as_4_0_and_the_original_is_unchanged() {
    let r = repo().await;
    let report = r.import_vcard(V3).await.unwrap();
    assert_eq!(report.created.len(), 1, "{report:?}");
    let id = report.created[0];
    let text = r.export_vcard(id).await.unwrap();
    let versions: Vec<&str> = text.lines().filter(|l| l.starts_with("VERSION:")).collect();
    assert_eq!(versions, ["VERSION:4.0"], "{text}");
    let stored = r.get(id).await.unwrap().unwrap();
    let original = stored.original.expect("the original is kept");
    assert_eq!(original.media_type, "text/vcard");
    assert_eq!(std::str::from_utf8(&original.bytes).unwrap(), V3);
    // Exporting twice changes nothing about the original either.
    let _ = r.export_vcard(id).await.unwrap();
    let again = r.get(id).await.unwrap().unwrap().original.unwrap();
    assert_eq!(std::str::from_utf8(&again.bytes).unwrap(), V3);
}

#[tokio::test]
async fn the_content_survives_the_version_change() {
    let r = repo().await;
    let id = r.import_vcard(V3).await.unwrap().created[0];
    let text = r.export_vcard(id).await.unwrap();
    assert!(text.contains("Bea Quill"), "{text}");
    assert!(text.contains("bea@example.org"), "{text}");
}
