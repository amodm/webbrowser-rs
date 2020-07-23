#[cfg(all(unix, not(target_os = "macos")))]
mod common;

#[cfg(all(unix, not(target_os = "macos")))]
mod tests {
    const TEST_PLATFORM: &str = "unix";

    use super::common::check_browser;
    use webbrowser::Browser;

    #[actix_rt::test]
    async fn test_open_default() {
        check_browser(Browser::Default, TEST_PLATFORM).await;
    }
}
