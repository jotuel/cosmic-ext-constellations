use cosmic::iced::Alignment;
use cosmic::widget::{Column, Row, button, text, toggler};
use cosmic::{Action, Element, Task};

#[derive(Debug, Clone, Default)]
pub struct State {
    pub show_sync_indicator: bool,
    pub send_typing_notifications: bool,
    pub render_markdown: bool,
    pub compact_mode: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleSyncIndicator(bool),
    ToggleTypingNotifications(bool),
    ToggleMarkdown(bool),
    ToggleCompactMode(bool),
    ClearCache,
}

impl State {
    pub fn update(&mut self, message: Message) -> Task<Action<crate::Message>> {
        match message {
            Message::ToggleSyncIndicator(show) => {
                self.show_sync_indicator = show;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleTypingNotifications(send) => {
                self.send_typing_notifications = send;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleMarkdown(render) => {
                self.render_markdown = render;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleCompactMode(compact) => {
                self.compact_mode = compact;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ClearCache => Task::done(Action::from(crate::Message::AppSettings(
                Message::ClearCache,
            ))),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut col = Column::new().spacing(20);

        col = col.push(text::title3("General Settings"));

        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Show Sync Indicator"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(toggler(self.show_sync_indicator).on_toggle(Message::ToggleSyncIndicator)),
        );
        col = col.push(text::body("Display a small indicator in the header when the app is actively syncing with Matrix servers.").size(12));

        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Send Typing Notifications"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(
                    toggler(self.send_typing_notifications)
                        .on_toggle(Message::ToggleTypingNotifications),
                ),
        );

        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Render Markdown"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(toggler(self.render_markdown).on_toggle(Message::ToggleMarkdown)),
        );

        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Compact Mode"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(toggler(self.compact_mode).on_toggle(Message::ToggleCompactMode)),
        );

        col = col.push(text::title3("Maintenance"));
        col = col.push(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(text::body("Media Cache"))
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(button::text("Clear Cache").on_press(Message::ClearCache)),
        );

        col.into()
    }
}
