use actix_files as fs;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use crossbeam_channel as cbc;
use std::sync::Arc;
use urlencoding::decode;
use webbrowser::{open_browser, Browser};

#[derive(Clone)]
struct AppState {
    tx: Arc<cbc::Sender<String>>,
}

async fn log_handler(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    if data.tx.send(req.uri().to_string()).is_err() {
        panic!("channel send failed");
    }
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(format!("<html><body><p>URI: {}</p><script type=\"text/javascript>window.close();</script></body></html>", req.uri()))
}

pub async fn check_request_received_using<F>(uri: String, host: &str, op: F)
where
    F: FnOnce(&str),
{
    // start the server on a random port
    let bind_addr = format!("{}:0", host);
    let (tx, rx) = cbc::bounded(2);
    let data = AppState {
        tx: Arc::new(tx.clone()),
    };
    let http_server = HttpServer::new(move || {
        App::new()
            .service(fs::Files::new("/static/wasm", "tests/test-wasm-app/pkg"))
            .data(data.clone())
            .route("/*", web::get().to(log_handler))
    })
    .bind(&bind_addr)
    .unwrap_or_else(|_| panic!("Can not bind to {}", &bind_addr));

    let port = http_server
        .addrs()
        .first()
        .expect("Failed to find bound address")
        .port();

    let server = http_server.run();

    // invoke the op
    op(&format!("http://{}:{}{}", host, port, &uri));

    // wait for the url to be hit
    match rx.recv_timeout(std::time::Duration::from_secs(30)) {
        Ok(msg) => assert_eq!(decode(&msg).unwrap(), uri),
        Err(_) => panic!("failed to receive uri data"),
    }

    // stop the server
    server.stop(true).await;
}

#[allow(dead_code)]
pub async fn check_request_received(browser: Browser, uri: String) {
    check_request_received_using(uri, "127.0.0.1", |url| {
        open_browser(browser, url).expect("failed to open browser");
    })
    .await;
}

#[allow(dead_code)]
pub async fn check_browser(browser: Browser, platform: &str) {
    check_request_received(browser, format!("/{}", platform)).await;
    check_request_received(browser, format!("/{}/😀😀😀", platform)).await;
}
