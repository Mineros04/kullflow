// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
/*#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}*/

mod image;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_asynchronous_uri_scheme_protocol("image", |_app, request, responder| {
            //println!("RUNNING IMAGE PROTOCOL");
            tauri::async_runtime::spawn(async move {
                let response = image::generate_image_response(request).await;

                responder.respond(response);
            });
        })
        .plugin(tauri_plugin_dialog::init())
        //.invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
