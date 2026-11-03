use http::{Request, Response, StatusCode, header};
use tauri::{Manager, UriSchemeContext, UriSchemeResponder};

use crate::state::AppState;
use crate::thumbs;

/// Windows WebView2 loads this scheme at http://wojak.localhost/<kind>?p=<path>.
/// Only paths already stored in the index are served. Nothing is written into
/// the library tree.
pub fn handle(
    ctx: UriSchemeContext<'_, tauri::Wry>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    let uri = request.uri().clone();
    std::thread::spawn(move || {
        let response = serve(&app, uri.path(), uri.query());
        responder.respond(response);
    });
}

fn serve(app: &tauri::AppHandle, path: &str, query: Option<&str>) -> Response<Vec<u8>> {
    let Some(state) = app.try_state::<AppState>() else {
        return empty(StatusCode::SERVICE_UNAVAILABLE);
    };
    let kind = path.rsplit('/').next().unwrap_or("");
    let Some(image_path) = query_param(query, "p") else {
        return empty(StatusCode::BAD_REQUEST);
    };
    let result = match kind {
        "thumb" => thumbs::ensure_jpeg(&state, &image_path).map(|bytes| ("image/jpeg", bytes)),
        "file" => thumbs::read_original(&state, &image_path),
        _ => Err("Unknown media request.".into()),
    };
    match result {
        Ok((content_type, bytes)) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "private, max-age=31536000")
            .body(bytes)
            .unwrap_or_else(|_| empty(StatusCode::INTERNAL_SERVER_ERROR)),
        Err(_) => empty(StatusCode::NOT_FOUND),
    }
}

fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    let query = query?;
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if name == key {
            return Some(percent_decode(value));
        }
    }
    None
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn empty(status: StatusCode) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(header::CACHE_CONTROL, "no-store")
        .body(Vec::new())
        .unwrap_or_else(|_| Response::new(Vec::new()))
}
