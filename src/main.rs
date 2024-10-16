use iced::executor;
use iced::widget::{ column, container, horizontal_space, row, text, text_editor };
use iced::{ Command, Application, Element, Length, Settings, Theme };

use std::io;
use std::path::Path;
use std::sync::Arc;

// Run returns a result for errors and etc
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    content: text_editor::Content,
    error: Option<io::ErrorKind>,
}

// Messages should generally to be clone because they represent pure events
#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    FileOpened(Result<Arc<String>, io::ErrorKind>),
}

impl Application for Editor {
    // Uma mensagem é um evento ou interações do usuário que a aplicação pode lidar ou reagir, ex: clique de um botão
    // A message is an event or user interaction that the application can handle or interact with, ex: a click in a button
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    // Dita o estado da aplicação ao iniciar
    // Application initial state
    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(
                load_file(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR"))),
                Message::FileOpened
            ),
        )
    }

    // Título da aplicação
    // Application title
    fn title(&self) -> String {
        String::from("A cool editor!")
    }

    // Lógica para lidar com as mensagens;
    // Logic that handles messages
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.content.edit(action);
            }
            Message::FileOpened(Ok(content)) => {
                self.content = text_editor::Content::with(&content);
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
            }
        }

        Command::none()
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

async fn load_file(path: impl AsRef<Path>) -> Result<Arc<String>, io::ErrorKind> {
    tokio::fs
        ::read_to_string(path).await
        .map(Arc::new)
        .map_err(|error| error.kind())
}
