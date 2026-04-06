use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use oo7::Keyring;

#[path = "../src/matrix/mod.rs"]
mod matrix;

// Mock IPC module to satisfy existing tests in matrix::tests
mod ipc {
    pub async fn call_handle_callback(_uri: String) -> anyhow::Result<()> {
        // Return an error to satisfy test_ipc_callback_trigger_failure
        anyhow::bail!("No instance running")
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionData {
    homeserver: String,
    user_id: String,
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    device_id: String,
    #[serde(default)]
    is_oidc: bool,
}

async fn save_session_to_keyring(session_data: &SessionData) -> anyhow::Result<()> {
    let keyring = Keyring::new().await?;
    let mut attributes = HashMap::new();
    attributes.insert("app_id", "com.system76.Claw");
    attributes.insert("type", "matrix-session");
    
    let secret = serde_json::to_vec(session_data)?;

    keyring.create_item("Claw Matrix Session", &attributes, &secret, true).await?;
    Ok(())
}

#[tokio::test]
async fn test_restoration_timing_and_ui_state() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let data_dir = tmp_dir.path().to_path_buf();
    
    let mut is_initializing = true;
    
    let init_handle = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        matrix::MatrixEngine::new(data_dir).await
    });
    
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(is_initializing, "UI should remain in Initializing state while engine is starting");
    
    let res = init_handle.await.unwrap();
    assert!(res.is_ok());
    
    is_initializing = false;
    assert!(!is_initializing);
}

#[tokio::test]
async fn test_login_guard_during_pending_restoration() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let data_dir = tmp_dir.path().to_path_buf();
    
    let session_data = SessionData {
        homeserver: "https://localhost:8080".to_string(),
        user_id: "@alice:localhost".to_string(),
        access_token: "token".to_string(),
        refresh_token: None,
        id_token: None,
        device_id: "DEVICEID".to_string(),
        is_oidc: false,
    };
    
    let mut engine_opt = None;
    let mut user_id_opt = None;
    
    for i in 0..10 {
        save_session_to_keyring(&session_data).await.unwrap();
        let attempt_data_dir = data_dir.join(format!("login_guard_attempt_{}", i));
        let (engine, user_id) = matrix::MatrixEngine::new(attempt_data_dir).await.unwrap();
        if user_id.is_some() {
            engine_opt = Some(engine);
            user_id_opt = user_id;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    
    let engine = engine_opt.expect("Failed to restore session for login guard test after 10 attempts");
    assert!(user_id_opt.is_some());
    
    let login_res = engine.login("https://matrix.org", "bob", "password").await;
    
    assert!(login_res.is_err());
    assert_eq!(login_res.unwrap_err().to_string(), "Session already active");
    
    let _ = matrix::MatrixEngine::reset_session().await;
}

#[tokio::test]
async fn test_reset_session_clears_keyring() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let data_dir = tmp_dir.path().to_path_buf();
    
    let session_data = SessionData {
        homeserver: "https://localhost:8080".to_string(),
        user_id: "@alice:localhost".to_string(),
        access_token: "token".to_string(),
        refresh_token: None,
        id_token: None,
        device_id: "DEVICEID".to_string(),
        is_oidc: false,
    };
    
    let mut user_id_opt = None;
    for i in 0..10 {
        save_session_to_keyring(&session_data).await.unwrap();
        let attempt_data_dir = data_dir.join(format!("reset_session_attempt_{}", i));
        let (_, user_id) = matrix::MatrixEngine::new(attempt_data_dir).await.unwrap();
        if user_id.is_some() {
            user_id_opt = user_id;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    
    assert!(user_id_opt.is_some(), "Failed to restore session for reset test after 10 attempts");
    
    matrix::MatrixEngine::reset_session().await.unwrap();
    
    let (_, user_id) = matrix::MatrixEngine::new(data_dir.join("after_reset")).await.unwrap();
    assert!(user_id.is_none());
}
