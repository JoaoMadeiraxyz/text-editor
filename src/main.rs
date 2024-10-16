use iced::executor;
use iced::widget::{ button, column, container, horizontal_space, row, text, text_editor };
use iced::{ Command, Application, Element, Length, Settings, Theme };

use std::io;
use std::path::{ Path, PathBuf };
use std::sync::Arc;

// Run returns a result for errors and etc
fn main() -> iced::Result {
    Editor::run(Settings::default())
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
}

// Messages should generally to be clone because they represent pure events
#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
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
                path: None,
                content: text_editor::Content::new(),
                error: None,
            },
            Command::perform(load_file(default_file()), Message::FileOpened),
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
                Command::none()
            }
            Message::Open => { Command::perform(pick_file(), Message::FileOpened) }
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);
                Command::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);
                Command::none()
            }
        }
    }

    // Lógica que produz os widgets da interface
    // Logic that produces the interface widgets
    fn view(&self) -> Element<'_, Message> {
        let controls = row![button("Open").on_press(Message::Open)];
        let input = text_editor(&self.content).on_edit(Message::Edit);
        let file_path = match self.path.as_deref().and_then(Path::to_str) {
            Some(path) => text(path).size(14),
            None => text(""),
        };

        let position = {
            let (line, column) = self.content.cursor_position();

            text(format!("{}:{}", line + 1, column + 1))
        };

        let status_bar = row![file_path, horizontal_space(Length::Fill), position];

        container(column![controls, input, status_bar].spacing(10)).padding(10).into()
    }

    // Theme provider method
    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

fn default_file() -> PathBuf {
    PathBuf::from(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")))
}

async fn pick_file() -> Result<(PathBuf, Arc<String>), Error> {
    let handle = rfd::AsyncFileDialog
        ::new()
        .set_title("Choose a text file...")
        .pick_file().await
        .ok_or(Error::DialogClosed)?;

    load_file(handle.path().to_owned()).await
}

async fn load_file(path: PathBuf) -> Result<(PathBuf, Arc<String>), Error> {
    let contents = tokio::fs
        ::read_to_string(&path).await
        .map(Arc::new)
        .map_err(|error| error.kind())
        .map_err(Error::IO)?;

    Ok((path, contents))
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IO(io::ErrorKind),
}
