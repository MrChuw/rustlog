use axum::{
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};
use reqwest::header;
use rust_embed::RustEmbed;
use std::{env, sync::OnceLock};

const INDEX_HTML: &str = "index.html";

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/web/dist"]
struct Assets;
static INDEX_HTML_CACHED: OnceLock<String> = OnceLock::new();

pub async fn static_asset(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() || path == INDEX_HTML {
        return index_html().await;
    }

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return not_found().await;
            }

            index_html().await
        }
    }
}

fn analytics_script() -> Option<String> {
    let url = env::var("ANALYTICS_URL").ok()?;
    let uuid = env::var("ANALYTICS_UUID").ok()?;

    Some(format!(
        r#"<script defer src="{url}" data-website-id="{uuid}"></script>"#
    ))
}

async fn index_html() -> Response {
    let html = INDEX_HTML_CACHED.get_or_init(|| {
        let content = Assets::get(INDEX_HTML)
            .expect("index.html not found in the embed");

        let mut html =
            String::from_utf8(content.data.to_vec()).expect("index.html invalid");

        if let Some(script) = analytics_script() {
            if html.contains("</head>") {
                html = html.replace("</head>", &format!("{script}\n</head>"));
            }
        }

        html
    });

    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html.clone(),
    )
        .into_response()
}

async fn not_found() -> Response {
    (StatusCode::NOT_FOUND, "404").into_response()
}
