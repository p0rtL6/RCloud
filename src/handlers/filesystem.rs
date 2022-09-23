use crate::{create_entry, delete_entry, update_entry};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Config, event::{Event, EventKind, ModifyKind}};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
};

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

pub async fn watch_fs() {
    let (sender, reciever) = channel();

    let mut watcher = RecommendedWatcher::new(sender, Config::default()).unwrap();
    
    watcher
        .watch(
            &PathBuf::from(&CONFIG.get().unwrap().storage.storage_dir),
            RecursiveMode::Recursive,
        )
        .unwrap();

    for res in reciever {
        match res {
            Ok(event) => {
                sync_db(event).await;
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}

pub async fn sync_db(event: Event) {
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
