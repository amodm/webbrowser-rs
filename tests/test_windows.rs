#[cfg(target_os = "windows")]
mod common;

#[cfg(target_os = "windows")]
mod tests {
    const TEST_PLATFORM: &str = "windows";

    use super::common::{check_browser, check_local_file};
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_local_file_abs_path() {
        check_local_file(Browser::Default, None, |pb| {
            pb.as_os_str().to_string_lossy().into()
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_local_file_rel_path() {
        let cwd = std::env::current_dir().expect("unable to get current dir");
        check_local_file(Browser::Default, None, |pb| {
            pb.strip_prefix(cwd)
                .expect("strip prefix failed")
                .as_os_str()
                .to_string_lossy()
                .into()
        })
        .await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_local_file_uri() {
        check_local_file(Browser::Default, None, |pb| {
            url::Url::from_file_path(pb)
                .expect("failed to convert path to url")
                .to_string()
        })
        .await;
    }
}
