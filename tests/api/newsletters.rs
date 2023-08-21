//! tests/api/main.rs

use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0) // Asserting that no request is fired to Postmark!
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_form = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "<p> Newsletter body as HTML </p>",
        "plain_text_content": "Newsletter body as plain text",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&newsletter_form).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");
    // Act - Part 3 - Follow the redirect
    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>Newsletter sent</i></p>"));
    // Mock verifies on Drop that the newsletters are not sent
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request).await
}

async fn create_confirmed_subscriber(app: &TestApp) {
    // Arrange
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;

    // Act - Part 1 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let newsletter_form = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "<p> Newsletter body as HTML </p>",
        "plain_text_content": "Newsletter body as plain text",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&newsletter_form).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");
    // Act - Part 3 - Follow the redirect
    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>Newsletter sent</i></p>"));
    // Mock verifies on drop that we have sent the newsletter email
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    let test_cases = vec![
        (
            serde_json::json!({
                "html_content": "<p> Newsletter body as HTML </p>",
                "plain_text_content": "Newsletter body as plain text",
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "plain_text_content": "Newsletter body as plain text",
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
            }),
            "missing html",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "html_content": "<p> Newsletter body as HTML </p>",
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
            }),
            "missing plain text",
        ),
        (
            serde_json::json!({
                "title": "Newsletter title",
                "html_content": "<p> Newsletter body as HTML </p>",
                "plain_text_content": "Newsletter body as plain text",
            }),
            "missing idempotency key",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(&invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}

#[tokio::test]
async fn unauthenticated_user_is_redirected_to_login() {
    // Arrange
    let app = spawn_app().await;
    let newsletter_form = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "<p> Newsletter body as HTML </p>",
        "plain_text_content": "Newsletter body as plain text",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&newsletter_form).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_form = serde_json::json!({
        "title": "Newsletter title",
        "html_content": "<p> Newsletter body as HTML </p>",
        "plain_text_content": "Newsletter body as plain text",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });
    let response = app.post_newsletters(&newsletter_form).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>Newsletter sent</i></p>"));

    // Act - Part 3 - Submit newsletter form **again**
    let response = app.post_newsletters(&newsletter_form).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 4 - Follow the redirect
    let html_page = app.get_newsletters_html().await;
    assert!(html_page.contains("<p><i>Newsletter sent</i></p>"));

    // Mock verifies on Drop that we have sent the newsletter **ONCE**
}
