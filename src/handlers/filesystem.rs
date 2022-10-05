use crate::{create_entry, delete_entry, update_entry};
use notify::{
    event::{Event, EventKind, ModifyKind},
    Config, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio::sync::broadcast::Receiver;

use crate::CONFIG;

pub fn new_file(file_path: PathBuf, file_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new(&CONFIG.get().unwrap().storage.storage_dir).join(file_path);

    fs::create_dir_all(file_path.parent().unwrap())?;
    fs::write(&file_path, file_data)?;

    Ok(())
}

pub fn normalize_path(file_path: PathBuf) -> PathBuf {
    Path::new(&CONFIG.get().unwrap().storage.storage_dir).join(file_path)
}

pub fn strip_storage_dir(file_path: PathBuf) -> PathBuf {
    Path::new("/").join(
        file_path
            .strip_prefix(&CONFIG.get().unwrap().storage.storage_dir)
            .unwrap(),
    )
}

pub async fn watch_fs(mut shutdown_receiver: Receiver<()>) {
    let (sender, receiver) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(sender, Config::default()).unwrap();

    watcher
        .watch(
            &PathBuf::from(&CONFIG.get().unwrap().storage.storage_dir),
            RecursiveMode::Recursive,
        )
        .unwrap();

    loop {
        if shutdown_receiver.try_recv().is_ok() {
            break;
        }

        let try_recv = receiver.try_recv();

        if let Ok(res) = try_recv {
            match res {
                Ok(event) => {
                    sync_db(event).await;
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }
}

pub async fn sync_db(event: Event) {
    println!("Syncing DB - {:?}", event);
    match event.kind {
        EventKind::Create(_) => {
            let create_path = event.paths.into_iter().next().unwrap();
            create_entry(create_path).await.unwrap();
        }
        EventKind::Remove(_) => {
            let delete_path = event.paths.into_iter().next().unwrap();
            delete_entry(delete_path).await.unwrap();
        }
        EventKind::Modify(ModifyKind::Name(_)) => {
            let mut paths_iter = event.paths.into_iter();
            let old_path = paths_iter.next().unwrap();
            let new_path = paths_iter.next().unwrap();
            update_entry(old_path, new_path).await.unwrap();
        }
        _ => {}
    }
}
