use std::{thread, time::Duration};

use notify::{Event, EventKind, event::{CreateKind, ModifyKind}};

use crate::service::video::ffmpeg;

pub fn handle_folder_change_event(event: Event) {
    match event.kind {
        EventKind::Create(CreateKind::File) => {
            println!("File created: {:?}", event.paths);
            println!("directory ? {}", event.paths[0].parent().unwrap().to_str().unwrap().to_string());
            println!("file ? {}", event.paths[0].file_name().unwrap().to_str().unwrap().to_string());
            ffmpeg::transcode( event.paths[0].parent().unwrap().to_str().unwrap().to_string(),
                             event.paths[0].file_name().unwrap().to_str().unwrap().to_string());
        }
        EventKind::Any => todo!(),
        EventKind::Access(access_kind) => todo!(),
        EventKind::Modify(ModifyKind::Name(_)) => {
            println!("File moved/renamed: {:?}", event.paths);
            ffmpeg::transcode(event.paths[0].parent().unwrap().to_str().unwrap().to_string(),
                             event.paths[0].file_name().unwrap().to_str().unwrap().to_string());
        },
        EventKind::Remove(remove_kind) => todo!(),
        EventKind::Other => todo!(),
        _ => {}
    }
}