use iced::widget::text;
use iced::{Element, Sandbox, Settings};
 
 // Run returns a result for errors and etc
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor;

#[derive(Debug)]
enum Message {}

impl Sandbox for Editor {
    // Uma mensagem é um evento ou interações do usuário que a aplicação pode lidar ou reagir, ex: clique de um botão
    // A message is an event or user interaction that the application can handle or interact with, ex: a click in a button
    type Message = Message;

    // Dita o estado da aplicação ao iniciar
    // Application initial state
    fn new() -> Self {
        Self
    }

    // Título da aplicação
    // Application title
    fn title(&self) -> String {
        String::from("A cool editor!")
    }

    // Lógica para lidar com as mensagens;
    // Logic that handles messages
    fn update(&mut self, message: Message) {
        match message {}
    }

    // Lógica que produz os widgets da interface
    // Logic that produces the interface widgets
    fn view(&self) -> iced::Element<'_, Message> {
        text("Hello, iced!").into()
    }
}