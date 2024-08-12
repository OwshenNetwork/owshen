use axum::http::{header, HeaderValue};
use axum::response::Response;
use eyre::Report;
use serde::Serialize;

#[derive(Serialize)]
pub struct StyleContext {}

pub async fn css_handler(
    _ctx: &StyleContext, // '_ctx' is not used, but retained if you might use it later
) -> Result<Response<String>, Report> {
    // Your CSS file content
    let css_content = r#"
        body{
        background:black;
        color:white;
        }
        h1 {
            font-family: "JetBrains Mono";
            margin: 0.5em;
            font-size: 3em;
            text-align: center;
        }
        a{
        display: block;
        width: 100%;
        }
        body {
            font-family: "JetBrains Mono";
            line-height: 1.4em;
            font-size: 1.1em;
        }

        h2 {
            font-size: 1.5em;
            text-align: center;
        }

        b {
            font-weight: bold;
        }

        th {
            font-weight: bold;
            border-bottom: 1px solid white;
        }

        th, td {
            padding: 0.2em;
        }
    "#;

    // Create the response with the CSS content
    let mut resp = Response::new(css_content.to_string());
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/css"));
    Ok(resp)
}
