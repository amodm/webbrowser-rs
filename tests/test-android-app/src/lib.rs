#[allow(non_snake_case)]
extern crate ndk_glue;

const SERVER_URL: &str = "http://127.0.0.1";

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn android_main() {
    println!("****** [WEBB DEBUG] ***** begin");
    webbrowser::open(SERVER_URL).unwrap();
    println!("****** [WEBB DEBUG] ***** end");
}
