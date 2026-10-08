use serde_json::json;
use url::Url;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wpx_api::WpClient;

fn client_for(server: &MockServer) -> WpClient {
    WpClient::new(
        Url::parse(&server.uri()).unwrap(),
        Box::new(wpx_auth::ApplicationPasswordAuth::new(
            "bot".into(),
            "secret".into(),
        )),
        5,
        0,
    )
    .unwrap()
}

#[tokio::test]
async fn upload_file_sends_multipart_with_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/wp-json/wp/v2/media"))
        .and(header("authorization", "Basic Ym90OnNlY3JldA=="))
        .and(body_string_contains("name=\"file\"; filename=\"hero.png\""))
        .and(body_string_contains("Content-Type: image/png"))
        .and(body_string_contains("name=\"title\""))
        .and(body_string_contains("Hero image"))
        .and(body_string_contains("name=\"alt_text\""))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": 6168,
            "slug": "hero",
            "title": {"rendered": "Hero image"},
            "mime_type": "image/png",
            "source_url": "https://example.com/wp-content/uploads/hero.png"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp: wpx_api::ApiResponse<serde_json::Value> = client
        .upload_file(
            "wp/v2/media",
            "hero.png",
            b"fake-png-bytes".to_vec(),
            "image/png",
            &[
                ("title", "Hero image".to_string()),
                ("alt_text", "A hero".to_string()),
            ],
        )
        .await
        .unwrap();

    assert_eq!(resp.data["id"], 6168);
    assert_eq!(resp.data["mime_type"], "image/png");
}

#[tokio::test]
async fn request_raw_get_with_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wp-json/wp/v2/types/blog"))
        .and(query_param("context", "edit"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "slug": "blog", "rest_base": "blog", "rest_namespace": "wp/v2"
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server);
    let resp = client
        .request_raw("get", "/wp/v2/types/blog", &[("context", "edit")], None)
        .await
        .unwrap();
    assert_eq!(resp.data["rest_base"], "blog");
}

#[tokio::test]
async fn request_raw_post_with_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/wp-json/rankmath/v1/updateMeta"))
        .and(header("content-type", "application/json"))
        .and(body_string_contains("\"objectID\":42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"success": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = client_for(&server);
    let body = json!({"objectType": "post", "objectID": 42, "meta": {"rank_math_title": "x"}});
    let resp = client
        .request_raw("POST", "rankmath/v1/updateMeta", &[], Some(&body))
        .await
        .unwrap();
    assert_eq!(resp.data["success"], true);
}

#[tokio::test]
async fn request_raw_rejects_unknown_method() {
    let server = MockServer::start().await;
    let client = client_for(&server);
    let err = client
        .request_raw("FETCH", "wp/v2/posts", &[], None)
        .await
        .unwrap_err();
    assert!(matches!(err, wpx_core::WpxError::Validation { .. }));
}

#[tokio::test]
async fn request_raw_maps_wordpress_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/wp-json/wp/v2/pages/999999"))
        .respond_with(ResponseTemplate::new(404).set_body_json(json!({
            "code": "rest_post_invalid_id",
            "message": "Invalid post ID.",
            "data": {"status": 404}
        })))
        .mount(&server)
        .await;

    let client = client_for(&server);
    let err = client
        .request_raw("GET", "wp/v2/pages/999999", &[], None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, wpx_core::WpxError::NotFound { .. }),
        "{err:?}"
    );
}
