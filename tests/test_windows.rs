#[cfg(target_os = "windows")]
mod common;

#[cfg(target_os = "windows")]
mod tests {
    const TEST_PLATFORM: &str = "windows";

    use super::common::check_browser;
    use webbrowser::Browser;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_open_default() {
        check_browser(Browser::Default, TEST_PLATFORM).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn test_open_internet_explorer() {
        check_browser(Browser::InternetExplorer, TEST_PLATFORM).await;
    }

    #[test]
    fn test_existence_default() {
        assert!(Browser::is_available(), "should have found a browser");
    }

    #[test]
    fn test_non_existence_safari() {
        assert!(!Browser::Safari.exists(), "should not have found Safari");
    }
}
