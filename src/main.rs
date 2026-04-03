mod matrix;

use cosmic::iced::{Alignment, Subscription};
use cosmic::widget::{button, column, row, scrollable, text, container};
use cosmic::{Application, Element, Task, Core, Action};
use std::path::PathBuf;
use std::sync::Arc;
use imbl::Vector;

struct Claw {
    core: Core,
    matrix: Option<matrix::MatrixEngine>,
    sync_status: matrix::SyncStatus,
    room_list: Vec<matrix::RoomData>,
    selected_room: Option<String>,
    timeline_items: Vector<Arc<matrix::TimelineItem>>,
}

#[derive(Debug, Clone)]
enum Message {
    Matrix(matrix::MatrixEvent),
    RoomSelected(String),
    EngineReady(matrix::MatrixEngine),
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

        (Claw { 
            core, 
            matrix: None,
            sync_status: matrix::SyncStatus::Disconnected,
            room_list: Vec::new(),
            selected_room: None,
            timeline_items: Vector::new(),
        }, Task::perform(async move {
            matrix::MatrixEngine::new(data_dir).await.expect("Failed to initialize Matrix engine")
        }, |engine| Action::from(Message::EngineReady(engine))))
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        match message {
            Message::EngineReady(engine) => {
                self.matrix = Some(engine);
            }
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
                    matrix::MatrixEvent::TimelineDiff(diff) => {
                        match diff {
                            eyeball_im::VectorDiff::Insert { index, value } => {
                                if index <= self.timeline_items.len() {
                                    self.timeline_items.insert(index, value);
                                } else {
                                    self.timeline_items.push_back(value);
                                }
                            }
                            eyeball_im::VectorDiff::Remove { index } => {
                                if index < self.timeline_items.len() {
                                    self.timeline_items.remove(index);
                                }
                            }
                            eyeball_im::VectorDiff::Set { index, value } => {
                                if index < self.timeline_items.len() {
                                    self.timeline_items.set(index, value);
                                }
                            }
                            eyeball_im::VectorDiff::Reset { values } => {
                                self.timeline_items = values;
                            }
                            eyeball_im::VectorDiff::PushBack { value } => {
                                self.timeline_items.push_back(value);
                            }
                            eyeball_im::VectorDiff::PushFront { value } => {
                                self.timeline_items.push_front(value);
                            }
                            eyeball_im::VectorDiff::PopBack => {
                                self.timeline_items.pop_back();
                            }
                            eyeball_im::VectorDiff::PopFront => {
                                self.timeline_items.pop_front();
                            }
                            eyeball_im::VectorDiff::Clear => {
                                self.timeline_items.clear();
                            }
                            _ => {}
                        }
                    }
                    matrix::MatrixEvent::TimelineReset => {
                        self.timeline_items.clear();
                    }
                }
            }
            Message::RoomSelected(room_id) => {
                self.selected_room = Some(room_id);
                self.timeline_items.clear();
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
            let name = room.name.as_deref().unwrap_or("Unknown Room");
            let room_id = room.id.clone();
            
            col.push(
                button::text(name)
                    .on_press(Message::RoomSelected(room_id))
                    .width(cosmic::iced::Length::Fill)
            )
        });

        let sidebar = sidebar.push(scrollable(room_list));

        let selected_room_name = self.selected_room.as_ref().and_then(|id| {
            self.room_list.iter().find(|r| &r.id == id).and_then(|r| r.name.as_deref())
        }).unwrap_or("Select a room to start chatting");

        let mut content = column()
            .spacing(20)
            .padding(20)
            .width(cosmic::iced::Length::Fill)
            .push(text::title1("Claw - Matrix Client"))
            .push(text::body(format!("Status: {}", status_text)))
            .push(text::body(selected_room_name));

        if self.selected_room.is_some() {
            let timeline = self.timeline_items.iter().fold(column().spacing(10), |col, item| {
                if let Some(event) = item.as_event() {
                    if let Some(message) = event.content().as_message() {
                        let sender = event.sender().to_string();
                        let body = message.body().to_string();
                        
                        col.push(
                            container(
                                column()
                                    .spacing(2)
                                    .push(text::body(sender).size(12))
                                    .push(text::body(body))
                            )
                            .padding(10)
                        )
                    } else {
                        col
                    }
                } else {
                    col
                }
            });
            content = content.push(scrollable(timeline).height(cosmic::iced::Length::Fill));
        } else {
            content = content.align_x(Alignment::Center);
        }

        row()
            .push(sidebar)
            .push(content)
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let matrix = match &self.matrix {
            Some(m) => m,
            None => return Subscription::none(),
        };

        let sync_sub = Subscription::run_with(
            MatrixEngineWrapper(matrix.clone()),
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
        );

        if let Some(room_id) = self.selected_room.clone() {
            let timeline_sub = Subscription::run_with(
                (MatrixEngineWrapper(matrix.clone()), room_id.clone()),
                |(wrapper, room_id)| {
                    let engine = wrapper.0.clone();
                    let room_id = room_id.clone();
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                    
                    tokio::spawn(async move {
                        let timeline = match engine.timeline(&room_id).await {
                            Ok(t) => t,
                            Err(_) => return,
                        };

                        let (items, mut stream) = timeline.subscribe().await;
                        let _ = tx.send(Message::Matrix(matrix::MatrixEvent::TimelineReset));
                        
                        for (index, item) in items.into_iter().enumerate() {
                            let _ = tx.send(Message::Matrix(matrix::MatrixEvent::TimelineDiff(
                                eyeball_im::VectorDiff::Insert { index, value: item }
                            )));
                        }

                        use cosmic::iced::futures::StreamExt;
                        while let Some(diff) = stream.next().await {
                            let _ = tx.send(Message::Matrix(matrix::MatrixEvent::TimelineDiff(diff)));
                        }
                    });

                    cosmic::iced::futures::stream::unfold(rx, |mut rx| async move {
                        rx.recv().await.map(|msg| (msg, rx))
                    })
                }
            );
            Subscription::batch([sync_sub, timeline_sub])
        } else {
            sync_sub
        }
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
