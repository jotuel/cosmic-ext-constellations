use crate::{
    matrix, redact_url, ApplyVectorDiffExt, Constellations, ConstellationsItem, MediaSource,
    Message, OwnedRoomId, Url,
};
use cosmic::{Action, Application, Task};

impl Constellations {
    pub fn handle_engine_ready(
        &mut self,
        res: Result<matrix::MatrixEngine, matrix::SyncError>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        match res {
            Ok(engine) => {
                self.matrix = Some(engine.clone());
                Task::perform(
                    async move {
                        let did_restore = engine.restore_session().await.unwrap_or(false);
                        if did_restore {
                            let user_id = engine.client().await.user_id().map(|u| u.to_string());
                            let sync_res = engine.start_sync().await;
                            (user_id, sync_res)
                        } else {
                            (
                                None,
                                Err(matrix::SyncError::Generic(
                                    "No session to restore".to_string(),
                                )),
                            )
                        }
                    },
                    |(user_id, sync_res)| {
                        if let Some(uid) = user_id {
                            Action::from(Message::UserReady(Some(uid), sync_res))
                        } else {
                            Action::from(Message::UserReady(None, sync_res))
                        }
                    },
                )
            }
            Err(e) => {
                self.error = Some(format!("Failed to initialize Matrix engine: {}", e));
                self.is_initializing = false;
                Task::none()
            }
        }
    }

    pub fn handle_user_ready(
        &mut self,
        user_id: Option<String>,
        sync_res: Result<(), matrix::SyncError>,
    ) -> Task<Action<Message>> {
        self.user_id = user_id;
        self.is_initializing = false;
        let title_task = self.update_title();
        if self.user_id.is_none() {
            return title_task;
        }

        match sync_res {
            Ok(_) => {}
            Err(matrix::SyncError::MissingSlidingSyncSupport) => {
                self.sync_status = matrix::SyncStatus::MissingSlidingSyncSupport;
            }
            Err(e) => {
                self.sync_status = matrix::SyncStatus::Error(e.to_string());
            }
        }
        title_task
    }

    pub fn handle_timeline_diff(
        &mut self,
        diff: eyeball_im::VectorDiff<std::sync::Arc<matrix::TimelineItem>>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        let mut tasks = Vec::new();
        let mut check_item = |item: &std::sync::Arc<matrix::TimelineItem>| {
            if let Some(event) = item.as_event() {
                if let matrix_sdk_ui::timeline::TimelineDetails::Ready(profile) =
                    event.sender_profile()
                {
                    if let Some(avatar_url) = &profile.avatar_url {
                        let url_str = avatar_url.to_string();
                        if !self.media_cache.contains_key(&url_str) {
                            if let Some(matrix) = &self.matrix {
                                let matrix_clone = matrix.clone();
                                let mxc_url = url_str.clone();
                                let source = matrix_sdk::ruma::events::room::MediaSource::Plain(
                                    avatar_url.clone(),
                                );
                                tasks.push(cosmic::iced::Task::perform(
                                    async move {
                                        matrix_clone
                                            .fetch_media(source)
                                            .await
                                            .map_err(|e| e.to_string())
                                    },
                                    move |res| Message::MediaFetched(mxc_url.clone(), res).into(),
                                ));
                            }
                        }
                    }
                }
            }
        };

        match &diff {
            eyeball_im::VectorDiff::Insert { value, .. } => check_item(value),
            eyeball_im::VectorDiff::Set { value, .. } => check_item(value),
            eyeball_im::VectorDiff::PushBack { value } => check_item(value),
            eyeball_im::VectorDiff::PushFront { value } => check_item(value),
            eyeball_im::VectorDiff::Append { values } => values.iter().for_each(&mut check_item),
            eyeball_im::VectorDiff::Reset { values } => values.iter().for_each(&mut check_item),
            _ => {}
        }

        let mapped_diff = match diff {
            eyeball_im::VectorDiff::Insert { index, value } => eyeball_im::VectorDiff::Insert {
                index,
                value: ConstellationsItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::Set { index, value } => eyeball_im::VectorDiff::Set {
                index,
                value: ConstellationsItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::PushBack { value } => eyeball_im::VectorDiff::PushBack {
                value: ConstellationsItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::PushFront { value } => eyeball_im::VectorDiff::PushFront {
                value: ConstellationsItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::Append { values } => eyeball_im::VectorDiff::Append {
                values: values
                    .into_iter()
                    .map(|v| ConstellationsItem::new(v, self.user_id.as_deref()))
                    .collect(),
            },
            eyeball_im::VectorDiff::Reset { values } => eyeball_im::VectorDiff::Reset {
                values: values
                    .into_iter()
                    .map(|v| ConstellationsItem::new(v, self.user_id.as_deref()))
                    .collect(),
            },
            eyeball_im::VectorDiff::Remove { index } => eyeball_im::VectorDiff::Remove { index },
            eyeball_im::VectorDiff::PopBack => eyeball_im::VectorDiff::PopBack,
            eyeball_im::VectorDiff::PopFront => eyeball_im::VectorDiff::PopFront,
            eyeball_im::VectorDiff::Clear => eyeball_im::VectorDiff::Clear,
            eyeball_im::VectorDiff::Truncate { length } => {
                eyeball_im::VectorDiff::Truncate { length }
            }
        };

        self.timeline_items.apply_diff(mapped_diff);

        if !tasks.is_empty() {
            cosmic::iced::Task::batch(tasks)
        } else {
            Task::none()
        }
    }

    pub fn handle_matrix_event(
        &mut self,
        event: matrix::MatrixEvent,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        match event {
            matrix::MatrixEvent::SyncStatusChanged(status) => {
                self.sync_status = status;
                Task::none()
            }
            matrix::MatrixEvent::SyncIndicatorChanged(show) => {
                self.is_sync_indicator_active = show;
                Task::none()
            }
            matrix::MatrixEvent::RoomDiff(diff) => {
                self.room_list.apply_diff(diff);
                self.update_filtered_rooms();
                self.update_title()
            }
            matrix::MatrixEvent::TimelineDiff(diff) => self.handle_timeline_diff(diff),
            matrix::MatrixEvent::TimelineReset => {
                self.timeline_items.clear();
                Task::none()
            }
            matrix::MatrixEvent::ReactionAdded { .. } => {
                // For now, we don't do anything specific as reactions are handled via TimelineDiff
                Task::none()
            }
        }
    }

    pub fn handle_load_more(&mut self) -> Task<Action<<Constellations as Application>::Message>> {
        if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
            let matrix = matrix.clone();
            let room_id = room_id.clone();
            Task::perform(
                async move {
                    matrix
                        .paginate_backwards(&room_id, 20)
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::LoadMoreFinished(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_send_message(
        &mut self,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
            let body = self.composer_text.clone();
            if body.is_empty() {
                return Task::none();
            }

            let html_body = matrix::markdown_to_html(&body);

            let matrix = matrix.clone();
            let room_id = room_id.clone();

            Task::perform(
                async move {
                    matrix
                        .send_message(&room_id, body, Some(html_body))
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::MessageSent(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_create_room(
        &mut self,
        name: String,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            Task::perform(
                async move {
                    matrix
                        .create_room(&name)
                        .await
                        .map(|id| id.to_string())
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::RoomCreated(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_select_space(
        &mut self,
        space_id: Option<OwnedRoomId>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        self.selected_space = space_id.clone();

        let mut tasks = Vec::new();

        if let Some(matrix) = &self.matrix {
            let matrix_clone = matrix.clone();
            let sid = space_id.clone();
            tasks.push(Task::perform(
                async move {
                    let _ = matrix_clone.update_room_list_filter(sid).await;
                },
                |_| Action::from(Message::NoOp),
            ));

            if let Some(space_id) = space_id {
                let matrix_clone = matrix.clone();
                tasks.push(Task::perform(
                    async move {
                        matrix_clone
                            .get_space_children(space_id.as_str())
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |res| Action::from(Message::SpaceChildrenFetched(res)),
                ));
            } else {
                self.other_rooms.clear();
            }
        }

        self.update_filtered_rooms();
        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    pub fn handle_space_children_fetched(
        &mut self,
        res: Result<Vec<matrix::RoomData>, String>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        match res {
            Ok(children) => {
                // First, update the filtered_room_list because the hierarchy in matrix engine was updated
                self.update_filtered_rooms();

                // Now filter out rooms that are already in room_list (i.e. joined rooms)
                let joined_ids: std::collections::HashSet<String> =
                    self.room_list.iter().map(|r| r.id.to_string()).collect();
                self.other_rooms = children
                    .into_iter()
                    .filter(|r| !joined_ids.contains(r.id.as_ref()) && !r.is_space)
                    .collect();
            }
            Err(e) => {
                self.error = Some(format!("Failed to fetch space children: {}", e));
            }
        }
        Task::none()
    }

    pub fn handle_fetch_media(
        &mut self,
        source: MediaSource,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            let mxc_url = match &source {
                MediaSource::Plain(uri) => uri.to_string(),
                MediaSource::Encrypted(file) => file.url.to_string(),
            };
            Task::perform(
                async move { matrix.fetch_media(source).await.map_err(|e| e.to_string()) },
                move |res| Action::from(Message::MediaFetched(mxc_url, res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_media_fetched(
        &mut self,
        mxc_url: String,
        res: Result<Vec<u8>, String>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        match res {
            Ok(data) => {
                self.media_cache.insert(
                    mxc_url,
                    cosmic::iced::widget::image::Handle::from_bytes(data),
                );
            }
            Err(e) => {
                self.error = Some(format!("Failed to fetch media: {}", e));
            }
        }
        Task::none()
    }

    pub fn handle_toggle_login_mode(
        &mut self,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        self.is_registering_mode = !self.is_registering_mode;
        self.error = None;
        Task::none()
    }

    pub fn handle_submit_register(
        &mut self,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            self.is_registering = true;
            self.error = None;
            self.sync_status = matrix::SyncStatus::Disconnected;
            let matrix = matrix.clone();
            let homeserver = self.login_homeserver.clone();
            let username = self.login_username.clone();
            let password = self.login_password.clone();
            self.login_password.clear();

            Task::perform(
                async move {
                    matrix.register(&homeserver, &username, &password).await?;
                    let user_id = matrix
                        .client()
                        .await
                        .user_id()
                        .map(|u| u.to_string())
                        .ok_or_else(|| {
                            anyhow::anyhow!("Failed to get user ID after registration")
                        })?;
                    matrix.start_sync().await?;
                    Ok(user_id)
                },
                |res: Result<String, anyhow::Error>| {
                    Action::from(Message::RegisterFinished(
                        res.map_err(matrix::SyncError::from),
                    ))
                },
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_register_finished(
        &mut self,
        res: Result<String, matrix::SyncError>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        self.is_registering = false;
        match res {
            Ok(user_id) => {
                self.user_id = Some(user_id);
                self.login_homeserver.clear();
                self.login_username.clear();
                self.login_password.clear();
                self.error = None;
                self.update_title()
            }
            Err(e) => {
                self.error = Some(format!("Registration failed: {}", e));
                Task::none()
            }
        }
    }

    pub fn handle_submit_login(
        &mut self,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            self.is_logging_in = true;
            self.error = None;
            self.sync_status = matrix::SyncStatus::Disconnected;
            let matrix = matrix.clone();
            let homeserver = self.login_homeserver.clone();
            let username = self.login_username.clone();
            let password = self.login_password.clone();
            self.login_password.clear();

            Task::perform(
                async move {
                    matrix.login(&homeserver, &username, &password).await?;
                    let user_id = matrix
                        .client()
                        .await
                        .user_id()
                        .map(|u| u.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Failed to get user ID after login"))?;
                    matrix.start_sync().await?;
                    Ok(user_id)
                },
                |res: Result<String, anyhow::Error>| {
                    Action::from(Message::LoginFinished(res.map_err(matrix::SyncError::from)))
                },
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_login_finished(
        &mut self,
        res: Result<String, matrix::SyncError>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        self.is_logging_in = false;
        self.is_oidc_logging_in = false;
        match res {
            Ok(user_id) => {
                self.user_id = Some(user_id);
            }
            Err(matrix::SyncError::MissingSlidingSyncSupport) => {
                self.sync_status = matrix::SyncStatus::MissingSlidingSyncSupport;
            }
            Err(e) => {
                self.error = Some(format!("Login failed: {}", e));
            }
        }
        Task::none()
    }

    pub fn handle_submit_oidc_login(
        &mut self,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            self.is_oidc_logging_in = true;
            self.error = None;
            let matrix = matrix.clone();
            let homeserver = self.login_homeserver.clone();
            Task::perform(
                async move {
                    matrix
                        .login_oidc(&homeserver)
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::OidcLoginStarted(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_oidc_login_started(
        &mut self,
        res: Result<Url, String>,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        match res {
            Ok(url) => {
                tracing::info!("Opening URL: {}", redact_url(&url));
                let _ = open::that(url.as_str());
            }
            Err(e) => {
                self.is_oidc_logging_in = false;
                self.error = Some(format!("OIDC login failed to start: {}", e));
            }
        }
        Task::none()
    }

    pub fn handle_oidc_callback(
        &mut self,
        url: Url,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            self.is_oidc_logging_in = true;
            self.error = None;
            let matrix = matrix.clone();
            Task::perform(
                async move {
                    matrix.complete_oidc_login(url).await?;
                    let user_id = matrix
                        .client()
                        .await
                        .user_id()
                        .map(|u| u.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Failed to get user ID after OIDC login"))?;
                    matrix.start_sync().await?;
                    Ok(user_id)
                },
                |res: Result<String, anyhow::Error>| {
                    Action::from(Message::LoginFinished(res.map_err(matrix::SyncError::from)))
                },
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_logout(&mut self) -> Task<Action<<Constellations as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            return Task::perform(
                async move {
                    let _ = matrix.logout().await;
                },
                |_| Action::from(Message::LogoutFinished),
            );
        }
        Task::none()
    }

    pub fn handle_logout_finished(
        &mut self,
    ) -> Task<Action<<Constellations as Application>::Message>> {
        self.user_id = None;
        self.matrix = None;
        self.sync_status = matrix::SyncStatus::Disconnected;
        self.room_list.clear();
        self.selected_room = None;
        self.timeline_items.clear();
        self.is_logging_in = false;
        self.is_oidc_logging_in = false;
        self.login_password.clear();
        self.error = None;
        self.selected_space = None;
        self.is_sync_indicator_active = false;
        Task::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Core;
    use std::collections::HashMap;

    fn create_dummy_constellations() -> Constellations {
        Constellations {
            core: Core::default(),
            matrix: None,
            sync_status: matrix::SyncStatus::Disconnected,
            room_list: Vec::new(),
            filtered_room_list: Vec::new(),
            selected_room: None,
            timeline_items: eyeball_im::Vector::new(),
            composer_text: String::new(),
            composer_preview_events: Vec::new(),
            composer_is_preview: false,
            user_id: None,
            media_cache: HashMap::new(),
            creating_room: false,
            new_room_name: String::new(),
            error: None,
            login_homeserver: String::new(),
            login_username: String::new(),
            login_password: String::new(),
            is_logging_in: false,
            is_oidc_logging_in: false,
            is_registering: false,
            is_registering_mode: false,
            is_initializing: false,
            is_sync_indicator_active: false,
            selected_space: None,
            current_settings_panel: None,
            user_settings: crate::settings::user::State::default(),
            room_settings: crate::settings::room::State::default(),
            space_settings: crate::settings::space::State::default(),
            app_settings: crate::settings::app::State::default(),
        }
    }

    #[test]
    fn test_handle_media_fetched_error() {
        let mut app = create_dummy_constellations();

        // Ensure error is initially None
        assert_eq!(app.error, None);

        // Call handle_media_fetched with an Err result
        let _task = app.handle_media_fetched(
            "mxc://example.com/media".to_string(),
            Err("network timeout".to_string()),
        );

        // Verify the error state is set correctly
        assert_eq!(
            app.error,
            Some("Failed to fetch media: network timeout".to_string())
        );

        // Ensure nothing was inserted into the cache
        assert!(app.media_cache.is_empty());
    }
}
