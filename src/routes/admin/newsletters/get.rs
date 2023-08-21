//! src/routes/admin/newsletters/get.rs

use actix_web::http::header::ContentType;
use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn publish_newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }
    let idempotency_key = uuid::Uuid::new_v4();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Publish newsletter</title>
        </head>
        <body>
            <form action="/admin/newsletters" method="post">
            {msg_html}
                <label>Newsletter Title:<br>
                    <input
                        type="text"
                        placeholder="Enter newsletter title"
                        name="newsletter_title"
                    >
                </label>
                <br>
                <label>HTML newsletter:<br>
                    <textarea
                        name="newsletter_html"
                        rows="25"
                        cols="65"
                    >
                    </textarea>
                </label>
                <br>
                <label>Plain text newsletter:<br>
                    <textarea
                        name="newsletter_plain_text"
                        rows="25"
                        cols="65"
                    >
                    </textarea>
                </label>
                <br>
                <input hidden type="text" name="idempotency_key" value="{idempotency_key}">
                <button type="submit">Publish</button>
            </form>
            <p><a href="/admin/dashboard">&lt; Back</a></p>
        </body>
        </html>
        "#,
        )))
}
