use crate::{create_entry, delete_entry, get_dir_contents, update_entry};
use notify::{DebouncedEvent, DebouncedEvent::*, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::Duration,
};

use crate::CONFIG;

pub fn new_file(file_path: PathBuf, file_data: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = Path::new(&CONFIG.get().unwrap().storage.storage_dir).join(file_path);

    fs::create_dir_all(file_path.parent().unwrap())?;
    fs::write(&file_path, file_data)?;

    Ok(())
}

pub async fn watch_fs() {
    let (sender, receiver) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(sender, Duration::from_secs(2)).unwrap();
    watcher
        .watch(
            &CONFIG.get().unwrap().storage.storage_dir,
            RecursiveMode::Recursive,
        )
        .unwrap();

    loop {
        match receiver.recv() {
            Ok(event) => sync_db(event).await,
            Err(e) => println!("Watch Error: {:?}", e),
        }
    }
}

pub async fn sync_db(event: DebouncedEvent) {
    match event {
        Write(path) => {
            create_entry(path).await.unwrap();
        }
        Create(path) => {
            create_entry(path).await.unwrap();
        }
        Remove(path) => {
            let dir_contents =
                get_dir_contents(PathBuf::from("/home/p0rtl/Sync/Code/cloud_2/files/dir1")).await;
            println!("{:?}", dir_contents);
            delete_entry(path).await.unwrap();
        }
        Rename(old_path, new_path) => {
            update_entry(old_path, new_path).await.unwrap();
        }
        _ => {}
    }
}
