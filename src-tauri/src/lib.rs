// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{Manager, Runtime, http::{Request, Response, StatusCode}};
use std::{path::Path, sync::Mutex};
use image_worker::resize_image_to_fit;

struct AppState {
    img_count: Mutex<u32>,
    img_basenames: Mutex<Vec<String>>,
    img_dir: Mutex<String>
}

async fn generate_image_response<R: Runtime>(app: tauri::AppHandle<R>, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path_str = request.uri().path();
    let index = path_str.trim_start_matches('/').parse::<usize>();

    let path = match index {
        Ok(idx) => {
            let state = app.state::<AppState>();
            let basenames = state.img_basenames.lock().unwrap();
            if let Some(name) = basenames.get(idx) {
                let dir = state.img_dir.lock().unwrap();
                Path::new(&*dir).join(name)
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
                img_basenames: Mutex::new(Vec::new()),
                img_dir: Mutex::new("".into()) 
            });
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
