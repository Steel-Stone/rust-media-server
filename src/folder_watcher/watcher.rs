use log::info;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};

use crate::db::watched_folders_table::WatchedFolder;

pub enum FolderListChangeEvent {
    Added(WatchedFolder),
    Removed(WatchedFolder),
}

/// Async, futures channel based event watching
pub struct FolderWatcher {
    watcher: Arc<Mutex<RecommendedWatcher>>,
    folder_content_update_event_receiver: Arc<Mutex<Receiver<Result<Event, notify::Error>>>>,
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

        // Spawn `watch_ui_events` as a separate task
        let ui_events_task = tokio::spawn(async move {
            if let Some(receiver) = folder_list_event_receiver {
                while let Ok(watched_folder_change_event) = receiver.recv() {
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

                    // info!("received event: folder path: {}", watched_folder.path);
                    // watcher_clone_ui
                    //     .lock()
                    //     .unwrap()
                    //     .watch(watched_folder.path.as_ref(), RecursiveMode::Recursive)
                    //     .unwrap();
                }
            }
        });

        // Spawn `watch_folders` as a separate task
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
                    Ok(event) => info!("changed: {:?}", event),
                    Err(e) => info!("watch error: {:?}", e),
                }
            }
        });

        // Wait for both tasks to complete
        let _ = tokio::try_join!(ui_events_task, folders_task);

        Ok(())
    }

    pub fn create_folders_to_watch_event_receiver(&mut self) -> mpsc::Sender<FolderListChangeEvent> {
        let (sender, receiver) = mpsc::channel();
        self.folder_list_event_receiver = Some(receiver);
        sender
    }

    // pub async fn watch_ui_events(&self) {
    //     info!("started watch_ui_events");
    //     // TODO give a reason whny you have to set folder_list_event_receiver if None
    //     while let Some(watched_folder) = self.folder_list_event_receiver.as_ref().unwrap().iter().next() {
    //         info!("received event: folder path: {}", watched_folder.path);
    //         self.watcher
    //         .lock()
    //         .unwrap()
    //         .watch(watched_folder.path.as_ref(), RecursiveMode::Recursive)
    //         .unwrap();
    //     }
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
