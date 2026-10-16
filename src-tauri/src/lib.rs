use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Mutex};
use tauri::{
    Manager, Runtime,
    http::{Request, Response, StatusCode}
};
use tauri_plugin_log::{
    RotationStrategy, Target, TargetKind,
    log::{LevelFilter, error, info, warn}
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CullState {
    Pending,
    Keep,
    Delete
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteAction {
    Keep,
    Delete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub basename: String,
    pub status: CullState
}

struct AppState {
    // TODO: Consider using RWMutexes instead (we write once and read a lot).
    img_count: Mutex<usize>,
    images: Mutex<Vec<ImageInfo>>,
    img_dir: Mutex<String>
}

fn generate_image_response<R: Runtime>(
    app: tauri::AppHandle<R>,
    request: Request<Vec<u8>>
) -> Response<Vec<u8>> {
    let path_str = request.uri().path();
    let index = match path_str.trim_start_matches('/').parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Access-Control-Allow-Origin", "*")
                .body("Invalid image index".as_bytes().to_vec())
                .unwrap();
        }
    };

    let state = app.state::<AppState>();

    let (path, basename) = {
        let images = state.images.lock().unwrap();
        let Some(image) = images.get(index) else {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Cache-Control", "no-store")
                .body(b"Image index out of bounds".to_vec())
                .unwrap();
        };
        let dir = state.img_dir.lock().unwrap();
        (Path::new(&*dir).join(&image.basename), image.basename.clone())
    };

    match std::fs::read(&path) {
        Ok(bytes) => {
            // Update window title to the currently open image.
            if let Some(window) = app.get_webview_window("main") {
                let title = format!("KullFlow - {basename}");

                if let Err(e) = window.set_title(&title) {
                    warn!("Failed to update window title on image load: {e}");
                }
            }

            Response::builder()
                .status(StatusCode::OK)
                .header(
                    "Content-Type",
                    mime_guess::from_path(&path).first_or_octet_stream().as_ref()
                )
                // Indices are reused when a different directory is selected.
                .header("Cache-Control", "no-store")
                .header("Access-Control-Allow-Origin", "*")
                .body(bytes)
                .unwrap()
        }
        Err(e) => {
            error!("Failed to read image {}: {e}", path.display());

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Cache-Control", "no-store")
                .header("Access-Control-Allow-Origin", "*")
                .body(e.to_string().into_bytes())
                .unwrap()
        }
    }
}

#[tauri::command]
fn init_images<R: Runtime>(app: tauri::AppHandle<R>, dir_str: &str) -> Result<usize, String> {
    let dir_path = Path::new(dir_str);

    let mut entries = std::fs::read_dir(&dir_path).map_err(|e| {
        error!("Failed to read directory: {e}");
        e.to_string()
    })?;
    let mut images = Vec::new();

    while let Some(entry) = entries.next() {
        let path = entry
            .map_err(|e| {
                error!("Failed to read directory entry: {e}");
                e.to_string()
            })?
            .path();

        // Ignore non-image files.
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        if mime.type_() != mime::IMAGE {
            continue;
        }

        // Only accept valid UTF-8 filenames to ensure they can be opened later.
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            images.push(ImageInfo { basename: name.to_owned(), status: CullState::Pending });
        }
    }

    let state = app.state::<AppState>();
    let images_len = images.len();
    *state.img_count.lock().unwrap() = images_len;
    *state.images.lock().unwrap() = images;
    *state.img_dir.lock().unwrap() = dir_str.to_string();

    info!("Initialized app state with {images_len} images");

    Ok(images_len)
}

#[tauri::command]
fn vote_image<R: Runtime>(
    app: tauri::AppHandle<R>,
    index: usize,
    action: VoteAction
) -> Result<(), String> {
    let state = app.state::<AppState>();
    let mut images = state.images.lock().unwrap();

    if let Some(image) = images.get_mut(index) {
        image.status = match action {
            VoteAction::Keep => CullState::Keep,
            VoteAction::Delete => CullState::Delete
        };
    } else {
        return Err("Image index out of bounds".into());
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) { LevelFilter::Trace } else { LevelFilter::Info })
                // Disable tao's warnings, as they clutter the logs.
                .level_for("tao", LevelFilter::Error)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("kullflow.log".into()) })
                ])
                .rotation_strategy(RotationStrategy::KeepSome(5))
                .max_file_size(1_000_000)
                .build()
        )
        .plugin(tauri_plugin_dialog::init())
        .register_uri_scheme_protocol("image", |ctx, request| {
            let app = ctx.app_handle().clone();
            generate_image_response(app, request)
        })
        .setup(|app| {
            app.manage(AppState {
                img_count: Mutex::new(0),
                images: Mutex::new(Vec::new()),
                img_dir: Mutex::new("".into())
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![init_images, vote_image])
        .run(tauri::generate_context!())
    {
        error!("Failed to start KullFlow: {e}");
    }
}
