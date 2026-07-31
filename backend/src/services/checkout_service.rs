use std::path::Path;

use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use crate::{entities::checkouts, error::AppError, state::AppState, util};

pub struct PhotoUpload {
    pub bytes: Vec<u8>,
    pub mime: String,
}

pub struct NewCheckout {
    pub user_id: Uuid,
    pub note: String,
    pub access_session_id: Option<Uuid>,
    pub box_device_id: Option<Uuid>,
    pub photo: Option<PhotoUpload>,
}

/// Record a materials checkout: a required text note plus an optional photo,
/// which is written under the upload dir and exposed via `/uploads`.
pub async fn create(state: &AppState, input: NewCheckout) -> Result<checkouts::Model, AppError> {
    let note = input.note.trim().to_owned();
    if note.is_empty() {
        return Err(AppError::Validation(
            "a note describing the materials taken is required".to_owned(),
        ));
    }

    let (photo_path, photo_mime) = match input.photo {
        Some(photo) => {
            let (path, mime) = save_photo(state, photo).await?;
            (Some(path), Some(mime))
        }
        None => (None, None),
    };

    let checkout = checkouts::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(input.user_id),
        access_session_id: Set(input.access_session_id),
        box_device_id: Set(input.box_device_id),
        note: Set(note),
        photo_path: Set(photo_path),
        photo_mime: Set(photo_mime),
        created_at: Set(util::now()),
    }
    .insert(&state.db)
    .await?;

    Ok(checkout)
}

async fn save_photo(state: &AppState, photo: PhotoUpload) -> Result<(String, String), AppError> {
    let dir = Path::new(&state.cfg.upload_dir);
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    let ext = mime_guess::get_mime_extensions_str(&photo.mime)
        .and_then(|exts| exts.first().copied())
        .unwrap_or("bin");
    let filename = format!("{}.{}", Uuid::new_v4(), ext);

    tokio::fs::write(dir.join(&filename), &photo.bytes)
        .await
        .map_err(|e| AppError::Internal(e.into()))?;

    Ok((format!("/uploads/{filename}"), photo.mime))
}
