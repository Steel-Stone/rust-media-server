use iced::Color;
use iced::widget::Container;
use log::info;
use std::{io, sync::Arc};
use std::sync::mpsc::Sender;
use iced::{
    executor,
    widget::{button, column, container, horizontal_space, row, text, Column},
    Application, Command, Element, Error, Length, Sandbox, Settings, Theme,
};
use notify::Event;
use rusqlite::Error as SqlErr;
use strum_macros::Display;

use crate::db::watched_folders_table::{self, WatchedFolder, WatchedFoldersDb};
use crate::folder_watcher::watcher::{self, FolderListChangeEvent};

pub struct Flags {
    pub watched_folders_db: WatchedFoldersDb,
    pub folder_watcher_notifier: Sender<FolderListChangeEvent>
}

struct Page {
    watched_folders_db: WatchedFoldersDb,
    watched_folders: Vec<WatchedFolder>,
    counter: u8,
    path: String,
    error: Option<MyError>,
    folder_watch_notifier: Sender<FolderListChangeEvent>
}

#[derive(Debug, Clone)]
enum Message {
    Click,
    OpenFilePicker,
    DeleteFilePicker(String),
    SetPath(Result<String, MyError>),
}

pub fn start(flags: Flags) -> Result<(), Error> {
    Page::run(Settings {
        flags,
        id: None,
        window: Default::default(),
        default_font: Default::default(),
        default_text_size: iced::Pixels(16.0),
        antialiasing: false,
        // exit_on_close_request: true,
        fonts: Default::default(),
    })
}

impl Page {
    fn refresh_watched_folder_list_ui(&mut self) {
        self.watched_folders = self.watched_folders_db.list().unwrap();
    }

    fn notify_folder_watcher_addition(&self, watched_folder: WatchedFolder) {
        info!("Sending event: added folder path : {}", watched_folder.path);
        self.folder_watch_notifier.send(FolderListChangeEvent::Added(watched_folder)).unwrap();
    }

    fn notify_folder_watcher_removal(&self, watched_folder: WatchedFolder) {
        info!("Sending event: removed folder path : {}", watched_folder.path);
        self.folder_watch_notifier.send(FolderListChangeEvent::Removed(watched_folder)).unwrap();
    }
}

impl Application for Page {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = Flags;

    fn new(flags: Self::Flags) -> (Self, Command<Message>) {
        (
            Self {
                watched_folders: flags.watched_folders_db.list().unwrap(),
                watched_folders_db: flags.watched_folders_db,
                counter: 0,
                path: "".to_string(),
                error: Option::None,
                folder_watch_notifier: flags.folder_watcher_notifier
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("cool ass gui")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Click => {
                self.counter = self.counter + 1;
                Command::none()
            }
            Message::OpenFilePicker => Command::perform(pick_file(), Message::SetPath),
            Message::DeleteFilePicker(path) => {
                let folder = self.watched_folders_db.get(&(path)).unwrap();
                self.watched_folders_db.delete(&(path)); //TODO deal with this error
                self.refresh_watched_folder_list_ui();
                // self.notify_folder_watcher_removal(folder);
                Command::none()
            }
            Message::SetPath(res) => {
                match res {
                    Ok(path) => {
                        let resultFolder = self.watched_folders_db.create(&(path)).unwrap();
                        info!("before refrsh : {}", resultFolder.path);
                        self.refresh_watched_folder_list_ui();
                        info!("beforesent event: folder path : {}",resultFolder.path);
                        self.notify_folder_watcher_addition(resultFolder);
                        self.path = path;
                    }
                    Err(err) => {
                        self.error = Some(err);
                    }
                }
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let mut watched_folders: Column<'_, Message, _> = Column::<'_, Message>::new();
        watched_folders = self
            .watched_folders
            .iter()
            .fold(watched_folders, |acc, watched_folder| {
                acc.push(row![
                    text(format!("{}", watched_folder.path)).width(400),
                    button("-").on_press(Message::DeleteFilePicker(watched_folder.path.clone()))
                ])
            });

        let top_row = row![
            text("Hello, iced!"),
            // horizontal_space(Length::Fixed(50.0)),
            text(format!("{}", self.path)),
            button("+").on_press(Message::OpenFilePicker),
            text(format!(
                "err {}",
                match &self.error {
                    Some(err) => format!("{}", err),
                    None => "none".to_string(),
                }
            ))
        ];

        let row_with_background = Container::new(top_row)
            // .width(Length::Fill)
            // .padding(8)
            .style(|_theme: &Theme| container::Appearance {
                background: Some(Color::from_rgb(0.2, 0.4, 0.8).into()),
                ..Default::default()
            });

        let content = row![
            watched_folders
        ];

        container(column![row_with_background, content, text(format!("{}", self.counter))].padding(10)).into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

async fn pick_file() -> Result<String, MyError> {
    let path = rfd::AsyncFileDialog::new()
        .set_title("choose a folder")
        .pick_folder()
        .await
        .ok_or(MyError::DialogClossed)?;
    Ok(path.path().to_string_lossy().to_string())
}

#[derive(Debug, Clone, Display)]
enum MyError {
    DialogClossed,
}
