// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{Manager, http::{Request, Response, StatusCode}};
use std::{path::PathBuf, sync::Mutex};
use image_worker::resize_image_to_fit;

struct AppState {
    img_count: Mutex<u32>,
    img_basenames: Mutex<Vec<String>>,
    img_dir: Mutex<String>
}

pub async fn generate_image_response(request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let path_str = request.uri().path();
    
    // Decode the URL
    let decoded_path = urlencoding::decode(path_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| path_str.to_string());

    let path = if cfg!(windows) && decoded_path.starts_with('/') {
        PathBuf::from(&decoded_path[1..])
    } else {
        PathBuf::from(&decoded_path)
    };

    let extension = path.extension().unwrap_or_default().to_string_lossy();
    let mime_type = mime_guess::from_ext(&extension).first_or_octet_stream();
    if mime_type.type_() != mime::IMAGE {
        return Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .header("Access-Control-Allow-Origin", "*")
            .body("File is not an image.".as_bytes().to_vec())
            .unwrap();
    }

    // Asynchronously read the file and return it as a Response.
    match tokio::fs::read(&path).await {
        Ok(data) => {
            // Resize the image if needed to ensure we do not transfer images that are too large.
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
        .register_asynchronous_uri_scheme_protocol("image", |_app, request, responder| {
            //println!("RUNNING IMAGE PROTOCOL");
            tauri::async_runtime::spawn(async move {
                let response = generate_image_response(request).await;

                responder.respond(response);
            });
        })
        .plugin(tauri_plugin_dialog::init())
        //.invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
