use cosmic::Element;
use cosmic::widget::{button, container, text, Row};
use cosmic::iced::Alignment;
use crate::Message;

pub fn view_error(error: &str) -> Element<'_, Message> {
    let error_card = container(
        Row::new()
            .spacing(12)
            .align_y(Alignment::Center)
            .push(cosmic::widget::icon::from_name("dialog-error-symbolic").size(20))
            .push(text::body(error.to_string()))
            .push(button::text(crate::fl!("dismiss")).on_press(Message::DismissError))
    )
    .padding(16)
    .max_width(500);

    container(error_card)
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fill)
        .padding(20)
        .align_x(Alignment::Center)
        .align_y(Alignment::Start)
        .into()
}
