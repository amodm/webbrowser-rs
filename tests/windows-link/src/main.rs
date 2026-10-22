fn main() -> std::io::Result<()> {
    // Link the Windows browser implementation without launching a browser.
    // Loading this executable also verifies the generated DLL import names.
    webbrowser::open_browser_with_options(
        webbrowser::Browser::Default,
        "https://example.com",
        webbrowser::BrowserOptions::new().with_dry_run(true),
    )?;
    println!("Windows link smoke test passed");
    Ok(())
}
