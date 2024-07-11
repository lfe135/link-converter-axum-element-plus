use std::{io::SeekFrom, sync::Arc};

use axum::{extract::{Path, State}, http::{header::{CONTENT_TYPE, LOCATION}, Method, StatusCode}, response::{ErrorResponse, Redirect, Result}, routing::{get, post}, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{fs::{File, OpenOptions}, io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt}, sync::Mutex};
use tower_http::{cors::{Any, CorsLayer}, services::ServeDir};
use uuid::Uuid;

struct AppState {
    mutex: Mutex<File>
}

#[tokio::main]
async fn main() {
    let lis = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let file = OpenOptions::new().read(true).write(true).create(true).open("record.json").await.unwrap();
    let mutex = Mutex::new(file);
    let state = Arc::new(AppState { mutex });
    let router = axum::Router::new()
        .route("/convert", post(convert))
        .route("/list", get(list))
        .route("/:id", get(redirect))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(Any).allow_methods([Method::GET, Method::POST]).allow_headers([CONTENT_TYPE, LOCATION]))
        .fallback_service(ServeDir::new("dist"));
    axum::serve(lis, router).await.unwrap();
}
#[derive(Deserialize, Serialize)]
struct ConvertRequest {
    url: String,
}

#[derive(Deserialize, Serialize)]
struct ConvertResponse {
    url: String,
}

#[derive(Deserialize, Serialize,Clone, Debug)]
struct Record {
    src: String,
    dst: String,
    count: i32
}

async fn convert(State(state): State<Arc<AppState>>,Json(request): Json<ConvertRequest>) -> Result<Json<Value>> {
    if request.url.is_empty() {
        return Err(ErrorResponse::from((StatusCode::BAD_REQUEST, "不能为空")));
    }
    let dst = Uuid::new_v4().to_string();
    let src = request.url;
    let record: Record = Record { src, dst, count: 0 };
    let mut file = state.mutex.lock().await;
    let mut buf = Vec::new();
    file.seek(SeekFrom::Start(0)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.read_to_end(&mut buf).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut records: Vec<Record> = serde_json::from_slice(&buf).unwrap_or_default();
    records.push(record.clone());
    let records_string = serde_json::to_string(&records).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.seek(SeekFrom::Start(0)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.set_len(0).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.write(records_string.as_bytes()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.flush().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(json!(record)))
}

async fn list(State(state): State<Arc<AppState>>) -> Result<Json<Value>> {
    let mut file = state.mutex.lock().await;
    let mut buf = Vec::new();
    file.seek(SeekFrom::Start(0)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.read_to_end(&mut buf).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let records: Vec<Record> = serde_json::from_slice(&buf).unwrap_or_default();
    Ok(Json(json!(records)))
}

async fn redirect(State(state): State<Arc<AppState>>,Path(id): Path<String>) -> Result<Redirect> {
    let mut file = state.mutex.lock().await;
    let mut buf = Vec::new();
    file.seek(SeekFrom::Start(0)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.read_to_end(&mut buf).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut records: Vec<Record> = serde_json::from_slice(&buf).unwrap_or_default();
    let mut found = None;
    for record in records.iter_mut() {
        if record.dst == id {
            record.count += 1;
            found = Some(record.clone());
        }
    }
    if let Some(record) = found {
        let records_string = serde_json::to_string(&records).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.seek(SeekFrom::Start(0)).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.set_len(0).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.write(records_string.as_bytes()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        file.flush().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Redirect::temporary(&record.src));
    }
    Err(StatusCode::NOT_FOUND.into())
}