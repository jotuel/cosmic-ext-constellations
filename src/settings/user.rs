use crate::matrix::MatrixEngine;
use cosmic::iced::Alignment;
use cosmic::widget::{
    Column, Row, button, icon::Named, text, text_input, tooltip, tooltip::Position,
};
use cosmic::{Action, Element, Task};
use matrix_sdk::encryption::CrossSigningStatus;
use matrix_sdk::encryption::verification::{
    SasState, SasVerification, VerificationRequest, VerificationRequestState,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CrossSigningInfo {
    pub status: CrossSigningStatus,
    pub master_key: Option<String>,
    pub self_signing_key: Option<String>,
    pub user_signing_key: Option<String>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Threepid {
    pub address: String,
    pub medium: matrix_sdk::ruma::thirdparty::Medium,
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

#[derive(Debug, Clone)]
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
    pub success_message: Option<String>,
    pub devices: Vec<DeviceInfo>,
    pub is_loading_devices: bool,
    pub active_verification_request: Option<VerificationRequest>,
    pub active_sas: Option<SasVerification>,
    pub verification_ui_state: VerificationUIState,
    pub device_delete_password: String,
    pub global_notification_mode_dm:
        Option<matrix_sdk::notification_settings::RoomNotificationMode>,
    pub global_notification_mode_group:
        Option<matrix_sdk::notification_settings::RoomNotificationMode>,
    pub is_loading_global_notifications: bool,
    pub cross_signing_info: Option<CrossSigningInfo>,
    pub is_loading_cross_signing: bool,
    pub is_bootstrapping: bool,
    pub media_previews_display_policy: bool,
    pub invite_avatars_display_policy: bool,
    pub threepids: Vec<Threepid>,
    pub is_loading_3pids: bool,
    pub new_3pid_email: String,
    pub new_3pid_msisdn: String,
    pub new_3pid_country_code: String,
    pub is_requesting_3pid_token: bool,
    pub adding_3pid_sid: Option<String>,
    pub adding_3pid_client_secret: Option<String>,
    pub add_3pid_password: String,
    pub keywords: Vec<String>,
    pub new_keyword: String,
    pub is_loading_keywords: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            original_display_name: String::new(),
            is_loading: false,
            is_saving: false,
            error: None,
            avatar_url: None,
            avatar_handle: None,
            is_uploading_avatar: false,
            is_loading_avatar: false,
            current_password: String::new(),
            new_password: String::new(),
            confirm_new_password: String::new(),
            is_changing_password: false,
            password_success: None,
            success_message: None,
            devices: Vec::new(),
            is_loading_devices: false,
            active_verification_request: None,
            active_sas: None,
            verification_ui_state: VerificationUIState::default(),
            device_delete_password: String::new(),
            global_notification_mode_dm: None,
            global_notification_mode_group: None,
            is_loading_global_notifications: false,
            cross_signing_info: None,
            is_loading_cross_signing: false,
            is_bootstrapping: false,
            media_previews_display_policy: true,
            invite_avatars_display_policy: true,
            threepids: Vec::new(),
            is_loading_3pids: false,
            new_3pid_email: String::new(),
            new_3pid_msisdn: String::new(),
            new_3pid_country_code: String::new(),
            is_requesting_3pid_token: false,
            adding_3pid_sid: None,
            adding_3pid_client_secret: None,
            add_3pid_password: String::new(),
            keywords: Vec::new(),
            new_keyword: String::new(),
            is_loading_keywords: false,
        }
    }
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
    GlobalNotificationModeChanged(
        bool,
        matrix_sdk::notification_settings::RoomNotificationMode,
    ),
    GlobalNotificationModeLoaded(
        bool,
        matrix_sdk::notification_settings::RoomNotificationMode,
    ),
    GlobalNotificationModeSet(Result<(), String>),
    LoadCrossSigningStatus,
    CrossSigningStatusLoaded(Result<Option<CrossSigningInfo>, String>),
    BootstrapCrossSigning,
    CrossSigningBootstrapped(Result<(), String>),
    ToggleMediaPreviewsDisplayPolicy(bool),
    ToggleInviteAvatarsDisplayPolicy(bool),
    Load3PIDs,
    ThreepidsLoaded(Result<Vec<Threepid>, String>),
    New3PIDEmailChanged(String),
    New3PIDMsisdnChanged(String),
    New3PIDCountryCodeChanged(String),
    Request3PIDEmailToken,
    Request3PIDMsisdnToken,
    ThreepidTokenRequested(Result<String, String>),
    Add3PIDPasswordChanged(String),
    Add3PID,
    ThreepidAdded(Result<(), String>),
    Delete3PID(String, matrix_sdk::ruma::thirdparty::Medium),
    ThreepidDeleted(String, Result<(), String>),
    DismissSuccessMessage,
    LoadKeywords,
    KeywordsLoaded(Vec<String>),
    NewKeywordChanged(String),
    AddKeyword,
    KeywordAdded(Result<(), String>),
    RemoveKeyword(String),
    KeywordRemoved(Result<(), String>),
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

                    let t_3pids = Task::perform(async move {}, |_| {
                        Action::from(crate::Message::UserSettings(Message::Load3PIDs))
                    });

                    let matrix_dm = matrix.clone();
                    let t_dm = Task::perform(
                        async move {
                            let client = matrix_dm.client().await;
                            let ns = client.notification_settings().await;
                            ns.get_default_room_notification_mode(
                                matrix_sdk::notification_settings::IsEncrypted::Yes,
                                matrix_sdk::notification_settings::IsOneToOne::Yes,
                            )
                            .await
                        },
                        |mode| {
                            Action::from(crate::Message::UserSettings(
                                Message::GlobalNotificationModeLoaded(true, mode),
                            ))
                        },
                    );

                    let matrix_group = matrix.clone();
                    let t_group = Task::perform(
                        async move {
                            let client = matrix_group.client().await;
                            let ns = client.notification_settings().await;
                            ns.get_default_room_notification_mode(
                                matrix_sdk::notification_settings::IsEncrypted::Yes,
                                matrix_sdk::notification_settings::IsOneToOne::No,
                            )
                            .await
                        },
                        |mode| {
                            Action::from(crate::Message::UserSettings(
                                Message::GlobalNotificationModeLoaded(false, mode),
                            ))
                        },
                    );

                    let t_cross_signing = Task::perform(async move {}, |_| {
                        Action::from(crate::Message::UserSettings(
                            Message::LoadCrossSigningStatus,
                        ))
                    });

                    let t_keywords = Task::done(Action::from(crate::Message::UserSettings(
                        Message::LoadKeywords,
                    )));

                    return Task::batch(vec![
                        t_name,
                        t_avatar,
                        t_devices,
                        t_3pids,
                        t_dm,
                        t_group,
                        t_cross_signing,
                        t_keywords,
                    ]);
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
            Message::GlobalNotificationModeLoaded(is_dm, mode) => {
                if is_dm {
                    self.global_notification_mode_dm = Some(mode);
                } else {
                    self.global_notification_mode_group = Some(mode);
                }
                Task::none()
            }
            Message::GlobalNotificationModeChanged(is_dm, mode) => {
                if is_dm {
                    self.global_notification_mode_dm = Some(mode);
                } else {
                    self.global_notification_mode_group = Some(mode);
                }
                if let Some(matrix) = matrix {
                    let matrix = matrix.clone();
                    self.is_loading_global_notifications = true;
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let ns = client.notification_settings().await;

                            // Set for both encrypted and unencrypted
                            ns.set_default_room_notification_mode(
                                matrix_sdk::notification_settings::IsEncrypted::Yes,
                                if is_dm {
                                    matrix_sdk::notification_settings::IsOneToOne::Yes
                                } else {
                                    matrix_sdk::notification_settings::IsOneToOne::No
                                },
                                mode,
                            )
                            .await
                            .map_err(|e| e.to_string())?;

                            ns.set_default_room_notification_mode(
                                matrix_sdk::notification_settings::IsEncrypted::No,
                                if is_dm {
                                    matrix_sdk::notification_settings::IsOneToOne::Yes
                                } else {
                                    matrix_sdk::notification_settings::IsOneToOne::No
                                },
                                mode,
                            )
                            .await
                            .map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(
                                Message::GlobalNotificationModeSet(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::GlobalNotificationModeSet(res) => {
                self.is_loading_global_notifications = false;
                if let Err(e) = res {
                    self.error = Some(format!("Failed to set notification mode: {}", e));
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
            Message::LoadCrossSigningStatus => {
                if let Some(matrix) = matrix {
                    self.is_loading_cross_signing = true;
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let encryption = client.encryption();
                            let status_opt = encryption.cross_signing_status().await;

                            if let Some(status) = status_opt {
                                return Ok(Some(CrossSigningInfo {
                                    status,
                                    master_key: None,
                                    self_signing_key: None,
                                    user_signing_key: None,
                                }));
                            }
                            Ok(None)
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(
                                Message::CrossSigningStatusLoaded(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::CrossSigningStatusLoaded(res) => {
                self.is_loading_cross_signing = false;
                match res {
                    Ok(info) => {
                        self.cross_signing_info = info;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load cross-signing status: {}", e));
                    }
                }
                Task::none()
            }
            Message::BootstrapCrossSigning => {
                if let Some(matrix) = matrix {
                    self.is_bootstrapping = true;
                    let matrix = matrix.clone();
                    let password = self.device_delete_password.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let user_id = client.user_id().ok_or("No user ID")?.to_string();

                            if let Err(e) = client.encryption().bootstrap_cross_signing(None).await
                            {
                                if let Some(info) = e.as_uiaa_response() {
                                    if password.is_empty() {
                                        return Err(
                                            "Password required for bootstrapping (use the delete-device password field)".to_string()
                                        );
                                    }

                                    let identifier = matrix_sdk::ruma::api::client::uiaa::UserIdentifier::UserIdOrLocalpart(user_id);
                                    let mut password_auth =
                                        matrix_sdk::ruma::api::client::uiaa::Password::new(
                                            identifier, password,
                                        );
                                    password_auth.session = info.session.clone();

                                    client
                                        .encryption()
                                        .bootstrap_cross_signing(Some(
                                            matrix_sdk::ruma::api::client::uiaa::AuthData::Password(
                                                password_auth,
                                            ),
                                        ))
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    return Ok(());
                                }
                                return Err(e.to_string());
                            }
                            Ok(())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(
                                Message::CrossSigningBootstrapped(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::CrossSigningBootstrapped(res) => {
                self.is_bootstrapping = false;
                match res {
                    Ok(_) => {
                        return self.update(Message::LoadCrossSigningStatus, matrix);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to bootstrap cross-signing: {}", e));
                    }
                }
                Task::none()
            }
            Message::ToggleMediaPreviewsDisplayPolicy(enabled) => {
                self.media_previews_display_policy = enabled;
                if let Some(matrix) = matrix {
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            matrix
                                .set_media_previews_display_policy(enabled)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |_| Action::from(crate::Message::AppSettingChanged),
                    );
                }
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleInviteAvatarsDisplayPolicy(enabled) => {
                self.invite_avatars_display_policy = enabled;
                if let Some(matrix) = matrix {
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            matrix
                                .set_invite_avatars_display_policy(enabled)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        |_| Action::from(crate::Message::AppSettingChanged),
                    );
                }
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::Load3PIDs => {
                if let Some(matrix) = matrix {
                    self.is_loading_3pids = true;
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let resp = client
                                .account()
                                .get_3pids()
                                .await
                                .map_err(|e| e.to_string())?;
                            let threepids = resp
                                .threepids
                                .into_iter()
                                .map(|t| Threepid {
                                    address: t.address,
                                    medium: t.medium,
                                })
                                .collect();
                            Ok(threepids)
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::ThreepidsLoaded(
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::ThreepidsLoaded(res) => {
                self.is_loading_3pids = false;
                match res {
                    Ok(threepids) => {
                        self.threepids = threepids;
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to load 3PIDs: {}", e));
                    }
                }
                Task::none()
            }
            Message::New3PIDEmailChanged(email) => {
                self.new_3pid_email = email;
                Task::none()
            }
            Message::New3PIDMsisdnChanged(msisdn) => {
                self.new_3pid_msisdn = msisdn;
                Task::none()
            }
            Message::New3PIDCountryCodeChanged(cc) => {
                self.new_3pid_country_code = cc;
                Task::none()
            }
            Message::Add3PIDPasswordChanged(pw) => {
                self.add_3pid_password = pw;
                Task::none()
            }
            Message::Request3PIDEmailToken => {
                if let Some(matrix) = matrix {
                    self.is_requesting_3pid_token = true;
                    self.error = None;
                    let matrix = matrix.clone();
                    let email = self.new_3pid_email.clone();
                    let client_secret = matrix_sdk::ruma::ClientSecret::new();
                    self.adding_3pid_client_secret = Some(client_secret.to_string());

                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let resp = client
                                .account()
                                .request_3pid_email_token(
                                    &client_secret,
                                    &email,
                                    matrix_sdk::ruma::uint!(1),
                                )
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok(resp.sid.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(
                                Message::ThreepidTokenRequested(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::Request3PIDMsisdnToken => {
                if let Some(matrix) = matrix {
                    self.is_requesting_3pid_token = true;
                    self.error = None;
                    let matrix = matrix.clone();
                    let msisdn = self.new_3pid_msisdn.clone();
                    let country = self.new_3pid_country_code.clone();
                    let client_secret = matrix_sdk::ruma::ClientSecret::new();
                    self.adding_3pid_client_secret = Some(client_secret.to_string());

                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let resp = client
                                .account()
                                .request_3pid_msisdn_token(
                                    &client_secret,
                                    &country,
                                    &msisdn,
                                    matrix_sdk::ruma::uint!(1),
                                )
                                .await
                                .map_err(|e| e.to_string())?;
                            Ok(resp.sid.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(
                                Message::ThreepidTokenRequested(res),
                            ))
                        },
                    );
                }
                Task::none()
            }
            Message::ThreepidTokenRequested(res) => {
                self.is_requesting_3pid_token = false;
                match res {
                    Ok(sid) => {
                        self.adding_3pid_sid = Some(sid);
                        self.success_message = Some("Verification code sent. Please confirm the link/code and then provide your password to add it here.".to_string());
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to request verification token: {}", e));
                        self.adding_3pid_client_secret = None;
                    }
                }
                Task::none()
            }
            Message::Add3PID => {
                if let (Some(matrix), Some(sid), Some(secret)) = (
                    matrix,
                    &self.adding_3pid_sid,
                    &self.adding_3pid_client_secret,
                ) {
                    let matrix = matrix.clone();
                    let sid = sid.clone();
                    let secret = secret.clone();
                    let password = self.add_3pid_password.clone();

                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let user_id = client.user_id().ok_or("No user ID")?.to_string();
                            let sid_typed = matrix_sdk::ruma::SessionId::parse(sid)
                                .map_err(|e| e.to_string())?;
                            let secret_typed = matrix_sdk::ruma::ClientSecret::parse(secret)
                                .map_err(|e| e.to_string())?;

                            let res = client
                                .account()
                                .add_3pid(&secret_typed, &sid_typed, None)
                                .await;

                            match res {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    if let Some(info) = e.as_uiaa_response() {
                                        if password.is_empty() {
                                            return Err("Password required to add 3PID".to_string());
                                        }

                                        let identifier = matrix_sdk::ruma::api::client::uiaa::UserIdentifier::UserIdOrLocalpart(user_id);
                                        let mut password_auth =
                                            matrix_sdk::ruma::api::client::uiaa::Password::new(
                                                identifier, password,
                                            );
                                        password_auth.session = info.session.clone();

                                        client.account().add_3pid(&secret_typed, &sid_typed, Some(matrix_sdk::ruma::api::client::uiaa::AuthData::Password(password_auth))).await.map(|_| ()).map_err(|e| e.to_string())
                                    } else {
                                        Err(e.to_string())
                                    }
                                }
                            }
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::ThreepidAdded(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::ThreepidAdded(res) => {
                match res {
                    Ok(_) => {
                        self.new_3pid_email.clear();
                        self.new_3pid_msisdn.clear();
                        self.add_3pid_password.clear();
                        self.adding_3pid_sid = None;
                        self.adding_3pid_client_secret = None;
                        return self.update(Message::Load3PIDs, matrix);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to add 3PID: {}", e));
                    }
                }
                Task::none()
            }
            Message::DismissSuccessMessage => {
                self.success_message = None;
                Task::none()
            }
            Message::Delete3PID(address, medium) => {
                if let Some(matrix) = matrix {
                    let matrix = matrix.clone();
                    let addr = address.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            client
                                .account()
                                .delete_3pid(&addr, medium, None)
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(crate::Message::UserSettings(Message::ThreepidDeleted(
                                address.clone(),
                                res,
                            )))
                        },
                    );
                }
                Task::none()
            }
            Message::ThreepidDeleted(address, res) => {
                match res {
                    Ok(_) => {
                        self.threepids.retain(|t| t.address != address);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to delete 3PID: {}", e));
                    }
                }
                Task::none()
            }
            Message::LoadKeywords => {
                if let Some(matrix) = matrix {
                    self.is_loading_keywords = true;
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let ns = client.notification_settings().await;
                            let mut res = Vec::new();
                            for k in ns.enabled_keywords().await {
                                res.push(k.into());
                            }
                            res
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::KeywordsLoaded(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::KeywordsLoaded(res) => {
                self.is_loading_keywords = false;
                self.keywords = res;
                Task::none()
            }
            Message::NewKeywordChanged(keyword) => {
                self.new_keyword = keyword;
                Task::none()
            }
            Message::AddKeyword => {
                if let Some(matrix) = matrix
                    && !self.new_keyword.is_empty()
                {
                    self.is_loading_keywords = true;
                    let matrix = matrix.clone();
                    let keyword = self.new_keyword.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let ns = client.notification_settings().await;
                            ns.add_keyword(keyword).await.map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::KeywordAdded(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::KeywordAdded(res) => {
                match res {
                    Ok(_) => {
                        self.new_keyword.clear();
                        return self.update(Message::LoadKeywords, matrix);
                    }
                    Err(e) => {
                        self.error = Some(format!("Failed to add keyword: {}", e));
                    }
                }
                Task::none()
            }
            Message::RemoveKeyword(keyword) => {
                if let Some(matrix) = matrix {
                    self.is_loading_keywords = true;
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let client = matrix.client().await;
                            let ns = client.notification_settings().await;
                            ns.remove_keyword(&keyword).await.map_err(|e| e.to_string())
                        },
                        |res| {
                            Action::from(crate::Message::UserSettings(Message::KeywordRemoved(res)))
                        },
                    );
                }
                Task::none()
            }
            Message::KeywordRemoved(res) => {
                match res {
                    Ok(_) => {
                        return self.update(Message::LoadKeywords, matrix);
                    }
                    Err(e) => {
                        self.is_loading_keywords = false;
                        self.error = Some(format!("Failed to remove keyword: {}", e));
                    }
                }
                Task::none()
            }
        }
    }

    fn view_privacy<'a>(&'a self) -> Element<'a, Message> {
        use cosmic::widget::toggler;
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Privacy & Preferences"));

        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Display Media Previews"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(
                    toggler(self.media_previews_display_policy)
                        .on_toggle(Message::ToggleMediaPreviewsDisplayPolicy),
                ),
        );
        col = col.push(
            text::body("Automatically load and display media previews in the chat timeline.")
                .size(12),
        );

        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Display Invite Avatars"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(
                    toggler(self.invite_avatars_display_policy)
                        .on_toggle(Message::ToggleInviteAvatarsDisplayPolicy),
                ),
        );
        col = col.push(
            text::body("Display avatars for rooms and spaces you haven't joined yet.").size(12),
        );

        col.into()
    }

    fn view_notifications<'a>(&'a self) -> Element<'a, Message> {
        use matrix_sdk::notification_settings::RoomNotificationMode;
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Default Notification Settings"));

        let modes = [
            RoomNotificationMode::AllMessages,
            RoomNotificationMode::MentionsAndKeywordsOnly,
            RoomNotificationMode::Mute,
        ];

        // DMs
        col = col.push(text::body("Direct Messages").size(12));
        let mut dm_row = Row::new().spacing(10);
        for mode in modes {
            let label = match mode {
                RoomNotificationMode::AllMessages => "All Messages",
                RoomNotificationMode::MentionsAndKeywordsOnly => "Mentions Only",
                RoomNotificationMode::Mute => "Muted",
            };

            let mut btn = if self.global_notification_mode_dm == Some(mode) {
                button::suggested(label)
            } else {
                button::text(label)
            };

            if self.global_notification_mode_dm != Some(mode)
                && !self.is_loading_global_notifications
            {
                btn = btn.on_press(Message::GlobalNotificationModeChanged(true, mode));
            }
            dm_row = dm_row.push(btn);
        }
        col = col.push(dm_row);

        // Groups
        col = col.push(text::body("Group Chats").size(12));
        let mut group_row = Row::new().spacing(10);
        for mode in modes {
            let label = match mode {
                RoomNotificationMode::AllMessages => "All Messages",
                RoomNotificationMode::MentionsAndKeywordsOnly => "Mentions Only",
                RoomNotificationMode::Mute => "Muted",
            };

            let mut btn = if self.global_notification_mode_group == Some(mode) {
                button::suggested(label)
            } else {
                button::text(label)
            };

            if self.global_notification_mode_group != Some(mode)
                && !self.is_loading_global_notifications
            {
                btn = btn.on_press(Message::GlobalNotificationModeChanged(false, mode));
            }
            group_row = group_row.push(btn);
        }
        col.into()
    }

    fn view_keywords<'a>(&'a self) -> Element<'a, Message> {
        let mut col = Column::new().spacing(10);
        col = col.push(text::title3("Keyword Notifications"));

        col = col.push(
            text::body("Receive notifications when these keywords are mentioned in any room.")
                .size(12),
        );

        if self.is_loading_keywords {
            col = col.push(text::body("Loading keywords..."));
        } else {
            for keyword in &self.keywords {
                let mut row = Row::new().spacing(10).align_y(Alignment::Center);
                row = row.push(text::body(keyword).size(14));
                row = row.push(cosmic::widget::space().width(cosmic::iced::Length::Fill));
                row = row.push(tooltip(
                    button::icon(Named::new("user-trash-symbolic"))
                        .on_press(Message::RemoveKeyword(keyword.clone())),
                    text::body("Remove Keyword"),
                    Position::Top,
                ));
                col = col.push(row);
            }

            let mut add_row = Row::new().spacing(10).align_y(Alignment::Center);
            add_row = add_row.push(
                text_input("New keyword", &self.new_keyword)
                    .on_input(Message::NewKeywordChanged)
                    .on_submit(|_| Message::AddKeyword),
            );

            let mut add_btn = button::text("Add");
            if !self.new_keyword.is_empty() {
                add_btn = add_btn.on_press(Message::AddKeyword);
            }
            add_row = add_row.push(add_btn);
            col = col.push(add_row);
        }

        col.into()
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
                let has_changes = self.display_name != self.original_display_name;

                if has_changes {
                    btn = btn.on_press(Message::SaveProfile);
                }

                let widget: Element<'_, Message> = if !has_changes {
                    tooltip(btn, text::body("Make changes to save"), Position::Top).into()
                } else {
                    btn.into()
                };

                save_row = save_row.push(widget);
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

        let is_empty = self.current_password.is_empty()
            || self.new_password.is_empty()
            || self.confirm_new_password.is_empty();

        let passwords_match = self.new_password == self.confirm_new_password;

        let mut pw_btn = button::text(if self.is_changing_password {
            "Changing..."
        } else {
            "Change Password"
        });

        if !self.is_changing_password && !is_empty && passwords_match {
            pw_btn = pw_btn.on_press(Message::ChangePassword);
        }

        let pw_btn_widget: Element<'_, Message> = if !self.is_changing_password {
            if is_empty {
                tooltip(
                    pw_btn,
                    text::body("Fill in all fields to change password"),
                    Position::Top,
                )
                .into()
            } else if !passwords_match {
                tooltip(
                    pw_btn,
                    text::body("New passwords do not match"),
                    Position::Top,
                )
                .into()
            } else {
                pw_btn.into()
            }
        } else {
            pw_btn.into()
        };

        col = col.push(pw_btn_widget);

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

    fn view_cross_signing<'a>(&'a self) -> Element<'a, Message> {
        let mut col = Column::new()
            .spacing(10)
            .push(text::title3("Cross-signing"));

        if self.is_loading_cross_signing {
            col = col.push(text::body("Loading cross-signing status..."));
        } else if let Some(info) = &self.cross_signing_info {
            let status = &info.status;
            let mut status_col = Column::new().spacing(5);

            status_col =
                status_col.push(Row::new().spacing(10).push(text::body("Master Key:")).push(
                    text::body(if status.has_master {
                        "✅ Present"
                    } else {
                        "❌ Missing"
                    }),
                ));

            if let Some(key) = &info.master_key {
                status_col = status_col.push(text::body(format!("Public Key: {}", key)).size(10));
            }

            status_col = status_col.push(
                Row::new()
                    .spacing(10)
                    .push(text::body("Self-signing Key:"))
                    .push(text::body(if status.has_self_signing {
                        "✅ Present"
                    } else {
                        "❌ Missing"
                    })),
            );

            if let Some(key) = &info.self_signing_key {
                status_col = status_col.push(text::body(format!("Public Key: {}", key)).size(10));
            }

            status_col = status_col.push(
                Row::new()
                    .spacing(10)
                    .push(text::body("User-signing Key:"))
                    .push(text::body(if status.has_user_signing {
                        "✅ Present"
                    } else {
                        "❌ Missing"
                    })),
            );

            if let Some(key) = &info.user_signing_key {
                status_col = status_col.push(text::body(format!("Public Key: {}", key)).size(10));
            }

            col = col.push(status_col);

            if !status.is_complete() {
                let mut btn = button::text(if self.is_bootstrapping {
                    "Bootstrapping..."
                } else {
                    "Bootstrap Cross-signing"
                });

                if !self.is_bootstrapping {
                    btn = btn.on_press(Message::BootstrapCrossSigning);
                }

                col = col.push(btn);
            }
        } else {
            col = col.push(text::body("Cross-signing is not set up."));
            let mut btn = button::text(if self.is_bootstrapping {
                "Bootstrapping..."
            } else {
                "Bootstrap Cross-signing"
            });

            if !self.is_bootstrapping {
                btn = btn.on_press(Message::BootstrapCrossSigning);
            }
            col = col.push(btn);
        }

        col.into()
    }

    fn view_3pids<'a>(&'a self) -> Element<'a, Message> {
        let mut col = Column::new()
            .spacing(10)
            .push(text::title3("Emails & Phone Numbers"));

        if self.is_loading_3pids {
            col = col.push(text::body("Loading linked identifiers..."));
        } else {
            for t in &self.threepids {
                let mut row = Row::new().spacing(10).align_y(Alignment::Center);
                let icon = match t.medium {
                    matrix_sdk::ruma::thirdparty::Medium::Email => "mail-unread-symbolic",
                    matrix_sdk::ruma::thirdparty::Medium::Msisdn => "phone-symbolic",
                    _ => "dialog-question-symbolic",
                };

                row = row
                    .push(button::icon(Named::new(icon)))
                    .push(text::body(t.address.clone()))
                    .push(cosmic::widget::space().width(cosmic::iced::Length::Fill));

                let del_btn = button::destructive("Remove")
                    .on_press(Message::Delete3PID(t.address.clone(), t.medium.clone()));
                row = row.push(del_btn);

                col = col.push(row);
            }

            col = col.push(text::body("Add Email Address").size(14));
            let mut email_row = Row::new().spacing(10).align_y(Alignment::Center);
            email_row = email_row.push(
                text_input("email@example.com", &self.new_3pid_email)
                    .on_input(Message::New3PIDEmailChanged),
            );

            if self.adding_3pid_sid.is_none() {
                let mut btn = button::text(if self.is_requesting_3pid_token {
                    "Sending..."
                } else {
                    "Send"
                });
                if !self.is_requesting_3pid_token && !self.new_3pid_email.is_empty() {
                    btn = btn.on_press(Message::Request3PIDEmailToken);
                }
                email_row = email_row.push(btn);
            }
            col = col.push(email_row);

            col = col.push(text::body("Add Phone Number").size(14));
            let mut phone_row = Row::new().spacing(10).align_y(Alignment::Center);
            phone_row = phone_row.push(
                text_input("Country (e.g. US)", &self.new_3pid_country_code)
                    .on_input(Message::New3PIDCountryCodeChanged)
                    .width(100),
            );
            phone_row = phone_row.push(
                text_input("Phone Number", &self.new_3pid_msisdn)
                    .on_input(Message::New3PIDMsisdnChanged),
            );

            if self.adding_3pid_sid.is_none() {
                let mut btn = button::text(if self.is_requesting_3pid_token {
                    "Sending..."
                } else {
                    "Send"
                });
                if !self.is_requesting_3pid_token
                    && !self.new_3pid_msisdn.is_empty()
                    && !self.new_3pid_country_code.is_empty()
                {
                    btn = btn.on_press(Message::Request3PIDMsisdnToken);
                }
                phone_row = phone_row.push(btn);
            }
            col = col.push(phone_row);

            if let Some(_sid) = &self.adding_3pid_sid {
                col = col.push(text::body("Complete Addition").size(14));
                let mut complete_row = Row::new().spacing(10).align_y(Alignment::Center);
                complete_row = complete_row.push(
                    text_input("Account Password", &self.add_3pid_password)
                        .password()
                        .on_input(Message::Add3PIDPasswordChanged),
                );
                complete_row =
                    complete_row.push(button::suggested("Add").on_press(Message::Add3PID));
                col = col.push(complete_row);
            }
        }

        col.into()
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

        col = col.push(self.view_notifications());

        col = col.push(self.view_privacy());

        col = col.push(self.view_keywords());

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

        col = col.push(self.view_cross_signing());

        col = col.push(self.view_3pids());

        if let Some(msg) = &self.success_message {
            col = col.push(
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(text::body(msg))
                    .push(button::text("Dismiss").on_press(Message::DismissSuccessMessage)),
            );
        }

        col = col.push(self.view_verification());

        col.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_dismiss_error() {
        let mut state = State::default();
        state.error = Some("Test error".to_string());

        let _ = state.update(Message::DismissError, &None);

        assert_eq!(state.error, None);
    }

    #[test]
    fn test_password_changed() {
        let mut state = State::default();

        let _ = state.update(
            Message::CurrentPasswordChanged("old_pass".to_string()),
            &None,
        );
        assert_eq!(state.current_password, "old_pass");

        let _ = state.update(Message::NewPasswordChanged("new_pass".to_string()), &None);
        assert_eq!(state.new_password, "new_pass");

        let _ = state.update(
            Message::ConfirmNewPasswordChanged("new_pass".to_string()),
            &None,
        );
        assert_eq!(state.confirm_new_password, "new_pass");
    }

    #[test]
    fn test_display_name_changed() {
        let mut state = State::default();

        let _ = state.update(Message::DisplayNameChanged("Alice".to_string()), &None);
        assert_eq!(state.display_name, "Alice");
    }

    #[test]
    fn test_device_rename_flow() {
        let mut state = State::default();
        let device_id: Arc<str> = Arc::from("DEVICE_1");

        // Setup initial device
        state.devices.push(DeviceInfo {
            device_id: device_id.clone(),
            display_name: Some("My Phone".to_string()),
            is_verified: true,
            is_current: false,
            is_renaming: false,
            edit_name: "".to_string(),
            is_deleting: false,
        });

        // Start rename
        let _ = state.update(Message::StartRenameDevice(device_id.clone()), &None);
        assert!(state.devices[0].is_renaming);
        assert_eq!(state.devices[0].edit_name, "My Phone");

        // Edit name
        let _ = state.update(
            Message::EditDeviceNameChanged(device_id.clone(), "My New Phone".to_string()),
            &None,
        );
        assert_eq!(state.devices[0].edit_name, "My New Phone");

        // Cancel rename
        let _ = state.update(Message::CancelRenameDevice(device_id.clone()), &None);
        assert!(!state.devices[0].is_renaming);
        // edit_name should be preserved as it was updated
        assert_eq!(state.devices[0].edit_name, "My New Phone");
    }

    #[test]
    fn test_cross_signing_messages() {
        let mut state = State::default();

        // Load status starts loading
        let _ = state.update(Message::LoadCrossSigningStatus, &None);
        assert!(state.is_loading_cross_signing);

        // Status loaded
        let info = CrossSigningInfo {
            status: CrossSigningStatus {
                has_master: true,
                has_self_signing: false,
                has_user_signing: true,
            },
            master_key: Some("master_pub".to_string()),
            self_signing_key: None,
            user_signing_key: Some("user_pub".to_string()),
        };
        let _ = state.update(
            Message::CrossSigningStatusLoaded(Ok(Some(info.clone()))),
            &None,
        );
        assert!(!state.is_loading_cross_signing);
        assert!(state.cross_signing_info.is_some());
        let info_state = state.cross_signing_info.as_ref().unwrap();
        assert!(info_state.status.has_master);
        assert!(!info_state.status.has_self_signing);
        assert_eq!(info_state.master_key.as_deref(), Some("master_pub"));

        // Bootstrap
        let _ = state.update(Message::BootstrapCrossSigning, &None);
        assert!(state.is_bootstrapping);

        // Bootstrapped
        let _ = state.update(Message::CrossSigningBootstrapped(Ok(())), &None);
        assert!(!state.is_bootstrapping);
    }

    #[test]
    fn test_threepids_loaded() {
        let mut state = State::default();
        let threepids = vec![Threepid {
            address: "test@example.com".to_string(),
            medium: matrix_sdk::ruma::thirdparty::Medium::Email,
        }];

        let _ = state.update(Message::ThreepidsLoaded(Ok(threepids.clone())), &None);

        assert!(!state.is_loading_3pids);
        assert_eq!(state.threepids, threepids);
    }

    #[test]
    fn test_new_3pid_email_changed() {
        let mut state = State::default();
        let email = "new@example.com".to_string();

        let _ = state.update(Message::New3PIDEmailChanged(email.clone()), &None);

        assert_eq!(state.new_3pid_email, email);
    }

    #[test]
    fn test_threepid_deleted() {
        let mut state = State::default();
        let address = "test@example.com".to_string();
        state.threepids = vec![Threepid {
            address: address.clone(),
            medium: matrix_sdk::ruma::thirdparty::Medium::Email,
        }];

        let _ = state.update(Message::ThreepidDeleted(address, Ok(())), &None);

        assert!(state.threepids.is_empty());
    }

    #[test]
    fn test_keywords_state() {
        let mut state = State::default();

        let _ = state.update(Message::NewKeywordChanged("rust".to_string()), &None);
        assert_eq!(state.new_keyword, "rust");

        let keywords = vec!["matrix".to_string(), "cosmic".to_string()];
        let _ = state.update(Message::KeywordsLoaded(keywords.clone()), &None);
        assert_eq!(state.keywords, keywords);
        assert!(!state.is_loading_keywords);
    }
}
