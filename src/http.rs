use crate::json::*;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use parking_lot::RwLock;
use serde_derive::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::mpsc::{self, UnboundedSender};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum RequestType {
    ListThemes,
    SetTheme(String),
}

impl RequestType {
    pub fn serialize(&self, uuid: String) -> Result<Vec<u8>, serde_json::Error> {
        match self {
            RequestType::ListThemes => {
                let req = ExtensionRequest {
                    uuid,
                    command: "list_themes".to_string(),
                    theme_id: None,
                };
                Ok(serde_json::to_string(&req)?.as_bytes().to_vec())
            }
            RequestType::SetTheme(theme_id) => {
                let req = ExtensionRequest {
                    uuid,
                    command: "set_theme".to_string(),
                    theme_id: Some(theme_id.to_string()),
                };
                Ok(serde_json::to_string(&req)?.as_bytes().to_vec())
            }
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    // When a request is made, a channel is added here where the response should be sent from the read thread
    pub incoming_receivers: Arc<RwLock<HashMap<String, UnboundedSender<ExtensionResponse>>>>,
    // We don't need a HashMap here since it's a queue
    pub outgoing: Arc<RwLock<VecDeque<(Uuid, RequestType)>>>,
}

fn push_request(req_id: Uuid, data: web::Data<AppState>, request: RequestType) {
    let mut requests = data.outgoing.write();
    requests.push_back((req_id, request));
    drop(requests);
}

async fn push_request_and_wait(
    req_id: Uuid,
    data: web::Data<AppState>,
    request: RequestType,
) -> Option<ExtensionResponse> {
    let mut channels = data.incoming_receivers.write();
    let (tx, mut rx) = mpsc::unbounded_channel::<ExtensionResponse>();
    channels.insert(req_id.to_string(), tx);
    drop(channels);
    push_request(req_id, data, request);
    rx.recv().await
}

#[get("/get_themes")]
async fn get_themes(data: web::Data<AppState>) -> impl Responder {
    let req_id = uuid::Uuid::new_v4();
    match push_request_and_wait(req_id, data, RequestType::ListThemes).await {
        Some(extresp) => HttpResponse::Ok().body(serde_json::to_string(&extresp.themes).unwrap()),
        None => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Deserialize)]
struct SetThemeInfo {
    id: String,
}

#[get("/set_theme")]
async fn set_theme(info: web::Query<SetThemeInfo>, data: web::Data<AppState>) -> impl Responder {
    let req_id = uuid::Uuid::new_v4();
    push_request(req_id, data, RequestType::SetTheme(info.id.clone()));
    HttpResponse::Ok().finish()
}

pub fn http_server(state: AppState) -> std::io::Result<()> {
    let state = web::Data::new(state);
    let server = HttpServer::new(move || {
        App::new()
            .service(get_themes)
            .service(set_theme)
            .app_data(state.clone())
    })
    .bind(("127.0.0.1", 8080))?;
    tokio::spawn(server.run());
    Ok(())
}
