use crate::matrix::MatrixEngine;
use cosmic::iced::Alignment;
use cosmic::widget::{
    Column, Row, button, icon::Named, text, text_input, tooltip, tooltip::Position,
};
use cosmic::{Action, Element, Task};
use matrix_sdk::encryption::verification::{
    SasState, SasVerification, VerificationRequest, VerificationRequestState,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: Arc<str>,
    pub display_name: Option<String>,
    pub is_verified: bool,
    pub is_current: bool,
    pub is_renaming: bool,
    pub edit_name: String,
    pub is_deleting: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum VerificationUIState {
    #[default]
    None,
    WaitingForOtherDevice,
    ShowingEmojis(Vec<(String, String)>),
    Done,
    Cancelled,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub display_name: String,
    pub original_display_name: String,
    pub is_loading: bool,
    pub is_saving: bool,
    pub error: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_handle: Option<cosmic::iced::widget::image::Handle>,
    pub is_uploading_avatar: bool,
    pub is_loading_avatar: bool,
    pub current_password: String,
    pub new_password: String,
    pub confirm_new_password: String,
    pub is_changing_password: bool,
    pub password_success: Option<String>,
    pub devices: Vec<DeviceInfo>,
    pub is_loading_devices: bool,
    pub active_verification_request: Option<VerificationRequest>,
    pub active_sas: Option<SasVerification>,
    pub verification_ui_state: VerificationUIState,
    pub device_delete_password: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadProfile,
    ProfileLoaded(Result<Option<String>, String>),
    AvatarUrlLoaded(Result<Option<String>, String>),
    AvatarMediaFetched(Result<Vec<u8>, String>),
    SelectAvatar,
    AvatarFileSelected(Option<std::path::PathBuf>),
    AvatarUploaded(Result<String, String>),
    DisplayNameChanged(String),
    SaveProfile,
    ProfileSaved(Result<(), String>),
    DismissError,
    CurrentPasswordChanged(String),
    NewPasswordChanged(String),
    ConfirmNewPasswordChanged(String),
    ChangePassword,
    PasswordChanged(Result<(), String>),
    DismissPasswordSuccess,
    LoadDevices,
    DevicesLoaded(Result<Vec<DeviceInfo>, String>),
    VerifyDevice(Arc<str>),
    VerificationRequested(Result<VerificationRequest, String>),
    VerificationRequestStateChanged(VerificationRequestState),
    SasStarted(Result<Option<SasVerification>, String>),
    SasStateChanged(SasState),
    ConfirmEmojis,
    EmojisConfirmed(Result<(), String>),
    CancelVerification,
    DeviceDeletePasswordChanged(String),
    StartRenameDevice(Arc<str>),
    CancelRenameDevice(Arc<str>),
    EditDeviceNameChanged(Arc<str>, String),
    SaveDeviceName(Arc<str>),
    DeviceRenamed(Arc<str>, Result<(), String>),
    DeleteDevice(Arc<str>),
    DeviceDeleted(Arc<str>, Result<(), String>),
}

impl State {
    pub fn update(
        &mut self,
        message: Message,
        matrix: &Option<MatrixEngine>,
    ) -> Task<Action<crate::Message>> {
        match message {
            Message::LoadProfile => {
                if let Some(matrix) = matrix {
                    self.is_loading = true;
                    self.error = None;
                    self.is_loading_avatar = true;
                    let matrix_name = matrix.clone();
                    let t_name = Task::perform(
                        async move {
                            matrix_name
                                .client()
                                .await
                                .account()
                                .get_display_name()
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::ProfileLoaded(res)))
                        },
                    );

                    let matrix_avatar = matrix.clone();
                    let t_avatar = Task::perform(
                        async move {
                            matrix_avatar
                                .client()
                                .await
                                .account()
                                .get_avatar_url()
                                .await
                                .map(|u| u.map(|uri| uri.to_string()))
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::AvatarUrlLoaded(
                                res,
                            )))
                        },
                    );

                    let t_devices = Task::perform(async move {}, |_| {
                        Action::from(crate::Message::UserSettings(Message::LoadDevices))
                    });

                    return Task::batch(vec![t_name, t_avatar, t_devices]);
                }
                Task::none()
            }
            Message::ProfileLoaded(res) => {
                self.is_loading = false;
                match res {
                    Ok(name) => {
                        let name = name.unwrap_or_default();
                        self.display_name = name.clone();
                        self.original_display_name = name;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load profile name: {}", e));
                    }
                }
                Task::none()
            }
            Message::AvatarUrlLoaded(res) => {
                self.is_loading_avatar = false;
                match res {
                    Ok(Some(url)) => {
                        self.avatar_url = Some(url.clone());
                        if let Some(matrix) = matrix {
                            let matrix = matrix.clone();
                            return Task::perform(
                                async move {
                                    let uri = matrix_sdk::ruma::OwnedMxcUri::from(url.as_str());
                                    let source =
                                        matrix_sdk::ruma::events::room::MediaSource::Plain(uri);
                                    matrix.fetch_media(source).await.map_err(|e| e.to_string())
                                },
                                |res| {
                                    Action::from(crate::Message::UserSettings(
                                        Message::AvatarMediaFetched(res),
                                    ))
                                },
                            );
                        }
                    }
                    Ok(None) => {
                        self.avatar_url = None;
                        self.avatar_handle = None;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load avatar URL: {}", e));
                    }
                }
                Task::none()
            }
            Message::AvatarMediaFetched(res) => {
                match res {
                    Ok(data) => {
                        self.avatar_handle =
                            Some(cosmic::iced::widget::image::Handle::from_bytes(data));
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to fetch avatar media: {}", e));
                    }
                }
                Task::none()
            }
            Message::SelectAvatar => Task::perform(
                async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("Image", &["png", "jpg", "jpeg", "webp"])
                        .pick_file()
                        .await
                        .map(|f| f.path().to_path_buf())
                },
                |res| {
                    Action::from(crate::Message::UserSettings(Message::AvatarFileSelected(
                        res,
                    )))
                },
            ),
            Message::AvatarFileSelected(path_opt) => {
                if let (Some(path), Some(matrix)) = (path_opt, matrix) {
                    self.is_uploading_avatar = true;
                    self.error = None;
                    let matrix = matrix.clone();

                    return Task::perform(
                        async move {
                            let data = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
                            let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
                            matrix
                                .client()
                                .await
                                .account()
                                .upload_avatar(&mime_type, data)
                                .await
                                .map(|uri| uri.to_string())
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::AvatarUploaded(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::AvatarUploaded(res) => {
                self.is_uploading_avatar = false;
                match res {
                    Ok(uri) => {
                        return self.update(Message::AvatarUrlLoaded(Ok(Some(uri))), matrix);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to upload avatar: {}", e));
                    }
                }
                Task::none()
            }
            Message::DisplayNameChanged(name) => {
                self.display_name = name;
                Task::none()
            }
            Message::SaveProfile => {
                if let Some(matrix) = matrix {
                    if self.display_name != self.original_display_name {
                        self.is_saving = true;
                        self.error = None;
                        let matrix = matrix.clone();
                        let new_name = self.display_name.clone();
                        return Task::perform(
                            async move {
                                let name_opt = if new_name.is_empty() {
                                    None
                                } else {
                                    Some(new_name.as_str())
                                };
                                matrix
                                    .client()
                                    .await
                                    .account()
                                    .set_display_name(name_opt)
                                    .await
                                    .map_err(|e| e.to_string())
                            },
                            |res| {
                                Action::from(crate::Message::UserSettings(Message::ProfileSaved(
                                    res,
                                )))
                            },
                        );
                    }
                }
                Task::none()
            }
            Message::ProfileSaved(res) => {
                self.is_saving = false;
                match res {
                    Ok(_) => {
                        self.original_display_name = self.display_name.clone();
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to save profile: {}", e));
                    }
                }
                Task::none()
            }
            Message::DismissError => {
                self.error = None;
                Task::none()
            }
            Message::CurrentPasswordChanged(pw) => {
                self.current_password = pw;
                Task::none()
            }
            Message::NewPasswordChanged(pw) => {
                self.new_password = pw;
                Task::none()
            }
            Message::ConfirmNewPasswordChanged(pw) => {
                self.confirm_new_password = pw;
                Task::none()
            }
            Message::ChangePassword => {
                if let Some(matrix) = matrix {
                    if self.new_password != self.confirm_new_password {
                        self.error = Some("New passwords do not match".to_string());
                        return Task::none();
                    }
                    if self.new_password.is_empty() || self.current_password.is_empty() {
                        self.error = Some("Passwords cannot be empty".to_string());
                        return Task::none();
                    }

                    self.is_changing_password = true;
                    self.error = None;
                    self.password_success = None;

                    let matrix = matrix.clone();
                    let current_password = self.current_password.clone();
                    let new_password = self.new_password.clone();

                    return Task::perform(
                        async move {
                            let user_id = matrix
                                .client()
                                .await
                                .user_id()
                                .map(|u| u.to_string())
                                .unwrap_or_default();
                            let identifier = matrix_sdk::ruma::api::client::uiaa::UserIdentifier::UserIdOrLocalpart(user_id);
                            let password_auth = matrix_sdk::ruma::api::client::uiaa::Password::new(
                                identifier,
                                current_password,
                            );
                            let auth_data = matrix_sdk::ruma::api::client::uiaa::AuthData::Password(
                                password_auth,
                            );

                            matrix
                                .client()
                                .await
                                .account()
                                .change_password(&new_password, Some(auth_data))
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::PasswordChanged(
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::PasswordChanged(res) => {
                self.is_changing_password = false;
                match res {
                    Ok(_) => {
                        self.password_success = Some("Password changed successfully".to_string());
                        self.current_password.clear();
                        self.new_password.clear();
                        self.confirm_new_password.clear();
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to change password: {}", e));
                    }
                }
                Task::none()
            }
            Message::DismissPasswordSuccess => {
                self.password_success = None;
                Task::none()
            }
            Message::LoadDevices => {
                if let Some(matrix) = matrix {
                    self.is_loading_devices = true;
                    self.error = None;
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let user_id = client.user_id().ok_or("No user ID")?;
                            let current_device_id = client.device_id().ok_or("No device ID")?;
                            let user_devices = client
                                .encryption()
                                .get_user_devices(user_id)
                                .await
                                .map_err(|e| e.to_string())?;

                            let mut devices = Vec::new();
                            for device in user_devices.devices() {
                                devices.push(DeviceInfo {
                                    device_id: Arc::from(device.device_id().as_str()),
                                    display_name: device.display_name().map(|n| n.to_string()),
                                    is_verified: device.is_verified(),
                                    is_current: device.device_id() == current_device_id,
                                    is_renaming: false,
                                    edit_name: String::new(),
                                    is_deleting: false,
                                });
                            }
                            Ok(devices)
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::DevicesLoaded(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::DevicesLoaded(res) => {
                self.is_loading_devices = false;
                match res {
                    Ok(devices) => {
                        self.devices = devices;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load devices: {}", e));
                    }
                }
                Task::none()
            }
            Message::VerifyDevice(device_id) => {
                if let Some(matrix) = matrix {
                    self.error = None;
                    self.verification_ui_state = VerificationUIState::WaitingForOtherDevice;
                    let matrix = matrix.clone();
                    let device_id_clone = device_id.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let user_id = client.user_id().ok_or("No user ID")?;
                            let device_id_typed =
                                matrix_sdk::ruma::OwnedDeviceId::from(device_id_clone.as_ref());
                            let device = client
                                .encryption()
                                .get_device(user_id, &device_id_typed)
                                .await
                                .map_err(|e| e.to_string())?
                                .ok_or("Device not found")?;

                            let request = device
                                .request_verification()
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok(request)
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(
                                Message::VerificationRequested(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::VerificationRequested(res) => {
                match res {
                    Ok(request) => {
                        self.active_verification_request = Some(request.clone());
                        return Task::run(request.changes(), |state| {
                            Action::from(crate::Message::UserSettings(
                                Message::VerificationRequestStateChanged(state),
                            ))
                        });
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to request verification: {}", e));
                        self.verification_ui_state = VerificationUIState::None;
                    }
                }
                Task::none()
            }
            Message::VerificationRequestStateChanged(state) => {
                match state {
                    VerificationRequestState::Ready { .. } => {
                        if let Some(request) = &self.active_verification_request {
                            let req = request.clone();
                            return Task::perform(
                                async move { req.start_sas().await.map_err(|e| e.to_string()) },
                                |res| {
                                    Action::from(crate::Message::UserSettings(Message::SasStarted(
                                        res,
                                    )))
                                },
                            );
                        }
                    }
                    VerificationRequestState::Done => {
                        self.verification_ui_state = VerificationUIState::Done;
                        self.active_verification_request = None;
                        self.active_sas = None;
                        return self.update(Message::LoadDevices, matrix);
                    }
                    VerificationRequestState::Cancelled(_) => {
                        self.verification_ui_state = VerificationUIState::Cancelled;
                        self.active_verification_request = None;
                        self.active_sas = None;
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::SasStarted(res) => {
                match res {
                    Ok(Some(sas)) => {
                        self.active_sas = Some(sas.clone());
                        return Task::run(sas.changes(), |state| {
                            Action::from(crate::Message::UserSettings(Message::SasStateChanged(
                                state,
                            )))
                        });
                    }
                    Ok(None) => {
                        self.error =
                            Some("Other device does not support SAS verification.".to_string());
                        self.verification_ui_state = VerificationUIState::Cancelled;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to start SAS: {}", e));
                        self.verification_ui_state = VerificationUIState::Cancelled;
                    }
                }
                Task::none()
            }
            Message::SasStateChanged(state) => {
                match state {
                    SasState::KeysExchanged {
                        emojis: Some(emojis),
                        ..
                    } => {
                        let emoji_list = emojis
                            .emojis
                            .iter()
                            .map(|e| (e.symbol.to_string(), e.description.to_string()))
                            .collect();
                        self.verification_ui_state = VerificationUIState::ShowingEmojis(emoji_list);
                    }
                    SasState::Done { .. } => {
                        self.verification_ui_state = VerificationUIState::Done;
                        self.active_sas = None;
                        self.active_verification_request = None;
                        return self.update(Message::LoadDevices, matrix);
                    }
                    SasState::Cancelled { .. } => {
                        self.verification_ui_state = VerificationUIState::Cancelled;
                        self.active_sas = None;
                        self.active_verification_request = None;
                    }
                    _ => {}
                }
                Task::none()
            }
            Message::ConfirmEmojis => {
                if let Some(sas) = &self.active_sas {
                    let sas = sas.clone();
                    return Task::perform(
                        async move { sas.confirm().await.map_err(|e| e.to_string()) },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::EmojisConfirmed(
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::EmojisConfirmed(res) => {
                if let Err(e) = res {
                    self.error = Some(format!("Failed to confirm emojis: {}", e));
                }
                Task::none()
            }
            Message::CancelVerification => {
                let mut task = Task::none();
                if let Some(sas) = &self.active_sas {
                    let sas = sas.clone();
                    task = Task::perform(
                        async move {
                            let _ = sas.cancel().await;
                        },
                        |_| Action::from(crate::Message::NoOp),
                    );
                } else if let Some(req) = &self.active_verification_request {
                    let req = req.clone();
                    task = Task::perform(
                        async move {
                            let _ = req.cancel().await;
                        },
                        |_| Action::from(crate::Message::NoOp),
                    );
                }
                self.verification_ui_state = VerificationUIState::Cancelled;
                self.active_sas = None;
                self.active_verification_request = None;
                task
            }
            Message::DeviceDeletePasswordChanged(pw) => {
                self.device_delete_password = pw;
                Task::none()
            }
            Message::StartRenameDevice(ref device_id) => {
                if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == *device_id) {
                    device.is_renaming = true;
                    device.edit_name = device.display_name.clone().unwrap_or_default();
                }
                Task::none()
            }
            Message::CancelRenameDevice(ref device_id) => {
                if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == *device_id) {
                    device.is_renaming = false;
                }
                Task::none()
            }
            Message::EditDeviceNameChanged(ref device_id, new_name) => {
                if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == *device_id) {
                    device.edit_name = new_name;
                }
                Task::none()
            }
            Message::SaveDeviceName(ref device_id) => {
                if let Some(matrix) = matrix {
                    if let Some(device) =
                        self.devices.iter_mut().find(|d| d.device_id == *device_id)
                    {
                        device.is_renaming = false;
                        let new_name = device.edit_name.clone();
                        let device_id_str = device_id.clone();
                        let device_id_for_closure = device_id_str.clone();
                        let matrix = matrix.clone();
                        return Task::perform(
                            async move {
                                let did =
                                    matrix_sdk::ruma::OwnedDeviceId::from(device_id_str.as_ref());
                                matrix
                                    .client()
                                    .await
                                    .rename_device(&did, &new_name)
                                    .await
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            },
                            move |res| {
                                Action::from(crate::Message::UserSettings(Message::DeviceRenamed(
                                    device_id_for_closure,
                                    res,
                                )))
                            },
                        );
                    }
                }
                Task::none()
            }
            Message::DeviceRenamed(ref device_id, res) => {
                match res {
                    Ok(_) => {
                        if let Some(device) =
                            self.devices.iter_mut().find(|d| d.device_id == *device_id)
                        {
                            device.display_name = Some(device.edit_name.clone());
                        }
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to rename device: {}", e));
                    }
                }
                Task::none()
            }
            Message::DeleteDevice(ref device_id) => {
                if let Some(matrix) = matrix {
                    if let Some(device) =
                        self.devices.iter_mut().find(|d| d.device_id == *device_id)
                    {
                        device.is_deleting = true;
                        let matrix = matrix.clone();
                        let device_id_str = device_id.clone();
                        let device_id_for_closure = device_id_str.clone();
                        let password = self.device_delete_password.clone();
                        return Task::perform(
                            async move {
                                let client = matrix.client().await;
                                let user_id = client.user_id().ok_or("No user ID")?.to_string();
                                let did =
                                    matrix_sdk::ruma::OwnedDeviceId::from(device_id_str.as_ref());

                                if let Err(e) = client
                                    .delete_devices(std::slice::from_ref(&did), None)
                                    .await
                                {
                                    if let Some(info) = e.as_uiaa_response() {
                                        if password.is_empty() {
                                            return Err(
                                                "Password required to delete device".to_string()
                                            );
                                        }

                                        let identifier = matrix_sdk::ruma::api::client::uiaa::UserIdentifier::UserIdOrLocalpart(user_id);
                                        let mut password_auth =
                                            matrix_sdk::ruma::api::client::uiaa::Password::new(
                                                identifier, password,
                                            );
                                        password_auth.session = info.session.clone();

                                        client.delete_devices(&[did], Some(matrix_sdk::ruma::api::client::uiaa::AuthData::Password(password_auth))).await.map(|_| ()).map_err(|e| e.to_string())?;
                                        return Ok(());
                                    }
                                    return Err(e.to_string());
                                }
                                Ok(())
                            },
                            move |res| {
                                Action::from(crate::Message::UserSettings(Message::DeviceDeleted(
                                    device_id_for_closure,
                                    res,
                                )))
                            },
                        );
                    }
                }
                Task::none()
            }
            Message::DeviceDeleted(ref device_id, res) => {
                if let Some(device) = self.devices.iter_mut().find(|d| d.device_id == *device_id) {
                    device.is_deleting = false;
                }
                match res {
                    Ok(_) => {
                        self.devices.retain(|d| d.device_id != *device_id);
                        self.device_delete_password.clear();
                    }
                    Err(e) => {
                        self.error = Some(format!(
                            "Failed to delete device: {}. You might need to provide a password below.",
                            e
                        ));
                    }
                }
                Task::none()
            }
        }
    }

    fn view_profile<'a>(&'a self) -> Element<'a, Message> {
        let mut col = Column::new().spacing(20);

        if self.is_loading || self.is_loading_avatar {
            col = col.push(text::body("Loading profile..."));
        } else {
            let mut avatar_col = Column::new().spacing(10).align_x(Alignment::Center);

            if let Some(handle) = &self.avatar_handle {
                avatar_col =
                    avatar_col.push(cosmic::widget::image(handle.clone()).width(128).height(128));
            } else {
                avatar_col = avatar_col.push(
                    cosmic::widget::container(text::body("No Avatar").size(16))
                        .width(128)
                        .height(128)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center),
                );
            }

            let mut avatar_btn = button::text(if self.is_uploading_avatar {
                "Uploading..."
            } else {
                "Change Avatar"
            });

            if !self.is_uploading_avatar {
                avatar_btn = avatar_btn.on_press(Message::SelectAvatar);
            }

            avatar_col = avatar_col.push(avatar_btn);

            col = col.push(avatar_col);

            col = col.push(
                Column::new()
                    .spacing(5)
                    .push(text::body("Display Name").size(16))
                    .push(
                        text_input("Enter your display name", &self.display_name)
                            .on_input(Message::DisplayNameChanged),
                    ),
            );

            let mut save_row = Row::new().spacing(10);

            if self.is_saving {
                save_row = save_row.push(button::text("Saving..."));
            } else {
                let mut btn = button::text("Save");
                if self.display_name != self.original_display_name {
                    btn = btn.on_press(Message::SaveProfile);
                }
                save_row = save_row.push(btn);
            }

            col = col.push(save_row);
        }
        col.into()
    }

    fn view_password_change<'a>(&'a self) -> Element<'a, Message> {
        let mut col = Column::new().spacing(20);

        col = col.push(
            Column::new()
                .spacing(5)
                .push(text::title3("Change Password"))
                .push(
                    text_input("Current password", &self.current_password)
                        .password()
                        .on_input(Message::CurrentPasswordChanged),
                )
                .push(
                    text_input("New password", &self.new_password)
                        .password()
                        .on_input(Message::NewPasswordChanged),
                )
                .push(
                    text_input("Confirm new password", &self.confirm_new_password)
                        .password()
                        .on_input(Message::ConfirmNewPasswordChanged),
                ),
        );

        let mut pw_btn = button::text(if self.is_changing_password {
            "Changing..."
        } else {
            "Change Password"
        });
        if !self.is_changing_password
            && !self.current_password.is_empty()
            && !self.new_password.is_empty()
            && self.new_password == self.confirm_new_password
        {
            pw_btn = pw_btn.on_press(Message::ChangePassword);
        }
        col = col.push(pw_btn);

        if let Some(success) = &self.password_success {
            col = col.push(
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(success))
                    .push(button::text("Dismiss").on_press(Message::DismissPasswordSuccess)),
            );
        }

        col.into()
    }

    fn view_devices<'a>(&'a self) -> Element<'a, Message> {
        let mut devices_col = Column::new()
            .spacing(10)
            .push(text::title3("Devices & Sessions"));

        if self.is_loading_devices {
            devices_col = devices_col.push(text::body("Loading devices..."));
        } else {
            for device in &self.devices {
                let name = device
                    .display_name
                    .clone()
                    .unwrap_or_else(|| "Unknown Device".to_string());
                let mut row = Row::new().spacing(10).align_y(Alignment::Center);

                if device.is_renaming {
                    row = row
                        .push(
                            text_input("New device name", &device.edit_name)
                                .on_input({
                                    let id = Arc::clone(&device.device_id);
                                    move |v| Message::EditDeviceNameChanged(id.clone(), v)
                                })
                                .on_submit(|_| Message::SaveDeviceName(device.device_id.clone())),
                        )
                        .push(
                            button::text("Save")
                                .on_press(Message::SaveDeviceName(device.device_id.clone())),
                        )
                        .push(
                            button::text("Cancel")
                                .on_press(Message::CancelRenameDevice(device.device_id.clone())),
                        );
                } else {
                    row = row
                        .push(text::body(name).size(14))
                        .push(text::body(format!("({})", device.device_id.as_ref())).size(12))
                        .push(tooltip(
                            button::icon(Named::new("document-edit-symbolic"))
                                .on_press(Message::StartRenameDevice(device.device_id.clone())),
                            text::body("Rename Device"),
                            Position::Top,
                        ));
                }

                if device.is_current {
                    row = row
                        .push(cosmic::widget::container(text::body("Current").size(12)).padding(2));
                }

                row = row.push(cosmic::widget::space().width(cosmic::iced::Length::Fill));

                if device.is_verified {
                    row = row.push(text::body("✅ Verified").size(14));
                } else {
                    row = row.push(text::body("❌ Unverified").size(14));
                    if !device.is_current {
                        row = row.push(
                            button::text("Verify")
                                .on_press(Message::VerifyDevice(device.device_id.clone())),
                        );
                    }
                }

                let mut del_btn = button::destructive(if device.is_deleting {
                    "Deleting..."
                } else {
                    "Delete"
                });
                if !device.is_deleting {
                    del_btn = del_btn.on_press(Message::DeleteDevice(device.device_id.clone()));
                }
                row = row.push(tooltip(del_btn, text::body("Delete Device"), Position::Top));

                devices_col = devices_col.push(row);
            }

            devices_col = devices_col.push(
                Column::new()
                    .spacing(5)
                    .push(text::body("Password to delete devices:").size(12))
                    .push(
                        text_input("Password", &self.device_delete_password)
                            .password()
                            .on_input(Message::DeviceDeletePasswordChanged),
                    ),
            );
        }

        devices_col.into()
    }

    fn view_verification<'a>(&'a self) -> Element<'a, Message> {
        let mut col = Column::new().spacing(10);
        match &self.verification_ui_state {
            VerificationUIState::WaitingForOtherDevice => {
                col = col.push(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(text::body(
                            "Verification requested. Please accept it on the other device.",
                        ))
                        .push(
                            button::text("Cancel Verification")
                                .on_press(Message::CancelVerification),
                        ),
                );
            }
            VerificationUIState::ShowingEmojis(emojis) => {
                let mut emoji_row = Row::new().spacing(10).align_y(Alignment::Center);
                for (symbol, desc) in emojis {
                    emoji_row = emoji_row.push(
                        Column::new()
                            .spacing(2)
                            .align_x(Alignment::Center)
                            .push(text::title1(symbol))
                            .push(text::body(desc).size(12)),
                    );
                }

                col = col.push(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(text::body("Do these emojis match the other device?"))
                        .push(emoji_row)
                        .push(
                            Row::new()
                                .spacing(10)
                                .push(button::text("Match!").on_press(Message::ConfirmEmojis))
                                .push(button::text("Cancel").on_press(Message::CancelVerification)),
                        ),
                );
            }
            VerificationUIState::Done => {
                col = col.push(text::body("Verification successful!"));
            }
            VerificationUIState::Cancelled => {
                col = col.push(text::body("Verification cancelled."));
            }
            VerificationUIState::None => {}
        }
        col.into()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(20);

        col = col.push(self.view_profile());

        if let Some(err) = &self.error {
            col = col.push(
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(err))
                    .push(button::text("Dismiss").on_press(Message::DismissError)),
            );
        }

        col = col.push(self.view_password_change());

        col = col.push(self.view_devices());

        col = col.push(self.view_verification());

        col.into()
    }
}
