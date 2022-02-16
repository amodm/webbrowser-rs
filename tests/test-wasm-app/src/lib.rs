use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn test_open_browser(url: String) {
    web_sys::console::log_1(&format!("checking in {}", &url).into());
    webbrowser::open(&url).expect("failed to open browser");
    web_sys::console::log_1(&"yolo".into());
}
