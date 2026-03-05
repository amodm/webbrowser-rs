const SERVER_URL: &str = "http://127.0.0.1";

#[unsafe(no_mangle)]
pub fn android_main(_app: android_activity::AndroidApp) {
    println!("****** [WEBB DEBUG] {} ***** begin", SERVER_URL);
    webbrowser::open(SERVER_URL).unwrap();
    println!("****** [WEBB DEBUG] {} ***** end", SERVER_URL);
}
