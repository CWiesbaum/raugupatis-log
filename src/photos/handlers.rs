use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use tower_sessions::Session;

use crate::fermentation::repository::FermentationRepository;
use crate::photos::models::{PhotoResponse, PhotoStage};
use crate::photos::repository::PhotoRepository;
use crate::users::UserSession;
use crate::AppState;

use std::fs;
use std::io::Write;

pub async fn upload_photo(
    State(state): State<AppState>,
    session: Session,
    Path(fermentation_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<PhotoResponse>), StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify fermentation exists and belongs to user
    let fermentation_repo = FermentationRepository::new(state.db.clone());
    let fermentation = fermentation_repo
        .find_by_id(fermentation_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut file_data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;
    let mut caption: Option<String> = None;
    let mut stage = PhotoStage::Progress;

    // Parse multipart form data
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "photo" => {
                file_name = field
                    .file_name()
                    .map(|name| sanitize_filename(name.to_string()));
                file_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .to_vec(),
                );
            }
            "caption" => {
                caption = field
                    .text()
                    .await
                    .ok()
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| s.to_string());
            }
            "stage" => {
                let stage_str = field.text().await.unwrap_or_default();
                stage = PhotoStage::from(stage_str);
            }
            _ => {}
        }
    }

    let file_data = file_data.ok_or(StatusCode::BAD_REQUEST)?;
    let file_name = file_name.ok_or(StatusCode::BAD_REQUEST)?;

    // Validate file size (max 10MB)
    if file_data.len() > 10 * 1024 * 1024 {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // Validate file extension
    let extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_default();

    if !["jpg", "jpeg", "png", "gif", "webp"].contains(&extension.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create uploads directory if it doesn't exist
    let uploads_dir = &state.config.uploads_dir;
    let fermentation_dir = format!("{}/{}", uploads_dir, fermentation.id);
    fs::create_dir_all(&fermentation_dir).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Generate unique filename using timestamp and nanoseconds
    let now = Utc::now();
    let timestamp = now.timestamp();
    let nanos = now.timestamp_subsec_nanos();
    let unique_filename = format!("{}_{:x}.{}", timestamp, nanos, extension);
    let file_path = format!("{}/{}", fermentation_dir, unique_filename);

    // Write file to disk
    let mut file = fs::File::create(&file_path).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    file.write_all(&file_data)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store photo metadata in database
    let photo_repo = PhotoRepository::new(state.db.clone());
    let relative_path = format!("{}/{}", fermentation.id, unique_filename);

    let photo = photo_repo
        .create_photo(fermentation.id, relative_path, caption, Utc::now(), stage)
        .await
        .map_err(|e| {
            tracing::error!("Error creating photo record: {}", e);
            // Clean up file if database insert fails
            let _ = fs::remove_file(&file_path);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(PhotoResponse::from(photo))))
}

pub async fn list_photos(
    State(state): State<AppState>,
    session: Session,
    Path(fermentation_id): Path<i64>,
) -> Result<Json<Vec<PhotoResponse>>, StatusCode> {
    // Get user from session
    let user_session: Option<UserSession> = session
        .get("user")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = user_session.ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify fermentation exists and belongs to user
    let fermentation_repo = FermentationRepository::new(state.db.clone());
    fermentation_repo
        .find_by_id(fermentation_id, user.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get photos
    let photo_repo = PhotoRepository::new(state.db.clone());
    let photos = photo_repo
        .find_by_fermentation(fermentation_id)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching photos: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(photos.into_iter().map(PhotoResponse::from).collect()))
}

fn sanitize_filename(filename: String) -> String {
    // Remove any path components and keep only the filename
    let filename = std::path::Path::new(&filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("upload")
        .to_string();

    // Replace any potentially problematic characters
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
