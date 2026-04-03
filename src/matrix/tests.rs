use super::*;
use tempfile::tempdir;

#[tokio::test]
async fn test_matrix_engine_init() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf()).await;
    
    // MatrixEngine::new should succeed even if not logged in, 
    // but we need to handle the RoomListService initialization carefully.
    assert!(engine.is_ok(), "Failed to initialize Matrix engine: {:?}", engine.err());
}

#[test]
fn test_markdown_to_html() {
    let markdown = "# Hello\nThis is **bold** and *italic*.";
    let html = markdown_to_html(markdown);
    assert!(html.contains("<h1>Hello</h1>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("<em>italic</em>"));
}

#[test]
fn test_room_data_serialization() {
    let room_data = RoomData {
        id: "!room:example.com".to_string(),
        name: Some("Example Room".to_string()),
        last_message: Some("Hello".to_string()),
        unread_count: 5,
    };
    let serialized = serde_json::to_string(&room_data).unwrap();
    let deserialized: RoomData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(room_data.id, deserialized.id);
    assert_eq!(room_data.name, deserialized.name);
    assert_eq!(room_data.last_message, deserialized.last_message);
    assert_eq!(room_data.unread_count, deserialized.unread_count);
}

#[test]
fn test_matrix_event_variants() {
    let status = SyncStatus::Syncing;
    let event = MatrixEvent::SyncStatusChanged(status);
    if let MatrixEvent::SyncStatusChanged(s) = event {
        assert_eq!(s, SyncStatus::Syncing);
    } else {
        panic!("Expected SyncStatusChanged variant");
    }

    let room_data = RoomData {
        id: "1".to_string(),
        name: None,
        last_message: None,
        unread_count: 0,
    };
    let event = MatrixEvent::RoomDiff(VectorDiff::Insert { index: 0, value: room_data.clone() });
    if let MatrixEvent::RoomDiff(VectorDiff::Insert { index, value }) = event {
        assert_eq!(index, 0);
        assert_eq!(value.id, "1");
    } else {
        panic!("Expected RoomDiff(Insert) variant");
    }
}

#[test]
fn test_timeline_diff_variant() {
    let diff: TimelineDiff<TimelineItem> = VectorDiff::Clear;
    let event = MatrixEvent::TimelineDiff(diff);
    if let MatrixEvent::TimelineDiff(d) = event {
        match d {
            VectorDiff::Clear => (),
            _ => panic!("Expected Clear variant"),
        }
    } else {
        panic!("Expected TimelineDiff variant");
    }
}
