use serde::{Deserialize, Serialize};
use tauri::{Manager, Runtime, http::{Request, Response, StatusCode}};
use std::{path::Path, sync::Mutex};
use image_worker::resize_image_to_fit;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CullState {
    Pending,
    Keep,
    Delete
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub basename: String,
    pub status: CullState
}

struct AppState {
    img_count: Mutex<usize>,
    images: Mutex<Vec<ImageInfo>>,
    img_dir: Mutex<String>
}

async fn generate_image_response<R: Runtime>(app: tauri::AppHandle<R>, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path_str = request.uri().path();
    let index = path_str.trim_start_matches('/').parse::<usize>();

    // Prepare the path of the image from the app state based on provided index.
    let path = match index {
        Ok(idx) => {
            let state = app.state::<AppState>();
            let basenames = state.images.lock().unwrap();

            if let Some(img_info) = basenames.get(idx) {
                let dir = state.img_dir.lock().unwrap();
                Path::new(&*dir).join(&img_info.basename)
            } else {
                return Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Access-Control-Allow-Origin", "*")
                    .body("Image index out of bounds.".as_bytes().to_vec())
                    .unwrap();
            }
        }
        Err(_) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .header("Access-Control-Allow-Origin", "*")
                .body("Invalid image index.".as_bytes().to_vec())
                .unwrap();
        }
    };

    // Asynchronously read the file and return it as a Response.
    match tokio::fs::read(&path).await {
        Ok(data) => {
            // Resize the image if needed to ensure we do not transfer images that are too large.
            // TODO: Adapt size to users screen (or really anything, so that the dimensions are not static ints).
            let img_res = resize_image_to_fit(data, 1920, 1080);
            return match img_res {
                // Image converted successfully.
                Ok((img, width, height)) => {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("Content-Type", "application/octet-stream")
                        .header("X-Image-Width", width)
                        .header("X-Image-Height", height)
                        .header("Access-Control-Allow-Origin", "*")
                        .header("Access-Control-Expose-Headers", "*")
                        .body(img)
                        .unwrap()
                },
                // Resize failed, return error message from resizing.
                Err(e) => Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header("Access-Control-Allow-Origin", "*")
                    .body(e.to_string().into_bytes())
                    .unwrap()
            }
        }
        Err(_) => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Access-Control-Allow-Origin", "*")
                .body("File not found.".as_bytes().to_vec())
                .unwrap()
        }
    }
}

#[tauri::command]
async fn init_images<R: Runtime>(app: tauri::AppHandle<R>, dir_str: &str) -> Result<usize, String> {
    let dir_path = Path::new(dir_str);
    
    let mut entries = tokio::fs::read_dir(dir_path).await.map_err(|e| e.to_string())?;
    let mut images = Vec::new();

    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();

        // Ignore non-image files.
        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        if mime.type_() != mime::IMAGE {
            continue;
        }

        // Only accept valid UTF-8 filenames to ensure they can be opened later.
        if path.is_file() && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            images.push(ImageInfo { 
                basename: name.to_string(),
                status: CullState::Pending
            });
        }
    }

    let state = app.state::<AppState>();
    let images_len = images.len();
    *state.img_count.lock().unwrap() = images_len;
    *state.images.lock().unwrap() = images;
    *state.img_dir.lock().unwrap() = dir_str.to_string();

    Ok(images_len)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol("image", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let response = generate_image_response(app, request).await;

                responder.respond(response);
            });
        })
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            app.manage(AppState {
                img_count: Mutex::new(0),
                images: Mutex::new(Vec::new()),
                img_dir: Mutex::new("".into()) 
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![init_images])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
