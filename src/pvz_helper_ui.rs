use iced::{Element, Sandbox, Settings, Size, window, Font, Alignment};
use iced::widget::{text, checkbox, button, text_editor, Row, Column};
use iced::window::Position;
use tracing_subscriber::fmt::time::LocalTime;

mod address;

mod pvz_helper;

use pvz_helper::PVZHelper;

pub fn main() -> iced::Result {
    tracing_subscriber::fmt().with_env_filter("pvz_helper_ui=trace")
        .with_thread_names(true)
        .with_timer(LocalTime::rfc_3339()).init();
    UI::run(Settings {
        window: window::Settings {
            size: Size::new(400_f32, 400_f32),
            resizable: false,
            position: Position::Centered,
            ..Default::default()
        },
        default_font: Font::with_name("微软雅黑"),
        ..Default::default()
    })
}

#[derive(Debug, Clone)]
enum Message {
    NoCDToggled(bool),
    Modify,
    ActionPerformed(text_editor::Action),
}

struct UI {
    value: text_editor::Content,
    helper: PVZHelper,
    no_cd: bool,
}

impl Sandbox for UI {
    type Message = Message;

    fn new() -> Self {
        UI { value: text_editor::Content::with_text("9999"), helper: PVZHelper::new(env!("PROGRAM_TITLE")), no_cd: false }
    }

    fn title(&self) -> String {
        "PVZ_Helper".into()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::NoCDToggled(v) => {
                self.no_cd = v;
                self.helper.modify_cd(v);
            }
            Message::Modify => {
                match self.value.text().strip_suffix('\n').unwrap().parse() {
                    Ok(v) => { self.helper.modify_sun(v); }
                    Err(e) => {
                        tracing::error!("{}", e);
                        self.value = text_editor::Content::new();
                    }
                }
            }
            Message::ActionPerformed(action) => {
                self.value.perform(action)
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let no_cd_checkbox = checkbox("无冷却模式", self.no_cd).on_toggle(Message::NoCDToggled);
        let value_editor = text_editor(&self.value).on_action(Message::ActionPerformed);
        let sun_text = text("修改阳光: ");
        let modify_button = button("修改").on_press(Message::Modify);
        let rows = Row::with_children([
            Element::from(sun_text),
            Element::from(value_editor),
            Element::from(modify_button)
        ]).spacing(10).align_items(Alignment::Center);
        Column::with_children([
            Element::from(no_cd_checkbox),
            Element::from(rows)
        ]).into()
    }
}