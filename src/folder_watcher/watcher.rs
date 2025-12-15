use log::info;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::broadcast::{self, Receiver, Sender};
use std::path::Path;
// use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};

use crate::db::watched_folders_table::WatchedFolder;
use crate::service::video::folder_change_event_handler::handle_folder_change_event;

#[derive(Debug, Clone)]
pub enum FolderListChangeEvent {
    Added(WatchedFolder),
    Removed(WatchedFolder),
}

/// Async, futures channel based event watching
pub struct FolderWatcher {
    watcher: Arc<Mutex<RecommendedWatcher>>,
    folder_content_update_event_receiver: Arc<Mutex<mpsc::Receiver<Result<Event, notify::Error>>>>,
    folder_list_event_receiver: Option<Receiver<FolderListChangeEvent>>,
}

impl FolderWatcher {
    pub fn new() -> Result<FolderWatcher, notify::Error> {
        let (tx, rx) = mpsc::channel();
        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let watcher = Arc::new(Mutex::new(RecommendedWatcher::new(
            move |res| tx.send(res).unwrap(),
            Config::default(),
        )?));

        Ok(FolderWatcher {
            watcher,
            folder_content_update_event_receiver: Arc::new(Mutex::new(rx)),
            folder_list_event_receiver: None,
        })
    }

    pub async fn async_watch<P: AsRef<Path> + Send + 'static>(&mut self, paths: Vec<P>) -> notify::Result<()> {
        let watcher_clone_ui = self.watcher.clone(); // Clone for UI events task
        let watcher_clone_folders = self.watcher.clone(); // Clone for folders task
        let folder_list_event_receiver = self.folder_list_event_receiver.take();
        let folder_content_update_event_receiver = self.folder_content_update_event_receiver.clone();

        // TODO mode this into a method
        let ui_events_task = tokio::spawn(async move {
            if let Some(mut receiver) = folder_list_event_receiver {
                while let Ok(watched_folder_change_event) = receiver.recv().await {
                    let mut unwrapped_watcher_clone_ui = watcher_clone_ui.lock().unwrap();

                    match watched_folder_change_event {
                        FolderListChangeEvent::Added(watched_folder) => {
                            info!("received event: added folder path to be watched: {}", watched_folder.path);
                            unwrapped_watcher_clone_ui
                                .watch(watched_folder.path.as_ref(), RecursiveMode::Recursive)
                                .unwrap();
                        }
                        FolderListChangeEvent::Removed(watched_folder) => {
                            info!("received event: removed folder path to be watched: {}", watched_folder.path);
                            unwrapped_watcher_clone_ui
                                .unwatch(watched_folder.path.as_ref())
                                .unwrap();
                        }
                    }
                }
            }
        });

        // TODO mode this into a method
        let folders_task = tokio::spawn(async move {
            for path in &paths {
                watcher_clone_folders
                    .lock()
                    .unwrap()
                    .watch(path.as_ref(), RecursiveMode::Recursive)
                    .unwrap();
            }

            while let Ok(res) = folder_content_update_event_receiver.lock().unwrap().recv() {
                info!("in loop");
                match res {
                    Ok(event) => {
                        info!("changed: {:?}", event);
                        handle_folder_change_event(event);
                    },
                    Err(e) => info!("watch error: {:?}", e),
                }
            }
        });

        // Wait for both tasks to complete
        let _ = tokio::try_join!(ui_events_task, folders_task);

        Ok(())
    }

    pub fn create_folders_to_watch_event_receiver(&mut self) -> Sender<FolderListChangeEvent> {
        let (sender, receiver) = broadcast::channel::<FolderListChangeEvent>(100);
        self.folder_list_event_receiver = Some(receiver);
        sender
    }

    // pub async fn watch_ui_events(&self, 
    //     watcher_clone_ui: Arc<Mutex<RecommendedWatcher>>, 
    //     mut folder_list_event_receiver: Option<Receiver<FolderListChangeEvent>>) {
    //     info!("started watch_ui_events");

    //     let ui_events_task = tokio::spawn(async move {
    //         if let Some(mut receiver) = folder_list_event_receiver {
    //             while let Ok(watched_folder_change_event) = receiver.recv().await {
    //                 let mut unwrapped_watcher_clone_ui = watcher_clone_ui.lock().unwrap();

    //                 match watched_folder_change_event {
    //                     FolderListChangeEvent::Added(watched_folder) => {
    //                         info!("received event: added folder path to be watched: {}", watched_folder.path);
    //                         unwrapped_watcher_clone_ui
    //                             .watch(watched_folder.path.as_ref(), RecursiveMode::Recursive)
    //                             .unwrap();
    //                     }
    //                     FolderListChangeEvent::Removed(watched_folder) => {
    //                         info!("received event: removed folder path to be watched: {}", watched_folder.path);
    //                         unwrapped_watcher_clone_ui
    //                             .unwatch(watched_folder.path.as_ref())
    //                             .unwrap();
    //                     }
    //                 }
    //             }
    //         }
    //     });
    // }

    
    // pub async fn watch_folders<P: AsRef<Path>>(&self, paths: Vec<P>) {
    //     info!("started watch_folders");
    //     let iterator = paths.iter();
    //     iterator.for_each(|path| {
    //         self.watcher
    //             .lock()
    //             .unwrap()
    //             .watch(path.as_ref(), RecursiveMode::Recursive)
    //             .unwrap();
    //     });

    //     while let Ok(res) = self.folder_content_update_event_receiver.lock().unwrap().recv() {
    //         info!("in loop");
    //         match res {
    //             Ok(event) => info!("changed: {:?}", event),
    //             Err(e) => info!("watch error: {:?}", e),
    //         }
    //     }
    // }
}
