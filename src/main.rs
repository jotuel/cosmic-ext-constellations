mod matrix;

use cosmic::iced::{Alignment, Subscription};
use cosmic::widget::{column, text};
use cosmic::{Application, Element, Task, Core, Action};
use std::path::PathBuf;

struct Claw {
    core: Core,
    matrix: matrix::MatrixEngine,
    sync_status: matrix::SyncStatus,
}

#[derive(Debug, Clone)]
enum Message {
    Matrix(matrix::MatrixEvent),
}

#[derive(Clone, Debug)]
struct MatrixClientWrapper(matrix_sdk::Client);

impl std::hash::Hash for MatrixClientWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use std::hash::Hash;
        "matrix-sync".hash(state);
    }
}

impl PartialEq for MatrixClientWrapper {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for MatrixClientWrapper {}

impl Application for Claw {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = ();
    const APP_ID: &'static str = "com.system76.Claw";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("com.system76.Claw");

        // Use block_on to initialize the Matrix engine during the synchronous libcosmic init phase.
        // In a production app, this should be handled via an async Task to avoid blocking the UI thread.
        let matrix = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(matrix::MatrixEngine::new(data_dir))
            .expect("Failed to initialize Matrix engine");

        (Claw { 
            core, 
            matrix,
            sync_status: matrix::SyncStatus::Disconnected,
        }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        match message {
            Message::Matrix(event) => {
                match event {
                    matrix::MatrixEvent::SyncStatusChanged(status) => {
                        self.sync_status = status;
                    }
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let status_text = match self.sync_status {
            matrix::SyncStatus::Disconnected => "Disconnected",
            matrix::SyncStatus::Syncing => "Syncing...",
            matrix::SyncStatus::Connected => "Connected",
            matrix::SyncStatus::Error => "Sync Error",
        };

        column()
            .spacing(20)
            .padding(20)
            .align_x(Alignment::Center)
            .push(text::title1("Claw - Matrix Client"))
            .push(text::body(format!("Status: {}", status_text)))
            .push(text::body("Select a room to start chatting"))
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let client = self.matrix.client().clone();

        Subscription::run_with(
            MatrixClientWrapper(client),
            |wrapper| {
                let client = wrapper.0.clone();
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                
                let _ = tx.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(matrix::SyncStatus::Syncing)));
                
                tokio::spawn(async move {
                    match client.sync(matrix_sdk::config::SyncSettings::default()).await {
                        Ok(_) => {
                            let _ = tx.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(matrix::SyncStatus::Connected)));
                        }
                        Err(_) => {
                            let _ = tx.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(matrix::SyncStatus::Error)));
                        }
                    }
                });

                cosmic::iced::futures::stream::unfold(rx, |mut rx| async move {
                    rx.recv().await.map(|msg| (msg, rx))
                })
            },
        )
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic::app::run::<Claw>(cosmic::app::Settings::default(), ())?;
    Ok(())
}
