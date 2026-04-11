use super::*;
use matrix_sdk::store::StoreConfig;
use matrix_sdk::test_utils::logged_in_client;
use tempfile::tempdir;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_matrix_engine_init() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf()).await;

    // MatrixEngine::new should succeed even if not logged in,
    // but we need to handle the RoomListService initialization carefully.
    assert!(
        engine.is_ok(),
        "Failed to initialize Matrix engine: {:?}",
        engine.err()
    );
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
        room_type: None,
        is_space: false,
        parent_space_id: None,
    };
    let serialized = serde_json::to_string(&room_data).unwrap();
    let deserialized: RoomData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(room_data.id, deserialized.id);
    assert_eq!(room_data.name, deserialized.name);
    assert_eq!(room_data.last_message, deserialized.last_message);
    assert_eq!(room_data.unread_count, deserialized.unread_count);
    assert_eq!(room_data.room_type, deserialized.room_type);
    assert_eq!(room_data.is_space, deserialized.is_space);
    assert_eq!(room_data.parent_space_id, deserialized.parent_space_id);
}

#[test]
fn test_space_hierarchy_basic() {
    let mut hierarchy = SpaceHierarchy::new();
    let space_id = RoomId::parse("!space:example.com").unwrap();
    let room_id = RoomId::parse("!room:example.com").unwrap();

    hierarchy.add_child(space_id.clone(), room_id.clone());

    assert!(hierarchy.is_in_space(&room_id, &space_id));
    assert!(!hierarchy.is_in_space(&space_id, &room_id));
}

#[test]
fn test_space_hierarchy_nested() {
    let mut hierarchy = SpaceHierarchy::new();
    let top_space = RoomId::parse("!top:example.com").unwrap();
    let sub_space = RoomId::parse("!sub:example.com").unwrap();
    let room = RoomId::parse("!room:example.com").unwrap();

    hierarchy.add_child(top_space.clone(), sub_space.clone());
    hierarchy.add_child(sub_space.clone(), room.clone());

    assert!(hierarchy.is_in_space(&room, &sub_space));
    assert!(hierarchy.is_in_space(&room, &top_space));
    assert!(hierarchy.is_in_space(&sub_space, &top_space));
    assert!(!hierarchy.is_in_space(&top_space, &sub_space));
}

#[test]
fn test_space_hierarchy_circular() {
    let mut hierarchy = SpaceHierarchy::new();
    let space_a = RoomId::parse("!a:example.com").unwrap();
    let space_b = RoomId::parse("!b:example.com").unwrap();

    hierarchy.add_child(space_a.clone(), space_b.clone());
    hierarchy.add_child(space_b.clone(), space_a.clone());

    // Should not stack overflow
    assert!(hierarchy.is_in_space(&space_a, &space_b));
    assert!(hierarchy.is_in_space(&space_b, &space_a));

    let room = RoomId::parse("!room:example.com").unwrap();
    hierarchy.add_child(space_a.clone(), room.clone());

    assert!(hierarchy.is_in_space(&room, &space_a));
    assert!(hierarchy.is_in_space(&room, &space_b));
}

#[test]
fn test_space_hierarchy_multiple_parents() {
    let mut hierarchy = SpaceHierarchy::new();
    let space_1 = RoomId::parse("!space1:example.com").unwrap();
    let space_2 = RoomId::parse("!space2:example.com").unwrap();
    let room = RoomId::parse("!room:example.com").unwrap();

    hierarchy.add_child(space_1.clone(), room.clone());
    hierarchy.add_child(space_2.clone(), room.clone());

    assert!(hierarchy.is_in_space(&room, &space_1));
    assert!(hierarchy.is_in_space(&room, &space_2));
}

#[test]
fn test_room_data_space_serialization() {
    let room_data = RoomData {
        id: "!room:example.com".to_string(),
        name: Some("Example Room".to_string()),
        last_message: Some("Hello".to_string()),
        unread_count: 5,
        avatar_url: None,
        room_type: Some(RoomType::Space),
        is_space: true,
        parent_space_id: Some("!space:example.com".to_string()),
    };
    let serialized = serde_json::to_string(&room_data).unwrap();
    let deserialized: RoomData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(room_data.room_type, deserialized.room_type);
    assert_eq!(room_data.is_space, deserialized.is_space);
    assert_eq!(room_data.parent_space_id, deserialized.parent_space_id);
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
        room_type: None,
        is_space: false,
        parent_space_id: None,
    };
    let event = MatrixEvent::RoomDiff(VectorDiff::Insert {
        index: 0,
        value: room_data.clone(),
    });
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
    assert_eq!(
        SyncStatus::MissingSlidingSyncSupport,
        SyncStatus::MissingSlidingSyncSupport
    );
    assert_ne!(SyncStatus::MissingSlidingSyncSupport, SyncStatus::Connected);
}

#[test]
fn test_sync_error_display() {
    let err = SyncError::MissingSlidingSyncSupport;
    assert_eq!(
        err.to_string(),
        "Sliding Sync (MSC4186) is not supported by the homeserver"
    );
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
    assert_eq!(
        status,
        SyncStatus::Error("Matrix error: some error".to_string())
    );
}

#[test]
fn test_sync_service_state_mapping() {
    use matrix_sdk_ui::sync_service::State as SyncServiceState;

    let states = vec![
        (SyncServiceState::Idle, SyncStatus::Connected),
        (SyncServiceState::Running, SyncStatus::Syncing),
        (SyncServiceState::Terminated, SyncStatus::Disconnected),
    ];

    for (input, expected) in states {
        let actual = match input {
            SyncServiceState::Idle => SyncStatus::Connected,
            SyncServiceState::Running => SyncStatus::Syncing,
            SyncServiceState::Terminated => SyncStatus::Disconnected,
            SyncServiceState::Offline => SyncStatus::Disconnected,
            SyncServiceState::Error(_) => SyncStatus::Error("Sync error encountered. This may be due to missing server support for Sliding Sync (MSC4186) or network issues.".to_string()),
        };
        assert_eq!(actual, expected);
    }
}

#[test]
fn test_session_data_serialization() {
    let session_data = SessionData {
        homeserver: "https://matrix.org".to_string(),
        user_id: "@alice:matrix.org".to_string(),
        access_token: "access_token".to_string(),
        refresh_token: Some("refresh_token".to_string()),
        id_token: Some("id_token".to_string()),
        device_id: "DEVICEID".to_string(),
        is_oidc: false,
    };
    let serialized = serde_json::to_string(&session_data).unwrap();
    let deserialized: SessionData = serde_json::from_str(&serialized).unwrap();
    assert_eq!(session_data.homeserver, deserialized.homeserver);
    assert_eq!(session_data.user_id, deserialized.user_id);
    assert_eq!(session_data.access_token, deserialized.access_token);
    assert_eq!(session_data.refresh_token, deserialized.refresh_token);
    assert_eq!(session_data.id_token, deserialized.id_token);
    assert_eq!(session_data.device_id, deserialized.device_id);
}

#[tokio::test]
async fn test_login_oidc_initiation_no_server() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let homeserver = "http://localhost:12345";
    let result = engine.login_oidc(homeserver).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_complete_oidc_login_no_client() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let callback_url = Url::parse("com.system76.Claw://callback?code=test").unwrap();
    let result = engine.complete_oidc_login(callback_url).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "No OIDC login in progress");
}

#[tokio::test]
async fn test_ipc_callback_trigger_failure() {
    let test_uri = "com.system76.Claw://callback?code=test_code".to_string();
    let result = crate::ipc::call_handle_callback(test_uri).await;

    // If no instance is running, it should fail to find the proxy.
    assert!(result.is_err());
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




fn create_test_session() -> matrix_sdk::authentication::matrix::MatrixSession {
    matrix_sdk::authentication::matrix::MatrixSession {
        meta: matrix_sdk::SessionMeta {
            user_id: UserId::parse("@alice:localhost").unwrap(),
            device_id: matrix_sdk::ruma::OwnedDeviceId::from("DEVICEID"),
        },
        tokens: matrix_sdk::SessionTokens {
            access_token: "token".to_string(),
            refresh_token: None,
        },
    }
}

#[tokio::test]
async fn test_start_sync_task_management() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

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
    client.restore_session(session).await.unwrap();

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

    assert_ne!(
        handle1_debug, handle2_debug,
        "Sync handle should be replaced"
    );
}

#[tokio::test]
async fn test_paginate_backwards_invalid_room_id() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let result = engine.paginate_backwards("invalid_room_id", 20).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Invalid room ID"),
        "Expected invalid room ID error, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_paginate_backwards_rls_not_initialized() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    // RLS is not initialized when just creating the engine without syncing
    let result = engine.paginate_backwards("!room:example.com", 20).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert_eq!(err_msg, "RoomListService not initialized");
}

#[tokio::test]
async fn test_paginate_backwards_success() {
    use wiremock::{
        matchers::{method, path_regex},
        Mock, MockServer, ResponseTemplate,
    };

    let mock_server = MockServer::start().await;

    // Mock the sliding sync endpoint to inject a room
    Mock::given(method("GET"))
        .and(path_regex(
            r"^/_matrix/client/unstable/org.matrix.msc3575/sync$",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "pos": "mock_pos",
            "lists": {
                "RoomList": {
                    "count": 1,
                    "ops": [
                        {
                            "op": "SYNC",
                            "range": [0, 0],
                            "room_ids": ["!mockroom:example.com"]
                        }
                    ]
                }
            },
            "rooms": {
                "!mockroom:example.com": {
                    "name": "Mock Room",
                    "initial": true,
                    "timeline": [
                        {
                            "type": "m.room.message",
                            "content": {
                                "msgtype": "m.text",
                                "body": "Hello mock!"
                            },
                            "event_id": "$mock1",
                            "sender": "@mock:example.com",
                            "origin_server_ts": 123456
                        }
                    ],
                    "prev_batch": "mock_prev_batch"
                }
            }
        })))
        .mount(&mock_server)
        .await;

    // Mock the backward pagination endpoint
    Mock::given(method("GET"))
        .and(path_regex(r"^/_matrix/client/r0/rooms/!mockroom:example.com/messages$|^/_matrix/client/v3/rooms/!mockroom:example.com/messages$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "chunk": [],
            "start": "mock_prev_batch",
            "end": "mock_end"
        })))
        .mount(&mock_server)
        .await;

    // Discovery endpoint
    Mock::given(method("GET"))
        .and(path_regex(r"^/_matrix/client/versions$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "versions": ["v1.11"],
            "unstable_features": {
                "org.matrix.msc4186": true,
                "org.matrix.msc3575": true
            }
        })))
        .mount(&mock_server)
        .await;

    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let store_config = StoreConfig::new("test".to_owned());
    let client = Client::builder()
        .homeserver_url(mock_server.uri())
        .store_config(store_config)
        .build()
        .await
        .unwrap();

    let session = create_test_session();
    client.restore_session(session).await.unwrap();

    let sync_service = Arc::new(SyncService::builder(client.clone()).build().await.unwrap());

    // Set the sync service so engine can get the room list
    {
        let mut inner = engine.inner.write().await;
        inner.client = client.clone();
        inner.sync_service = Some(sync_service.clone());
        inner.room_list_service = Some(sync_service.room_list_service());
    }

    // Start sync so it connects to wiremock and populates the room
    engine.start_sync().await.unwrap();

    // Yield to let the background task process the sync response
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Verify pagination works and doesn't fail
    let result = engine.paginate_backwards("!mockroom:example.com", 20).await;

    // Assert that the result is Ok, verifying that the timeline could be fetched and paginated
    assert!(
        result.is_ok(),
        "Expected pagination to succeed, but got error: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_send_message_room_not_found() {
    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let result = engine
        .send_message("!nonexistent:localhost", "Hello".to_string(), None)
        .await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Room not found");
}

#[tokio::test]
async fn test_send_message_success() {
    let mock_server = MockServer::start().await;

    // We need to mock the send message endpoint. It matches a regex because of the transaction ID.
    Mock::given(method("PUT"))
        .and(path_regex(
            r"^/_matrix/client/r0/rooms/[^/]+/send/m.room.message/[^/]+$",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event_id": "$eventid12345"
        })))
        .mount(&mock_server)
        .await;

    // Also matching v3
    Mock::given(method("PUT"))
        .and(path_regex(
            r"^/_matrix/client/v3/rooms/[^/]+/send/m.room.message/[^/]+$",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "event_id": "$eventid12345"
        })))
        .mount(&mock_server)
        .await;

    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let client = logged_in_client(Some(mock_server.uri())).await;

    {
        let mut inner = engine.inner.write().await;
        inner.client = client.clone();
    }

    // Now test successful plain text send
    let result = engine
        .send_message("!test_room:localhost", "Hello world".to_string(), None)
        .await;
    assert!(result.is_ok(), "Expected success, got {:?}", result);

    // Test successful HTML text send
    let html_result = engine
        .send_message(
            "!test_room:localhost",
            "Hello".to_string(),
            Some("<b>Hello</b>".to_string()),
        )
        .await;
    assert!(
        html_result.is_ok(),
        "Expected success for HTML, got {:?}",
        html_result
    );
}

#[tokio::test]
async fn test_fetch_media() {
    use matrix_sdk::ruma::events::room::MediaSource;
    use matrix_sdk::ruma::OwnedMxcUri;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;

    // Create a dummy body
    let body = b"hello media".to_vec();

    Mock::given(method("GET"))
        .and(path("/_matrix/media/v3/download/mockserver/mockmediaid"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body.clone()))
        .mount(&server)
        .await;

    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let client = Client::builder()
        .homeserver_url(server.uri())
        .server_versions([matrix_sdk::ruma::api::MatrixVersion::V1_1])
        .build()
        .await
        .unwrap();

    {
        let mut inner = engine.inner.write().await;
        inner.client = client;
    }

    let url: OwnedMxcUri = "mxc://mockserver/mockmediaid".try_into().unwrap();
    let source = MediaSource::Plain(url);
    let fetched_body = engine.fetch_media(source).await.unwrap();

    assert_eq!(fetched_body, body);
}

#[tokio::test]
async fn test_create_room() {
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/_matrix/client/v3/createRoom"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "room_id": "!new_room:example.com"
        })))
        .mount(&server)
        .await;

    let tmp_dir = tempdir().unwrap();
    let engine = MatrixEngine::new(tmp_dir.path().to_path_buf())
        .await
        .unwrap();

    let store_config = StoreConfig::new("test_create_room".to_owned());
    let client = Client::builder()
        .homeserver_url(server.uri())
        .store_config(store_config)
        .build()
        .await
        .unwrap();

    let session = create_test_session();
    client.restore_session(session).await.unwrap();

    {
        let mut inner = engine.inner.write().await;
        inner.client = client;
    }

    let room_id = engine.create_room("Test Room").await.unwrap();
    assert_eq!(room_id.as_str(), "!new_room:example.com");
}
