use std::path::Path;

use actix_files;
use actix_multipart::Multipart;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    dev::{Server, ServiceRequest},
    get, post, web, App, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_httpauth::{
    extractors::{
        basic::{BasicAuth, Config as HttpConfig},
        AuthenticationError,
    },
    middleware::HttpAuthentication,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::{new_file, normalize_path, CONFIG};

#[derive(Serialize, Deserialize, Debug)]
pub struct Auth {
    username: String,
    password: String,
}

#[get("/api/files/{file_path:.*}")]
pub async fn get_file(req: HttpRequest, session: Session) -> HttpResponse {
    if session.get::<i32>("Authenticated").unwrap().is_none() {
        session.insert("Authenticated", 1).unwrap();
    }
    let path = normalize_path(req.match_info().query("file_path").parse().unwrap());
    if path.is_file() {
        let file_contents = std::fs::read(path).unwrap();
        HttpResponse::Ok().body(file_contents)
    } else if path.is_dir() {
        HttpResponse::Ok().body(serde_json::to_string(&crate::ls_dir(path).await).unwrap())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/upload/{directory:.*}")]
async fn upload(
    mut payload: Multipart,
    path: web::Path<(String,)>,
) -> Result<HttpResponse, actix_multipart::MultipartError> {
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let file_path: String = path.0.to_owned();
        field.next().await.into_iter().for_each(|file| {
            new_file(
                Path::new(&file_path).join(field.content_disposition().get_filename().unwrap()),
                file.unwrap().to_vec(),
            )
            .unwrap();
        });
    }

    Ok(HttpResponse::Ok().into())
}

pub fn start_server() -> Server {
    let secret_key = Key::generate();
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key.clone(),
            ))
            .wrap(HttpAuthentication::basic(validator))
            .service(get_file)
            // .service(upload)
            .service(actix_files::Files::new("/raw/files", "./files").show_files_listing())
    })
    .bind(("127.0.0.1", 8888))
    .unwrap()
    .run()
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, actix_web::Error> {
    let config = req.app_data::<HttpConfig>().cloned().unwrap_or_default();

    let username: &str = credentials.user_id();
    let password: &str = credentials
        .password()
        .unwrap_or(&std::borrow::Cow::Borrowed(""));

    if username == CONFIG.get().unwrap().authentication.username
        && password == CONFIG.get().unwrap().authentication.password
    {
        return Ok(req);
    }

    Err(AuthenticationError::from(config).into())
}
