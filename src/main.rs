use iced::{ executor, Subscription };
use iced::widget::{
    button,
    column,
    container,
    horizontal_space,
    row,
    text,
    text_editor,
    tooltip,
    pick_list,
};
use iced::keyboard;
use iced::{ Font, Command, Application, Element, Length, Settings, Theme };
use iced::theme;
use iced::highlighter::{ self, Highlighter };

use std::io;
use std::path::{ Path, PathBuf };
use std::sync::Arc;

// Run returns a result for errors and etc
fn main() -> iced::Result {
    Editor::run(Settings {
        default_font: Font::MONOSPACE,
        fonts: vec![include_bytes!("../fonts/editor-icons.ttf").as_slice().into()],
        ..Settings::default()
    })
}

struct EditHistory {
    history: Vec<String>,
    current_index: usize,
}

impl EditHistory {
    fn new(initial_text: String) -> Self {
        Self {
            history: vec![initial_text],
            current_index: 0,
        }
    }

    fn add_edit(&mut self, text: String) {
        // Remove qualquer edição futura se houver
        if self.current_index < self.history.len() - 1 {
            self.history.truncate(self.current_index + 1);
        }
        self.history.push(text);
        self.current_index += 1;
    }

    fn undo(&mut self) -> Option<String> {
        if self.current_index > 0 {
            self.current_index -= 1;
            Some(self.history[self.current_index].clone())
        } else {
            None
        }
    }

    fn is_clean(&self) -> bool {
        self.current_index == 0
    }
}

struct Editor {
    path: Option<PathBuf>,
    content: text_editor::Content,
    error: Option<Error>,
    theme: highlighter::Theme,
    is_dirty: bool,
    block_edit: bool,
    edit_history: EditHistory,
}

// Messages should generally to be clone because they represent pure events
#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    New,
    Open,
    FileOpened(Result<(PathBuf, Arc<String>), Error>),
    Save,
    FileSaved(Result<PathBuf, Error>),
    ThemeSelected(highlighter::Theme),
    BlockEdit,
    UnblockEdit,
    Undo,
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
                theme: highlighter::Theme::Base16Mocha,
                is_dirty: true,
                block_edit: false,
                edit_history: EditHistory::new("".to_string()),
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
                if self.block_edit {
                    return Command::none();
                }

                self.is_dirty = self.is_dirty || action.is_edit();
                self.error = None;
                self.content.edit(action);

                self.edit_history.add_edit(self.content.text().to_string());

                Command::none()
            }
            Message::BlockEdit => {
                self.block_edit = true;

                Command::none()
            }
            Message::UnblockEdit => {
                self.block_edit = false;

                Command::none()
            }
            Message::New => {
                self.path = None;
                self.content = text_editor::Content::new();
                self.is_dirty = true;

                Command::none()
            }
            Message::Open => { Command::perform(pick_file(), Message::FileOpened) }
            Message::FileOpened(Ok((path, content))) => {
                self.path = Some(path);
                self.content = text_editor::Content::with(&content);

                self.edit_history = EditHistory::new(content.as_ref().clone());
                self.is_dirty = false;

                Command::none()
            }
            Message::FileOpened(Err(error)) => {
                self.error = Some(error);

                Command::none()
            }
            Message::Save => {
                let text = self.content.text();

                self.edit_history = EditHistory::new(text.clone());

                Command::perform(save_file(self.path.clone(), text), Message::FileSaved)
            }
            Message::FileSaved(Ok(path)) => {
                self.path = Some(path);
                self.is_dirty = false;

                Command::none()
            }
            Message::FileSaved(Err(error)) => {
                self.error = Some(error);

                Command::none()
            }
            Message::ThemeSelected(theme) => {
                self.theme = theme;

                Command::none()
            }
            Message::Undo => {
                if self.is_dirty {
                    if let Some(undo_text) = self.edit_history.undo() {
                        self.content = text_editor::Content::with(&undo_text);
                        if self.edit_history.is_clean() {
                            self.is_dirty = false;
                        }
                    }
                }

                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Define as duas handlers
        let key_press_handler = keyboard::on_key_press(|key_code, modifiers| {
            if modifiers.command() {
                if key_code == keyboard::KeyCode::Z {
                    return Some(Message::Undo);
                }

                if key_code == keyboard::KeyCode::S {
                    return Some(Message::Save);
                }

                return Some(Message::BlockEdit);
            }

            None
        });

        let key_release_handler = keyboard::on_key_release(|_key_code, modifiers| {
            if !modifiers.command() {
                return Some(Message::UnblockEdit);
            }

            None
        });

        // Retorna uma batch das duas subscriptions
        Subscription::batch(vec![key_press_handler, key_release_handler])
    }

    // Lógica que produz os widgets da interface
    // Logic that produces the interface widgets
    fn view(&self) -> Element<'_, Message> {
        let controls = row![
            action(new_icon(), "New File", Some(Message::New)),
            action(open_icon(), "Open File", Some(Message::Open)),
            action(save_icon(), "Save File", self.is_dirty.then_some(Message::Save)),
            horizontal_space(Length::Fill),
            pick_list(highlighter::Theme::ALL, Some(self.theme), Message::ThemeSelected)
        ].spacing(10);

        let input = text_editor(&self.content)
            .on_edit(Message::Edit)
            .highlight::<Highlighter>(
                highlighter::Settings {
                    theme: self.theme,
                    extension: self.path
                        .as_ref()
                        .and_then(|path| path.extension()?.to_str())
                        .unwrap_or("rs")
                        .to_string(),
                },
                |highlight, _theme| highlight.to_format()
            );

        let status_bar = {
            let status = if let Some(Error::IOFailed(error)) = self.error.as_ref() {
                text(error.to_string())
            } else {
                match self.path.as_deref().and_then(Path::to_str) {
                    Some(path) => text(path).size(14),
                    None => text("New file"),
                }
            };

            let position = {
                let (line, column) = self.content.cursor_position();

                text(format!("{}:{}", line + 1, column + 1))
            };

            row![status, horizontal_space(Length::Fill), position]
        };

        container(column![controls, input, status_bar].spacing(10)).padding(10).into()
    }

    // Theme provider method
    fn theme(&self) -> Theme {
        if self.theme.is_dark() { Theme::Dark } else { Theme::Light }
    }
}

fn action<'a>(
    content: Element<'a, Message>,
    label: &str,
    on_press: Option<Message>
) -> Element<'a, Message> {
    let is_disabled = on_press.is_none();

    tooltip(
        button(container(content).width(30).center_x())
            .on_press_maybe(on_press)
            .padding([5, 10])
            .style(if is_disabled { theme::Button::Secondary } else { theme::Button::Primary }),
        label,
        tooltip::Position::FollowCursor
    )
        .style(theme::Container::Box)
        .into()
}

fn new_icon<'a>() -> Element<'a, Message> {
    icon('\u{E800}')
}

fn open_icon<'a>() -> Element<'a, Message> {
    icon('\u{F115}')
}

fn save_icon<'a>() -> Element<'a, Message> {
    icon('\u{E801}')
}

fn icon<'a>(codepoint: char) -> Element<'a, Message> {
    const ICON_FONT: Font = Font::with_name("editor-icons");

    text(codepoint).font(ICON_FONT).into()
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
        .map_err(|error: io::Error| error.kind())
        .map_err(Error::IOFailed)?;

    Ok((path, contents))
}

async fn save_file(path: Option<PathBuf>, text: String) -> Result<PathBuf, Error> {
    let path = if let Some(path) = path {
        path
    } else {
        rfd::AsyncFileDialog
            ::new()
            .set_title("Choose a file name...")
            .save_file().await
            .ok_or(Error::DialogClosed)
            .map(|handle| handle.path().to_owned())?
    };

    tokio::fs::write(&path, text).await.map_err(|error| Error::IOFailed(error.kind()))?;

    Ok(path)
}

#[derive(Debug, Clone)]
enum Error {
    DialogClosed,
    IOFailed(io::ErrorKind),
}
