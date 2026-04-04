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
        avatar_url: None,
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

    let error_status = SyncStatus::Error("test error".to_string());
    let error_event = MatrixEvent::SyncStatusChanged(error_status);
    if let MatrixEvent::SyncStatusChanged(SyncStatus::Error(e)) = error_event {
        assert_eq!(e, "test error");
    } else {
        panic!("Expected SyncStatusChanged(Error) variant");
    }

    let room_data = RoomData {
        id: "1".to_string(),
        name: None,
        last_message: None,
        unread_count: 0,
        avatar_url: None,
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
fn test_sync_status_error_propagation() {
    let error_msg = "Sync error encountered. This may be due to missing server support for Sliding Sync (MSC4186) or network issues.";
    let status = SyncStatus::Error(error_msg.to_string());
    
    // Verify SyncStatus variant and payload
    if let SyncStatus::Error(msg) = &status {
        assert_eq!(msg, error_msg);
    } else {
        panic!("Expected SyncStatus::Error variant");
    }

    // Verify MatrixEvent carries the error status
    let event = MatrixEvent::SyncStatusChanged(status.clone());
    if let MatrixEvent::SyncStatusChanged(SyncStatus::Error(msg)) = event {
        assert_eq!(msg, error_msg);
    } else {
        panic!("Expected MatrixEvent::SyncStatusChanged(SyncStatus::Error) variant");
    }
}

#[test]
fn test_sync_status_equality() {
    assert_eq!(SyncStatus::Disconnected, SyncStatus::Disconnected);
    assert_eq!(SyncStatus::Syncing, SyncStatus::Syncing);
    assert_eq!(SyncStatus::Connected, SyncStatus::Connected);
    assert_eq!(
        SyncStatus::Error("error".to_string()),
        SyncStatus::Error("error".to_string())
    );
    assert_ne!(
        SyncStatus::Error("error 1".to_string()),
        SyncStatus::Error("error 2".to_string())
    );
    assert_ne!(SyncStatus::Connected, SyncStatus::Syncing);
    assert_eq!(SyncStatus::MissingSlidingSyncSupport, SyncStatus::MissingSlidingSyncSupport);
    assert_ne!(SyncStatus::MissingSlidingSyncSupport, SyncStatus::Connected);
}

#[test]
fn test_sync_error_display() {
    let err = SyncError::MissingSlidingSyncSupport;
    assert_eq!(err.to_string(), "Sliding Sync (MSC4186) is not supported by the homeserver");
}

#[test]
fn test_sync_status_missing_support() {
    let status = SyncStatus::MissingSlidingSyncSupport;
    let event = MatrixEvent::SyncStatusChanged(status);
    if let MatrixEvent::SyncStatusChanged(SyncStatus::MissingSlidingSyncSupport) = event {
        // success
    } else {
        panic!("Expected SyncStatus::MissingSlidingSyncSupport");
    }
}

#[test]
fn test_sync_error_to_status_mapping() {
    let err = SyncError::MissingSlidingSyncSupport;
    let status = match err {
        SyncError::MissingSlidingSyncSupport => SyncStatus::MissingSlidingSyncSupport,
        _ => SyncStatus::Error(err.to_string()),
    };
    assert_eq!(status, SyncStatus::MissingSlidingSyncSupport);

    let err = SyncError::Matrix("some error".to_string());
    let status = match err {
        SyncError::MissingSlidingSyncSupport => SyncStatus::MissingSlidingSyncSupport,
        _ => SyncStatus::Error(err.to_string()),
    };
    assert_eq!(status, SyncStatus::Error("Matrix error: some error".to_string()));
}

#[test]
fn test_sync_service_state_mapping() {
    use matrix_sdk_ui::sync_service::State as SyncServiceState;

    let states = vec![
        (SyncServiceState::Idle, SyncStatus::Connected),
        (SyncServiceState::Running, SyncStatus::Syncing),
        (SyncServiceState::Terminated, SyncStatus::Disconnected),
        (SyncServiceState::Error, SyncStatus::Error("Sync error encountered. This may be due to missing server support for Sliding Sync (MSC4186) or network issues.".to_string())),
    ];

    for (input, expected) in states {
        let actual = match input {
            SyncServiceState::Idle => SyncStatus::Connected,
            SyncServiceState::Running => SyncStatus::Syncing,
            SyncServiceState::Terminated => SyncStatus::Disconnected,
            SyncServiceState::Error => SyncStatus::Error("Sync error encountered. This may be due to missing server support for Sliding Sync (MSC4186) or network issues.".to_string()),
        };
        assert_eq!(actual, expected);
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

#[test]
fn test_backoff_logic() {
    let mut backoff = Backoff::new(2, 60);
    assert_eq!(backoff.next(), 2);
    assert_eq!(backoff.next(), 4);
    assert_eq!(backoff.next(), 8);
    assert_eq!(backoff.next(), 16);
    assert_eq!(backoff.next(), 32);
    assert_eq!(backoff.next(), 60);
    assert_eq!(backoff.next(), 60);
}

fn create_test_session() -> matrix_sdk::matrix_auth::MatrixSession {
    matrix_sdk::matrix_auth::MatrixSession {
        meta: matrix_sdk::SessionMeta {
            user_id: UserId::parse("@alice:localhost").unwrap(),
            device_id: matrix_sdk::ruma::OwnedDeviceId::from("DEVICEID"),
        },
        tokens: matrix_sdk::matrix_auth::MatrixSessionTokens {
            access_token: "token".to_string(),
            refresh_token: None,
        },
    }
}

#[tokio::test]
async fn test_start_sync_task_management() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf()).await.unwrap();
    
    // We need a real-ish client to build a SyncService
    let store_config = StoreConfig::new("test".to_owned());
    let client = Client::builder()
        .homeserver_url("https://localhost:8080")
        .store_config(store_config)
        .build()
        .await
        .unwrap();
    
    // Set a dummy session so SyncService::builder doesn't fail
    let session = create_test_session();
    client.matrix_auth().restore_session(session).await.unwrap();

    let sync_service = Arc::new(SyncService::builder(client).build().await.unwrap());
    
    {
        let mut inner = engine.inner.write().await;
        inner.sync_service = Some(sync_service);
    }

    // Call start_sync first time
    let _ = engine.start_sync().await;
    let handle1_debug = {
        let inner = engine.inner.read().await;
        format!("{:?}", inner.sync_handle)
    };

    // Call start_sync second time - should replace the handle
    let _ = engine.start_sync().await;
    let handle2_debug = {
        let inner = engine.inner.read().await;
        format!("{:?}", inner.sync_handle)
    };

    assert_ne!(handle1_debug, handle2_debug, "Sync handle should be replaced");
}
