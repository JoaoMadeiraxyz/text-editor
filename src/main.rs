use iced::widget::{column, container, horizontal_space, row, text, text_editor};
use iced::{Element, Length, Sandbox, Settings, Theme};
 
 // Run returns a result for errors and etc
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
}

// Messages should generally to be clone because they represent pure events
#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action)
}

impl Sandbox for Editor {
    // Uma mensagem é um evento ou interações do usuário que a aplicação pode lidar ou reagir, ex: clique de um botão
    // A message is an event or user interaction that the application can handle or interact with, ex: a click in a button
    type Message = Message;

    // Dita o estado da aplicação ao iniciar
    // Application initial state
    fn new() -> Self {
        Self {
            content: text_editor::Content::with(include_str!("main.rs")),
        }
    }

    // Título da aplicação
    // Application title
    fn title(&self) -> String {
        String::from("A cool editor!")
    }

    // Lógica para lidar com as mensagens;
    // Logic that handles messages
    fn update(&mut self, message: Message) {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
            }
        }
    }

    // Lógica que produz os widgets da interface
    // Logic that produces the interface widgets
    fn view(&self) -> Element<'_, Message> {
        let input = text_editor(&self.content).on_edit(Message::Edit);

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };

        let status_bar = row![horizontal_space(Length::Fill), position];

        container(column![input, status_bar].spacing(10)).padding(10).into()
    }

    // Theme provider method
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}