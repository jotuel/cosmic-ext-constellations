use cosmic::{Action, Task};

use crate::{Constellations, Message};

pub mod chat;
pub mod login;
pub mod switcher;

impl Constellations {
    pub fn update_title(&mut self) -> Task<Action<Message>> {
        let selected_room_name = self.selected_room.as_ref().and_then(|id| {
            self.room_list
                .iter()
                .find(|r| &r.id == id)
                .and_then(|r| r.name.as_deref())
        });

        let title = selected_room_name.unwrap_or("Constellations - Matrix Client");
        self.core.set_header_title(title.to_string());
        Task::none()
    }
}
