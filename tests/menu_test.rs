use cosmic::app::Core;
use cosmic::iced::Alignment;
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::{Column, RcElementWrapper};
use cosmic::widget::{button, container, menu, text};
use cosmic::{Action, Application, Element, Task};
use std::collections::HashMap;

fn main() {}

pub struct App {
    core: Core,
}

#[derive(Clone, Debug)]
pub enum Message {
    Logout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MenuAct {
    Logout,
}

impl MenuAction for MenuAct {
    type Message = Message;
    fn message(&self) -> Self::Message {
        match self {
            MenuAct::Logout => Message::Logout,
        }
    }
}

impl Application for App {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = ();
    const APP_ID: &'static str = "com.test.menu";

    fn core(&self) -> &Core {
        &self.core
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }
    fn init(core: Core, _flags: ()) -> (Self, Task<Action<Self::Message>>) {
        (Self { core }, Task::none())
    }
    fn update(&mut self, _message: Message) -> Task<Action<Self::Message>> {
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        let avatar = container(text::body("U").size(24))
            .padding(8)
            .align_x(Alignment::Center);

        let user_btn = button::custom(avatar);
        let key_binds: HashMap<cosmic::widget::menu::key_bind::KeyBind, MenuAct> = HashMap::new();

        let menu_tree = menu::Tree::with_children(
            RcElementWrapper::new(Element::from(user_btn)),
            menu::items(
                &key_binds,
                vec![menu::Item::Button("Logout", None, MenuAct::Logout)],
            ),
        );

        let user_menu = menu::bar(vec![menu_tree])
            .item_height(menu::ItemHeight::Dynamic(40))
            .item_width(menu::ItemWidth::Uniform(120))
            .spacing(4.0);

        Column::new().push(user_menu).into()
    }
}
