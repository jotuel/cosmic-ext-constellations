use cosmic::widget::{button, text, text_input, Column, Row};
use cosmic::iced::Alignment;
use cosmic::{Element, Task, Action};
use crate::matrix::{MatrixEngine, RoomData};
use matrix_sdk::ruma::RoomId;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub space_id: Option<String>,
    pub name: String,
    pub original_name: String,
    pub is_loading: bool,
    pub is_saving: bool,
    pub error: Option<String>,
    pub children: Vec<RoomData>,
    pub is_loading_children: bool,
    pub new_child_id: String,
    pub is_adding_child: bool,
    pub topic: String,
    pub original_topic: String,
    pub avatar_url: Option<String>,
    pub avatar_handle: Option<cosmic::iced::widget::image::Handle>,
    pub is_uploading_avatar: bool,
    pub is_loading_avatar: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadSpace(String),
    SpaceLoaded(Result<SpaceInfo, String>),
    NameChanged(String),
    TopicChanged(String),
    SaveSpace,
    SpaceSaved(Result<(), String>),
    DismissError,
    LoadChildren,
    ChildrenLoaded(Result<Vec<RoomData>, String>),
    AddChild,
    ChildAdded(Result<(), String>),
    RemoveChild(String),
    ChildRemoved(String, Result<(), String>),
    NewChildIdChanged(String),
    AvatarMediaFetched(Result<Vec<u8>, String>),
    SelectAvatar,
    AvatarFileSelected(Option<std::path::PathBuf>),
    AvatarUploaded(Result<(), String>),
}

#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub name: String,
    pub topic: String,
    pub avatar_url: Option<String>,
}

impl State {
    pub fn update(
        &mut self,
        message: Message,
        matrix: &Option<MatrixEngine>,
    ) -> Task<Action<crate::Message>> {
        match message {
            Message::LoadSpace(space_id) => {
                if let Some(matrix) = matrix {
                    self.space_id = Some(space_id.clone());
                    self.is_loading = true;
                    self.error = None;

                    let engine = matrix.clone();
                    Task::perform(
                        async move {
                            let room_id_parsed = RoomId::parse(&space_id).map_err(|e| e.to_string())?;
                            let client = engine.client().await;
                            let room = client.get_room(&room_id_parsed)
                                .ok_or_else(|| "Space not found".to_string())?;

                            Ok(SpaceInfo {
                                name: room.name().unwrap_or_default(),
                                topic: room.topic().unwrap_or_default(),
                                avatar_url: room.avatar_url().map(|u| u.to_string()),
                            })
                        },
                        |res| Action::from(crate::Message::SpaceSettings(Message::SpaceLoaded(res)))
                    )
                } else {
                    Task::none()
                }
            }
            Message::SpaceLoaded(res) => {
                self.is_loading = false;
                match res {
                    Ok(info) => {
                        self.name = info.name.clone();
                        self.original_name = info.name;
                        self.topic = info.topic.clone();
                        self.original_topic = info.topic;
                        self.avatar_url = info.avatar_url;
                        self.error = None;
                        
                        let mut tasks = Vec::new();

                        if let Some(url) = &self.avatar_url {
                            if let Some(matrix) = matrix {
                                let engine = matrix.clone();
                                let mxc = url.clone();
                                self.is_loading_avatar = true;
                                tasks.push(Task::perform(
                                    async move {
                                        use matrix_sdk::ruma::events::room::MediaSource;
                                        let mxc_uri = <&matrix_sdk::ruma::MxcUri>::try_from(mxc.as_str()).map_err(|e| e.to_string())?;
                                        let source = MediaSource::Plain(mxc_uri.to_owned());
                                        engine.fetch_media(source).await.map_err(|e| e.to_string())
                                    },
                                    |res| Action::from(crate::Message::SpaceSettings(Message::AvatarMediaFetched(res)))
                                ));
                            }
                        }

                        tasks.push(Task::done(Action::from(crate::Message::SpaceSettings(Message::LoadChildren))));
                        return Task::batch(tasks);
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            Message::AvatarMediaFetched(res) => {
                self.is_loading_avatar = false;
                match res {
                    Ok(data) => {
                        self.avatar_handle = Some(cosmic::iced::widget::image::Handle::from_bytes(data));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to fetch avatar: {}", e));
                    }
                }
                Task::none()
            }
            Message::SelectAvatar => {
                Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
                            .set_title("Select Space Avatar")
                            .pick_file()
                            .await
                            .map(|handle| handle.path().to_owned())
                    },
                    |res| Action::from(crate::Message::SpaceSettings(Message::AvatarFileSelected(res)))
                )
            }
            Message::AvatarFileSelected(path_opt) => {
                if let Some(path) = path_opt {
                    if let Some(matrix) = matrix {
                        self.is_uploading_avatar = true;
                        let engine = matrix.clone();
                        let room_id = self.space_id.clone().unwrap_or_default();
                        
                        return Task::perform(
                            async move {
                                let data = std::fs::read(&path).map_err(|e| e.to_string())?;
                                let mime = mime_guess::from_path(&path).first_raw().unwrap_or("image/jpeg");
                                engine.upload_room_avatar(&room_id, data, mime).await.map_err(|e| e.to_string())
                            },
                            |res| Action::from(crate::Message::SpaceSettings(Message::AvatarUploaded(res)))
                        );
                    }
                }
                Task::none()
            }
            Message::AvatarUploaded(res) => {
                self.is_uploading_avatar = false;
                match res {
                    Ok(_) => {
                        if let Some(space_id) = &self.space_id {
                            return self.update(Message::LoadSpace(space_id.clone()), matrix);
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to upload avatar: {}", e));
                    }
                }
                Task::none()
            }
            Message::LoadChildren => {
                if let Some(matrix) = matrix {
                    if let Some(space_id) = &self.space_id {
                        self.is_loading_children = true;
                        let engine = matrix.clone();
                        let space_id_clone = space_id.clone();
                        return Task::perform(
                            async move {
                                engine.get_space_children(&space_id_clone).await.map_err(|e| e.to_string())
                            },
                            |res| Action::from(crate::Message::SpaceSettings(Message::ChildrenLoaded(res)))
                        );
                    }
                }
                Task::none()
            }
            Message::ChildrenLoaded(res) => {
                self.is_loading_children = false;
                match res {
                    Ok(children) => {
                        self.children = children;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load space children: {}", e));
                    }
                }
                Task::none()
            }
            Message::NameChanged(name) => {
                self.name = name;
                Task::none()
            }
            Message::TopicChanged(topic) => {
                self.topic = topic;
                Task::none()
            }
            Message::SaveSpace => {
                if let Some(matrix) = matrix {
                    if let Some(space_id) = &self.space_id {
                        self.is_saving = true;
                        self.error = None;

                        let engine = matrix.clone();
                        let new_name = self.name.clone();
                        let new_topic = self.topic.clone();
                        let space_id_clone = space_id.clone();
                        let original_name = self.original_name.clone();
                        let original_topic = self.original_topic.clone();

                        Task::perform(
                            async move {
                                if new_name != original_name {
                                    engine.set_room_name(&space_id_clone, new_name).await.map_err(|e| e.to_string())?;
                                }
                                if new_topic != original_topic {
                                    engine.set_room_topic(&space_id_clone, new_topic).await.map_err(|e| e.to_string())?;
                                }
                                Ok(())
                            },
                            |res| Action::from(crate::Message::SpaceSettings(Message::SpaceSaved(res)))
                        )
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
            Message::SpaceSaved(res) => {
                self.is_saving = false;
                match res {
                    Ok(_) => {
                        self.original_name = self.name.clone();
                        self.original_topic = self.topic.clone();
                        self.error = None;
                    }
                    Err(e) => {
                        self.error = Some(e);
                    }
                }
                Task::none()
            }
            Message::AddChild => {
                if let Some(matrix) = matrix {
                    if let Some(space_id) = &self.space_id {
                        self.is_adding_child = true;
                        let engine = matrix.clone();
                        let space_id_clone = space_id.clone();
                        let child_id_clone = self.new_child_id.clone();
                        return Task::perform(
                            async move {
                                engine.add_space_child(&space_id_clone, &child_id_clone).await.map_err(|e| e.to_string())
                            },
                            |res| Action::from(crate::Message::SpaceSettings(Message::ChildAdded(res)))
                        );
                    }
                }
                Task::none()
            }
            Message::ChildAdded(res) => {
                self.is_adding_child = false;
                match res {
                    Ok(_) => {
                        self.new_child_id = String::new();
                        return Task::done(Action::from(crate::Message::SpaceSettings(Message::LoadChildren)));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to add child: {}", e));
                    }
                }
                Task::none()
            }
            Message::RemoveChild(child_id) => {
                if let Some(matrix) = matrix {
                    if let Some(space_id) = &self.space_id {
                        let engine = matrix.clone();
                        let space_id_clone = space_id.clone();
                        let child_id_clone = child_id.clone();
                        let child_id_for_task = child_id.clone();
                        return Task::perform(
                            async move {
                                engine.remove_space_child(&space_id_clone, &child_id_for_task).await.map_err(|e| e.to_string())
                            },
                            move |res| Action::from(crate::Message::SpaceSettings(Message::ChildRemoved(child_id_clone, res)))
                        );
                    }
                }
                Task::none()
            }
            Message::ChildRemoved(_child_id, res) => {
                match res {
                    Ok(_) => {
                        return Task::done(Action::from(crate::Message::SpaceSettings(Message::LoadChildren)));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to remove child: {}", e));
                    }
                }
                Task::none()
            }
            Message::NewChildIdChanged(id) => {
                self.new_child_id = id;
                Task::none()
            }
            Message::DismissError => {
                self.error = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.is_loading {
            return Column::new()
                .spacing(20)
                .push(text::body("Loading space data..."))
                .into();
        }

        let mut col = Column::new().spacing(20);

        if let Some(error) = &self.error {
            col = col.push(
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(error))
                    .push(button::text("Dismiss").on_press(Message::DismissError))
            );
        }

        col = col.push(text::title3("Space Profile"));

        // Avatar Section
        let mut avatar_row = Row::new().spacing(20).align_y(Alignment::Center);
        if let Some(handle) = &self.avatar_handle {
             avatar_row = avatar_row.push(
                cosmic::widget::image(handle.clone())
                    .width(cosmic::iced::Length::Fixed(64.0))
                    .height(cosmic::iced::Length::Fixed(64.0))
            );
        } else if self.is_loading_avatar {
             avatar_row = avatar_row.push(text::body("Loading avatar..."));
        } else {
             avatar_row = avatar_row.push(
                cosmic::widget::container(text::body("No Avatar"))
                    .width(cosmic::iced::Length::Fixed(64.0))
                    .height(cosmic::iced::Length::Fixed(64.0))
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
            );
        }

        let mut upload_btn = button::text(if self.is_uploading_avatar { "Uploading..." } else { "Change Avatar" });
        if !self.is_uploading_avatar {
            upload_btn = upload_btn.on_press(Message::SelectAvatar);
        }
        avatar_row = avatar_row.push(upload_btn);
        col = col.push(avatar_row);
        
        col = col.push(
            Column::new().spacing(5)
                .push(text::body("Space Name").size(12))
                .push(text_input::text_input("Name", &self.name)
                    .on_input(Message::NameChanged))
        );

        col = col.push(
            Column::new().spacing(5)
                .push(text::body("Space Topic").size(12))
                .push(text_input::text_input("Topic", &self.topic)
                    .on_input(Message::TopicChanged))
        );

        let mut save_btn = button::text(if self.is_saving { "Saving..." } else { "Save Changes" });
        if (self.name != self.original_name || self.topic != self.original_topic) && !self.is_saving {
            save_btn = save_btn.on_press(Message::SaveSpace);
        }
        col = col.push(save_btn);

        col = col.push(text::title3("Space Hierarchy"));

        // Manage Children
        let mut children_col = Column::new().spacing(10);
        if self.is_loading_children {
            children_col = children_col.push(text::body("Loading children..."));
        } else {
            for child in &self.children {
                let name = child.name.clone().unwrap_or_else(|| child.id.clone());
                children_col = children_col.push(
                    Row::new()
                        .spacing(10)
                        .align_y(Alignment::Center)
                        .push(text::body(name))
                        .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                        .push(button::text("Remove").on_press(Message::RemoveChild(child.id.clone())))
                );
            }
        }
        col = col.push(children_col);

        // Add Child
        col = col.push(text::body("Add room or subspace by ID").size(12));
        col = col.push(
            Row::new()
                .spacing(10)
                .push(text_input::text_input("!room_id:server.com", &self.new_child_id)
                    .on_input(Message::NewChildIdChanged))
                .push(button::text("Add Child").on_press(Message::AddChild))
        );

        col.into()
    }
}
