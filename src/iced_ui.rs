use iced::{Alignment, Element, Sandbox, Settings, Size, window};
use iced::widget::column;
use iced::window::Position;

pub fn main() -> iced::Result {
    Counter::run(Settings {
        window: window::Settings {
            size: Size::new(400_f32, 400_f32),
            position: Position::Centered,
            ..Default::default()
        },
        ..Default::default()
    })
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    IncrementPressed,
    DecrementPressed,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self { value: 0 }
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::IncrementPressed => {
                self.value += 1;
            }
            Message::DecrementPressed => {
                self.value -= 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
           button("Increment").on_press(Message::IncrementPressed),
            text(self.value).size(50),
            button("Decrement").on_press(Message::DecrementPressed)
        ].padding(20)
            .align_items(Alignment::Center)
            .into()
    }
}