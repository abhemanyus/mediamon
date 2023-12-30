use axum::extract::DefaultBodyLimit;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use utoipa::openapi::security::ApiKey;
use utoipa::openapi::security::ApiKeyValue;
use utoipa::openapi::security::SecurityScheme;
use utoipa::Modify;
use utoipa::OpenApi;
use utoipa::ToSchema;

use axum::{
    extract::{MatchedPath, Multipart},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing, Router,
};
use futures::TryStreamExt;
use log::info;
use tokio_util::io::StreamReader;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};
use tracing::info_span;
use utoipa_swagger_ui::SwaggerUi;

use crate::database::Database;
use crate::deepbooru::Jarvis;

pub fn router(jarvis: Jarvis, db: Database) -> Router {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let trace_layer = TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
        // Log the matched route's path (with placeholders not filled in).
        // Use request.uri() or OriginalUri if you want the real path.
        let matched_path = request
            .extensions()
            .get::<MatchedPath>()
            .map(MatchedPath::as_str);

        info_span!(
            "http_request",
            method = ?request.method(),
            matched_path,
            some_other_field = tracing::field::Empty,
        )
    });
    let cors_layer = CorsLayer::new()
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_origin(tower_http::cors::Any);
    let app_state = AppState { jarvis, db };
    let app = Router::new()
        .route("/", routing::get(root))
        .route(
            "/upload/image/file",
            routing::post(upload_image_file).layer(DefaultBodyLimit::disable()),
        )
        .route("/upload/image/url", routing::post(upload_image_url))
        .route(
            "/upload/video/file",
            routing::post(upload_video_file).layer(DefaultBodyLimit::disable()),
        )
        .route("/upload/video/url", routing::post(upload_video_url))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors_layer)
        .layer(trace_layer)
        .layer(CompressionLayer::new().gzip(true).deflate(true))
        .with_state(Arc::new(app_state));
    app
}

struct AppState {
    jarvis: Jarvis,
    db: Database,
}

#[utoipa::path(
    post,
    path = "/upload/image/url",
    request_body(content = UploadUrlBody),
    responses(
        (status = 201, description = "Downloaded file successfully", body = String),
        (status = 400, description = "Failed to download file", body = String),
    )
)]

async fn upload_image_url(Json(body): Json<UploadUrlBody>) -> Response {
    body.url.into_response()
}

#[utoipa::path(
    post,
    path = "/upload/video/url",
    request_body(content = UploadUrlBody),
    responses(
        (status = 201, description = "Downloaded file successfully", body = String),
        (status = 400, description = "Failed to download file", body = String),
    )
)]

async fn upload_video_url(Json(body): Json<UploadUrlBody>) -> Response {
    body.url.into_response()
}

#[derive(ToSchema, Deserialize)]
struct UploadUrlBody {
    url: String,
}

#[utoipa::path(
    post,
    path = "/upload/image/file",
    request_body(content = UploadFileBody, content_type="multipart/form-data"),
    responses(
        (status = 201, description = "Uploaded file successfully", body = String),
        (status = 400, description = "Failed to upload file", body = String),
    )
)]

async fn upload_image_file(State(state): State<Arc<AppState>>, multipart: Multipart) -> Response {
    info!("Uploading...");
    let file = extract_file("file", multipart).await.unwrap();
    let file_data = tokio::fs::read(&file.file_path).await.unwrap();
    let image_data = image::load_from_memory(&file_data).unwrap();
    let image_tags = state.jarvis.infer_tags(&image_data).unwrap();
    let tag_names: Vec<(String, (f32, usize))> = state
        .db
        .get_tag_names(
            &image_tags
                .iter()
                .map(|(_, tag_id)| *tag_id as i32)
                .collect::<Vec<i32>>(),
        )
        .await
        .unwrap()
        .into_iter()
        .map(|(_, tag_name)| tag_name)
        .zip(image_tags)
        .collect();
    serde_json::to_string_pretty(&tag_names)
        .unwrap()
        .into_response()
}

#[utoipa::path(
    post,
    path = "/upload/video/file",
    request_body(content = UploadFileBody, content_type="multipart/form-data"),
    responses(
        (status = 201, description = "Uploaded file successfully", body = String),
        (status = 400, description = "Failed to upload file", body = String),
    )
)]

async fn upload_video_file(multipart: Multipart) -> Response {
    info!("Uploading...");
    let file = extract_file("file", multipart).await.unwrap();
    file.file_path.into_response()
}

#[derive(ToSchema)]
struct UploadFileBody {
    #[schema(value_type = String, format = Binary)]
    file: Vec<u8>,
}

struct MultipartFile {
    file_path: String,
    file_name: Option<String>,
    file_type: Option<String>,
}
async fn extract_file(field_name: &str, mut multipart: Multipart) -> Result<MultipartFile, ()> {
    let field = loop {
        match multipart.next_field().await {
            Ok(Some(field)) => match field.name() {
                Some(name) if name == field_name => break field,
                _ => continue,
            },
            _ => return Err(()),
        }
    };
    let file_name = field.file_name().map(|str| str.to_owned());
    let file_type = field.content_type().map(|str| str.to_owned());

    let body_with_io_error =
        field.map_err(|err| tokio::io::Error::new(tokio::io::ErrorKind::Other, err));
    let mut body_reader = StreamReader::new(body_with_io_error);
    let file_path = format!("/tmp/{}", uuid::Uuid::new_v4().to_string());
    let mut file_store = tokio::fs::File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&file_path)
        .await
        .unwrap();
    tokio::io::copy(&mut body_reader, &mut file_store)
        .await
        .unwrap();
    Ok(MultipartFile {
        file_path,
        file_name,
        file_type,
    })
}

#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "List all todos successfully", body = String),
        (status = 201, description = "Random bullshit go!", body = u8),
    )
)]
async fn root() -> Response {
    info!("Rooting...");
    match std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        % 2
        == 0
    {
        true => "Hello world!".into_response(),
        false => (StatusCode::CREATED, "test").into_response(),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        root,
        upload_image_file,
        upload_image_url,
        upload_video_file,
        upload_video_url,
    ),
    components(schemas(UploadFileBody, UploadUrlBody)),
    modifiers(&SecurityAddon),
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("mediamon_api_key"))),
            )
        }
    }
}
