#[cfg(target_os = "macos")]
mod common;

#[cfg(target_os = "macos")]
mod tests {
    const TEST_PLATFORM: &str = "android";

    use super::common::check_request_received_using;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    // to run this test, run it as:
    // cargo test --test test_android -- --ignored
    //
    // For this to run, we need ANDROID_NDK_ROOT env defined, e.g.
    // ANDROID_NDK_ROOT=$ANDROID_SDK_ROOT/ndk/22.1.7171670
    //
    #[ignore]
    #[actix_rt::test]
    async fn test_android() {
        let uri = format!("/{}", TEST_PLATFORM);
        let ipv4 = get_ipv4_address();
        check_request_received_using(uri, &ipv4, |url| {
            // modify android app code to use the correct url
            let mut app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            app_dir.push("tests/test-android-app");
            let mut lib_rs = PathBuf::from(&app_dir);
            lib_rs.push("src/lib.rs");
            let old_code =
                fs::read_to_string(&lib_rs).expect("failed to read lib.rs for android app");
            let new_code = old_code
                .split('\n')
                .map(|s| {
                    if s.starts_with("const SERVER_URL") {
                        format!("const SERVER_URL: &str = \"{}\";", url)
                    } else {
                        s.into()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            fs::write(&lib_rs, &new_code).expect("failed to modify src/lib.rs");

            // invoke app in android
            let apk_run_status = Command::new("cargo")
                .arg("apk")
                .arg("run")
                .current_dir(&app_dir)
                .status();

            // revert to the old code
            fs::write(&lib_rs, &old_code).expect("failed to modify src/lib.rs");

            // check for apk run status
            assert!(
                apk_run_status.expect("cargo apk failed").success(),
                "failed to run: cargo apk run"
            );
        })
        .await;
    }

    fn get_ipv4_address() -> String {
        let output = Command::new("sh")
            .arg("-c")
            .arg("ifconfig | grep 'inet ' | awk '{ print $2 }' | grep -v ^127.0.0")
            .output()
            .expect("failed to get non-local ipv4 address");
        std::str::from_utf8(&output.stdout)
            .expect("unable to parse output into utf8")
            .split('\n')
            .next()
            .expect("no ip address found")
            .into()
    }
}
