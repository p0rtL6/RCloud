use sea_orm::{
    prelude::*, sea_query::query::OnConflict, ActiveValue::Set, ConnectionTrait, DbBackend,
    EntityTrait, Schema,
};
use std::path::{Component, PathBuf};

use crate::{
    dirs_table::prelude::*, files_table::prelude::*, Dirs, Files, CONFIG, DATABASE_CONNECTION,
};

pub async fn create_schema() {
    let schema = Schema::new(DbBackend::Sqlite);
    let files_table = schema.create_table_from_entity(Files);
    let dirs_table = schema.create_table_from_entity(Dirs);

    let db = DATABASE_CONNECTION.get().unwrap();

    db.execute(db.get_database_backend().build(&files_table))
        .await
        .unwrap();

    db.execute(db.get_database_backend().build(&dirs_table))
        .await
        .unwrap();
}

pub async fn create_root_dir() {
    let root_dir = DirsActiveModel {
        dir_name: Set("".to_owned()),
        full_path: Set("/".to_owned()),
        parent_dir_id: Set(1),
        ..Default::default()
    };
    Dirs::insert(root_dir)
        .exec(DATABASE_CONNECTION.get().unwrap())
        .await
        .unwrap();
}

pub fn bytes_to_kb(bytes: u64) -> i32 {
    (bytes / 1000) as i32
}

pub async fn create_entry(path: PathBuf) -> Result<(), ()> {
    let db = DATABASE_CONNECTION.get().unwrap();

    let path = path
        .strip_prefix(&CONFIG.get().unwrap().storage.path_prefix)
        .unwrap()
        .to_path_buf();
    let prefixed_path = PathBuf::from(&CONFIG.get().unwrap().storage.storage_dir).join(&path);
    let mut full_path: PathBuf = PathBuf::from("/");

    let mut parent_dir_id: i32 = 1;

    let mut path_components = path.components().peekable();
    while let Some(component) = path_components.next() {
        if let Component::Normal(component_name) = component {
            full_path.push(component_name);
            if prefixed_path.is_file() && path_components.peek().is_none() {
                let file = FilesActiveModel {
                    file_name: Set(component_name.to_str().unwrap().to_owned()),
                    full_path: Set(full_path.to_str().unwrap().to_owned()),
                    file_size: Set(bytes_to_kb(prefixed_path.metadata().unwrap().len())),
                    parent_dir_id: Set(parent_dir_id),
                    ..Default::default()
                };
                Files::insert(file)
                    .on_conflict(
                        OnConflict::column(FilesColumn::FullPath)
                            .update_columns([FilesColumn::FileSize])
                            .to_owned(),
                    )
                    .exec(db)
                    .await
                    .unwrap();
            } else {
                let dir_lookup = Dirs::find()
                    .filter(DirsColumn::FullPath.like(full_path.to_str().unwrap()))
                    .one(db)
                    .await
                    .unwrap();
                match dir_lookup {
                    Some(current_dir) => {
                        parent_dir_id = current_dir.dir_id;
                    }
                    None => {
                        let dir = DirsActiveModel {
                            dir_name: Set(component_name.to_str().unwrap().to_owned()),
                            full_path: Set(full_path.to_str().unwrap().to_owned()),
                            parent_dir_id: Set(parent_dir_id),
                            ..Default::default()
                        };
                        let dir_result = Dirs::insert(dir).exec(db).await.unwrap();
                        parent_dir_id = dir_result.last_insert_id;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn create_entries(paths: Vec<PathBuf>) -> Result<(), ()> {
    for path in paths {
        create_entry(path).await?;
    }
    Ok(())
}

pub async fn delete_entry(path: PathBuf) -> Result<(), ()> {
    let db = DATABASE_CONNECTION.get().unwrap();

    let path = PathBuf::from("/").join(
        path.strip_prefix(&CONFIG.get().unwrap().storage.path_prefix)
            .unwrap(),
    );

    let dir_lookup = Dirs::find()
        .filter(DirsColumn::FullPath.like(path.to_str().unwrap()))
        .one(db)
        .await
        .unwrap();

    match dir_lookup {
        Some(dir) => {
            dir.delete(db).await.unwrap();
        }
        None => {
            let file_lookup = Files::find()
                .filter(FilesColumn::FullPath.like(path.to_str().unwrap()))
                .one(db)
                .await
                .unwrap();

            match file_lookup {
                Some(file) => {
                    file.delete(db).await.unwrap();
                }
                None => {}
            }
        }
    }
    Ok(())
}

pub async fn delete_entries(paths: Vec<PathBuf>) -> Result<(), ()> {
    for path in paths {
        delete_entry(path).await?;
    }
    Ok(())
}

pub async fn update_entry(old_path: PathBuf, new_path: PathBuf) -> Result<(), ()> {
    let db = DATABASE_CONNECTION.get().unwrap();

    let old_path = PathBuf::from("/").join(
        old_path
            .strip_prefix(&CONFIG.get().unwrap().storage.path_prefix)
            .unwrap()
    );

    let new_path = PathBuf::from("/").join(
        new_path
            .strip_prefix(&CONFIG.get().unwrap().storage.path_prefix)
            .unwrap()
    );

    let dir_lookup = Dirs::find()
        .filter(DirsColumn::FullPath.like(old_path.to_str().unwrap()))
        .one(db)
        .await
        .unwrap();

    match dir_lookup {
        Some(dir) => {
            let mut dir: DirsActiveModel = dir.into();
            dir.full_path = Set(new_path.to_str().unwrap().to_owned());
            dir.dir_name = Set(new_path.file_name().unwrap().to_str().unwrap().to_owned());
            dir.update(db).await.unwrap();
        }
        None => {
            let file_lookup = Files::find()
                .filter(FilesColumn::FullPath.like(old_path.to_str().unwrap()))
                .one(db)
                .await
                .unwrap();

            match file_lookup {
                Some(file) => {
                    let mut file: FilesActiveModel = file.into();
                    file.full_path = Set(new_path.to_str().unwrap().to_owned());
                    file.file_name =
                        Set(new_path.file_name().unwrap().to_str().unwrap().to_owned());
                    file.update(db).await.unwrap();
                }
                None => {}
            }
        }
    }
    Ok(())
}

pub async fn update_entries(old_paths: Vec<PathBuf>, new_paths: Vec<PathBuf>) -> Result<(), ()> {
    let paths = old_paths.into_iter().zip(new_paths.into_iter());
    for (old_path, new_path) in paths {
        update_entry(old_path, new_path).await?;
    }
    Ok(())
}

pub async fn get_dir_contents(path: PathBuf) -> (Vec<FilesModel>, Vec<DirsModel>) {
    let db = DATABASE_CONNECTION.get().unwrap();

    let path = PathBuf::from("/").join(
        path.strip_prefix(&CONFIG.get().unwrap().storage.path_prefix)
            .unwrap()
    );

    let dir_lookup = Dirs::find()
        .filter(DirsColumn::FullPath.like(path.to_str().unwrap()))
        .one(db)
        .await
        .unwrap();

    match dir_lookup {
        Some(dir) => {
            let dir_id = dir.dir_id.to_string();
            let child_files = Files::find()
                .filter(FilesColumn::ParentDirId.like(&dir_id))
                .all(db)
                .await
                .unwrap();
            let child_folders = Dirs::find()
                .filter(DirsColumn::ParentDirId.like(&dir_id))
                .all(db)
                .await
                .unwrap();
            (child_files, child_folders)
        }
        None => (vec![], vec![]),
    }
}

pub async fn query() {
    let files_response = Files::find()
        .all(DATABASE_CONNECTION.get().unwrap())
        .await
        .unwrap();
    let dirs_response = Dirs::find()
        .all(DATABASE_CONNECTION.get().unwrap())
        .await
        .unwrap();
    dbg!(&files_response);
    dbg!(&dirs_response);
}
