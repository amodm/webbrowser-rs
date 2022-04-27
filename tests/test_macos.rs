#[cfg(target_os = "macos")]
mod common;

#[cfg(target_os = "macos")]
mod tests {
    const TEST_PLATFORM: &str = "macos";

    use super::common::check_browser;
    use webbrowser::Browser;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_open_default() {
        check_browser(Browser::Default, TEST_PLATFORM).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_open_safari() {
        check_browser(Browser::Safari, TEST_PLATFORM).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn test_open_firefox() {
        check_browser(Browser::Firefox, TEST_PLATFORM).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore]
    async fn test_open_chrome() {
        check_browser(Browser::Chrome, TEST_PLATFORM).await;
    }

    #[test]
    fn test_existence_default() {
        assert!(Browser::is_available(), "should have found a browser");
    }

    #[test]
    fn test_existence_safari() {
        assert!(Browser::Safari.exists(), "should have found Safari");
    }

    #[test]
    fn test_non_existence_webpositive() {
        assert!(
            !Browser::WebPositive.exists(),
            "should not have found WebPositive",
        );
    }
}
