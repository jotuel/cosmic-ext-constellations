mod matrix;

use cosmic::iced::{Alignment, Subscription};
use cosmic::widget::{column, row, scrollable, text};
use cosmic::{Application, Element, Task, Core, Action};
use std::path::PathBuf;

struct Claw {
    core: Core,
    matrix: matrix::MatrixEngine,
    sync_status: matrix::SyncStatus,
    room_list: Vec<matrix::RoomData>,
}

#[derive(Debug, Clone)]
enum Message {
    Matrix(matrix::MatrixEvent),
}

#[derive(Clone, Debug)]
struct MatrixEngineWrapper(matrix::MatrixEngine);

impl std::hash::Hash for MatrixEngineWrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        "matrix-sync".hash(state);
    }
}

impl PartialEq for MatrixEngineWrapper {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Eq for MatrixEngineWrapper {}

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
            room_list: Vec::new(),
        }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        match message {
            Message::Matrix(event) => {
                match event {
                    matrix::MatrixEvent::SyncStatusChanged(status) => {
                        self.sync_status = status;
                    }
                    matrix::MatrixEvent::RoomInserted(index, data) => {
                        if index <= self.room_list.len() {
                            self.room_list.insert(index, data);
                        } else {
                            self.room_list.push(data);
                        }
                    }
                    matrix::MatrixEvent::RoomRemoved(index) => {
                        if index < self.room_list.len() {
                            self.room_list.remove(index);
                        }
                    }
                    matrix::MatrixEvent::RoomUpdated(index, data) => {
                        if index < self.room_list.len() {
                            self.room_list[index] = data;
                        }
                    }
                    matrix::MatrixEvent::RoomListReset => {
                        self.room_list.clear();
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

        let sidebar = column()
            .spacing(10)
            .padding(10)
            .width(250)
            .push(text::title3("Rooms"));

        let room_list = self.room_list.iter().fold(column().spacing(5), |col, room| {
            col.push(text::body(room.name.as_deref().unwrap_or("Unknown Room")))
        });

        let sidebar = sidebar.push(scrollable(room_list));

        let content = column()
            .spacing(20)
            .padding(20)
            .width(cosmic::iced::Length::Fill)
            .align_x(Alignment::Center)
            .push(text::title1("Claw - Matrix Client"))
            .push(text::body(format!("Status: {}", status_text)))
            .push(text::body("Select a room to start chatting"));

        row()
            .push(sidebar)
            .push(content)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::run_with(
            MatrixEngineWrapper(self.matrix.clone()),
            |wrapper| {
                let engine = wrapper.0.clone();
                let client = engine.client().clone();
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                
                let _ = tx.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(matrix::SyncStatus::Syncing)));
                
                let tx_sync = tx.clone();
                let client_sync = client.clone();
                tokio::spawn(async move {
                    match client_sync.sync(matrix_sdk::config::SyncSettings::default()).await {
                        Ok(_) => {
                            let _ = tx_sync.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(matrix::SyncStatus::Connected)));
                        }
                        Err(_) => {
                            let _ = tx_sync.send(Message::Matrix(matrix::MatrixEvent::SyncStatusChanged(matrix::SyncStatus::Error)));
                        }
                    }
                });

                let tx_rooms = tx.clone();
                let engine_rooms = engine.clone();
                tokio::spawn(async move {
                    let room_list_service = engine_rooms.room_list_service();
                    let (_entries, stream) = match room_list_service.all_rooms().await {
                        Ok(rooms) => rooms.entries(),
                        Err(_) => return,
                    };

                    use cosmic::iced::futures::StreamExt;
                    let mut stream = stream;
                    while let Some(diffs) = stream.next().await {
                        for diff in diffs {
                            match diff {
                                eyeball_im::VectorDiff::Insert { index, value } => {
                                    if let Some(room_data) = get_room_data(&engine_rooms, &value).await {
                                        let _ = tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomInserted(index, room_data)));
                                    }
                                }
                                eyeball_im::VectorDiff::Remove { index } => {
                                    let _ = tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomRemoved(index)));
                                }
                                eyeball_im::VectorDiff::Set { index, value } => {
                                    if let Some(room_data) = get_room_data(&engine_rooms, &value).await {
                                        let _ = tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomUpdated(index, room_data)));
                                    }
                                }
                                eyeball_im::VectorDiff::Reset { values } => {
                                    let _ = tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomListReset));
                                    for (index, value) in values.into_iter().enumerate() {
                                        if let Some(room_data) = get_room_data(&engine_rooms, &value).await {
                                            let _ = tx_rooms.send(Message::Matrix(matrix::MatrixEvent::RoomInserted(index, room_data)));
                                        }
                                    }
                                }
                                _ => {}
                            }
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

async fn get_room_data(engine: &matrix::MatrixEngine, entry: &matrix_sdk_ui::room_list_service::RoomListEntry) -> Option<matrix::RoomData> {
    let room_id = match entry {
        matrix_sdk_ui::room_list_service::RoomListEntry::Filled(id) => id,
        matrix_sdk_ui::room_list_service::RoomListEntry::Invalidated(id) => id,
        _ => return None,
    };

    let client = engine.client();
    let room = client.get_room(room_id)?;
    
    let name = room.display_name().await.ok().map(|n| n.to_string());
    
    Some(matrix::RoomData {
        id: room_id.to_string(),
        name,
        last_message: None, // Simplified for now
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic::app::run::<Claw>(cosmic::app::Settings::default(), ())?;
    Ok(())
}
