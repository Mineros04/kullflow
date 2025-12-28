use image_worker::{ImageData, resize_image_to_fit};
use moka::sync::Cache;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Mutex};
use tauri::{
    Manager, Runtime,
    http::{Request, Response, StatusCode}
};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, log};

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
    img_dir: Mutex<String>,
    image_cache: Cache<usize, ImageData>
}

fn process_image<R: Runtime>(app: &tauri::AppHandle<R>, index: usize) -> Result<ImageData, String> {
    let state = app.state::<AppState>();

    let images = state.images.lock().unwrap();
    let img_info = images.get(index).ok_or("Image index out of bounds.")?;
    let dir = state.img_dir.lock().unwrap();
    let path = Path::new(&*dir).join(&img_info.basename);

    drop(images);
    drop(dir);

    let img_bytes = std::fs::read(&path).map_err(|e| e.to_string())?;

    let window = app.get_webview_window("main").unwrap();
    let window_size = window.inner_size().map_err(|e| e.to_string())?;
    resize_image_to_fit(img_bytes, window_size.width.min(1920), window_size.height.min(1080))
        .map_err(|e| e.to_string())
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
                .body("Invalid image index.".as_bytes().to_vec())
                .unwrap();
        }
    };

    let state = app.state::<AppState>();

    // Get cached image; if not found, process it now.
    let result = if let Some(cached) = state.image_cache.remove(&index) {
        Ok(cached)
    } else {
        process_image(&app, index)
    };

    match result {
        Ok((img, width, height)) => {
            let app_handle = app.clone();
            // Preprocess (resize) images in the background.
            std::thread::spawn(move || {
                let state = app_handle.state::<AppState>();
                // TODO: Make it react to max cache size.
                for i in 1..=5 {
                    let next_idx = index + i;
                    if !state.image_cache.contains_key(&next_idx) {
                        match process_image(&app_handle, next_idx) {
                            Ok(res) => {
                                state.image_cache.insert(next_idx, res);
                            }
                            Err(e) => log::warn!("Failed to pre-fetch image {next_idx}: {e}")
                        }
                    }
                }
            });

            // Update window title to the currently open image.
            let curr_img = state.images.lock().unwrap().get(index).cloned();
            if let Some(curr_img) = curr_img {
                let window = app.get_webview_window("main").unwrap();
                let title = format!("KullFlow - {}", curr_img.basename);

                if let Err(e) = window.set_title(&title) {
                    log::warn!("Failed to update window title on image load: {e}");
                }
            }

            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/octet-stream")
                .header("X-Image-Width", width)
                .header("X-Image-Height", height)
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Expose-Headers", "*")
                .body(img)
                .unwrap()
        }
        Err(e) => {
            log::error!("Failed to resize image: {e}");

            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Access-Control-Allow-Origin", "*")
                .body(e.into_bytes())
                .unwrap()
        }
    }
}

#[tauri::command]
fn init_images<R: Runtime>(app: tauri::AppHandle<R>, dir_str: &str) -> Result<usize, String> {
    let dir_path = Path::new(dir_str);

    let mut entries = std::fs::read_dir(&dir_path).map_err(|e| {
        log::error!("Failed to read directory: {e}");
        e.to_string()
    })?;
    let mut images = Vec::new();

    while let Some(entry) = entries.next() {
        let path = entry
            .map_err(|e| {
                log::error!("Failed to read directory entry: {e}");
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
        return Err("Image index out of bounds.".into());
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Trace
                } else {
                    log::LevelFilter::Info
                })
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some("kullflow.log".into()) })
                ])
                .rotation_strategy(RotationStrategy::KeepSome(5))
                .max_file_size(1_000_000)
                .build()
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .register_uri_scheme_protocol("image", |ctx, request| {
            let app = ctx.app_handle().clone();
            generate_image_response(app, request)
        })
        .setup(|app| {
            let image_cache = Cache::builder().max_capacity(5).build();

            app.manage(AppState {
                img_count: Mutex::new(0),
                images: Mutex::new(Vec::new()),
                img_dir: Mutex::new("".into()),
                image_cache
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![init_images, vote_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
