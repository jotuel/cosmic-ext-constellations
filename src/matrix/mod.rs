use anyhow::{Context, Result};
use matrix_sdk::{
    config::{StoreConfig, SyncSettings},
    ruma::{UserId, OwnedDeviceId},
    Client,
    matrix_auth::MatrixSession,
};
use matrix_sdk_sqlite::SqliteStateStore;
use matrix_sdk_ui::RoomListService;
use oo7::Keyring;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Disconnected,
    Syncing,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomData {
    pub id: String,
    pub name: Option<String>,
    pub last_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MatrixEvent {
    SyncStatusChanged(SyncStatus),
    RoomInserted(usize, RoomData),
    RoomRemoved(usize),
    RoomUpdated(usize, RoomData),
    RoomListReset,
}

#[derive(Serialize, Deserialize, Debug)]
struct SessionData {
    homeserver: String,
    user_id: String,
    access_token: String,
    refresh_token: Option<String>,
    device_id: String,
}

#[derive(Clone, Debug)]
pub struct MatrixEngine {
    client: Client,
    room_list_service: Arc<RoomListService>,
    data_dir: PathBuf,
}

impl MatrixEngine {
    pub async fn new(data_dir: PathBuf) -> Result<Self> {
        let store_path = data_dir.join("matrix-store.db");
        let sqlite_store = SqliteStateStore::open(&store_path, None).await?;
        let store_config = StoreConfig::default().state_store(sqlite_store);

        // We'll try to restore session later, for now just build a client with a placeholder server
        let client = Client::builder()
            .homeserver_url("https://matrix.org")
            .store_config(store_config)
            .build()
            .await?;

        let room_list_service = Arc::new(RoomListService::new(client.clone()).await?);

        Ok(Self { client, room_list_service, data_dir })
    }

    pub async fn sync(&self) -> Result<()> {
        let settings = SyncSettings::default();
        self.client.sync(settings).await.context("Sync failed")?;
        Ok(())
    }

    pub async fn login(&mut self, homeserver: &str, username: &str, password: &str) -> Result<()> {
        let homeserver_url = if homeserver.starts_with("http") {
            homeserver.to_string()
        } else {
            format!("https://{}", homeserver)
        };

        // We might need to rebuild the client with the correct homeserver
        let store_path = self.data_dir.join("matrix-store.db");
        let sqlite_store = SqliteStateStore::open(&store_path, None).await?;
        let store_config = StoreConfig::default().state_store(sqlite_store);

        self.client = Client::builder()
            .homeserver_url(&homeserver_url)
            .store_config(store_config)
            .build()
            .await?;

        self.room_list_service = Arc::new(RoomListService::new(self.client.clone()).await?);

        self.client
            .matrix_auth()
            .login_username(username, password)
            .initial_device_display_name("Claw Matrix Client")
            .send()
            .await
            .context("Failed to login")?;

        // After successful login, save the session to oo7
        let session = self.client.session().expect("Session should exist after login");
        
        let meta = session.meta();
        let access_token = session.access_token();

        let session_data = SessionData {
            homeserver: homeserver_url,
            user_id: meta.user_id.to_string(),
            access_token: access_token.to_string(),
            refresh_token: None, // Simplified for now
            device_id: meta.device_id.to_string(),
        };

        let keyring = Keyring::new().await?;
        let mut attributes = HashMap::new();
        attributes.insert("app_id", "com.system76.Claw");
        attributes.insert("type", "matrix-session");
        
        let secret = serde_json::to_vec(&session_data)?;

        keyring
            .create_item("Claw Matrix Session", &attributes, &secret, true)
            .await?;

        Ok(())
    }

    pub async fn restore_session(&mut self) -> Result<bool> {
        let keyring = Keyring::new().await?;
        let mut attributes = HashMap::new();
        attributes.insert("app_id", "com.system76.Claw");
        attributes.insert("type", "matrix-session");
        
        let items = keyring.search_items(&attributes).await?;

        if let Some(item) = items.first() {
            let secret = item.secret().await?;
            let session_data: SessionData = serde_json::from_slice(&secret)?;

            let matrix_session = MatrixSession {
                meta: matrix_sdk::SessionMeta {
                    user_id: UserId::parse(session_data.user_id.clone())?,
                    device_id: OwnedDeviceId::from(session_data.device_id),
                },
                tokens: matrix_sdk::matrix_auth::MatrixSessionTokens {
                    access_token: session_data.access_token,
                    refresh_token: session_data.refresh_token,
                },
            };

            // Rebuild client with the correct homeserver and restore session
            let store_path = self.data_dir.join("matrix-store.db");
            let sqlite_store = SqliteStateStore::open(&store_path, None).await?;
            let store_config = StoreConfig::default().state_store(sqlite_store);

            self.client = Client::builder()
                .homeserver_url(&session_data.homeserver)
                .store_config(store_config)
                .build()
                .await?;

            self.room_list_service = Arc::new(RoomListService::new(self.client.clone()).await?);

            self.client.matrix_auth().restore_session(matrix_session).await?;
            
            return Ok(true);
        }

        Ok(false)
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn room_list_service(&self) -> Arc<RoomListService> {
        self.room_list_service.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_matrix_engine_init() {
        let tmp_dir = tempdir().unwrap();
        let engine = MatrixEngine::new(tmp_dir.path().to_path_buf()).await;
        assert!(engine.is_ok());
    }
}
