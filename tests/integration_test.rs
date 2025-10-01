use std::{env, time::Duration};

use base64::Engine;
use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::{sync::OnceCell, time::sleep};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT: OnceCell<()> = OnceCell::const_new();
const IP_HEADER_KEY: &str = "x-forwarded-for";

async fn init_wait_for_ready() {
    INIT.get_or_init(|| async {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::Layer::default().compact())
            .init();

        wait_for_server_ready().await;
    })
    .await;

    tracing::info!("Test init is done!");
}

fn get_auth_service_url() -> String {
    env::var("AUTH_SERVICE_URL").expect("AUTH_SERVICE_URL not set")
}

fn create_client() -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build reqwest client")
}

fn make_ip_header(ip: &str) -> String {
    let mut ips = vec!["1.1.1.1"];
    if !ip.is_empty() {
        ips.insert(0, ip);
    }
    ips.join(",")
}

async fn get_session_id(ip: &str) -> String {
    let client = create_client();
    let url = format!("{}/login", get_auth_service_url());
    let payload = json!({
        "user": "admin",
        "pass": "admin"
    });

    let response = client
        .post(url)
        .header(IP_HEADER_KEY, make_ip_header(ip))
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("session_id").is_some());
    body["session_id"].as_str().unwrap().to_string()
}

async fn wait_for_server_ready() {
    let timeout_sec = 30;
    let url = format!("{}/live", get_auth_service_url());

    let client = create_client(); // Use the custom client
    let timeout = Duration::from_secs(timeout_sec);
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < timeout {
        let response = client.get(&url).send().await;

        match response {
            Ok(resp) if resp.status() == StatusCode::OK => {
                tracing::info!("Server is ready!");
                return;
            }
            Ok(_) => {
                tracing::info!("Server not ready yet, retrying...");
            }
            Err(e) => {
                tracing::info!("Error connecting to server: {:?}", e);
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    panic!("Server did not become ready within {timeout_sec} seconds");
}

async fn logout(client: &Client, token: &str) {
    let url = format!("{}/logout", get_auth_service_url());
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_live() {
    init_wait_for_ready().await;
    let client = create_client();
    let url = format!("{}/live", get_auth_service_url());
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_successful_login() {
    init_wait_for_ready().await;
    get_session_id("").await;
}

#[tokio::test]
async fn test_failed_login() {
    init_wait_for_ready().await;
    let client = create_client();
    let url = format!("{}/login", get_auth_service_url());
    let payload = json!({
        "user": "admin",
        "pass": "admin_wrong"
    });

    let response = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_successful_auth() {
    init_wait_for_ready().await;
    let token = get_session_id("").await;
    let client = create_client();
    let url = get_auth_service_url();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_failed_validate() {
    init_wait_for_ready().await;
    let token = get_session_id("").await;
    let client = create_client();
    logout(&client, &token).await;
    let url = format!("{}/validate", get_auth_service_url());
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_successful_validate() {
    init_wait_for_ready().await;
    let token = get_session_id("").await;
    let client = create_client();
    let url = format!("{}/validate", get_auth_service_url());
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_successful_auth_query() {
    init_wait_for_ready().await;
    let token = get_session_id("").await;
    let client = create_client();
    let url = format!("{}?token={}", get_auth_service_url(), token);
    let response = client
        .get(&url)
        .header("x-forwarded-uri", url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let header = response
        .headers()
        .get("User-Info")
        .expect("No User-Info header");
    let decoded = base64::prelude::BASE64_STANDARD
        .decode(header.as_bytes())
        .expect("Failed to decode User-Info header");
    let user_info: serde_json::Value =
        serde_json::from_slice(&decoded).expect("Failed to parse user info from User-Info header");
    tracing::info!("User info: {:?}", user_info);
    assert_eq!(user_info.get("id").unwrap(), "admin");
    assert!(user_info.get("roles").is_some());
    assert_eq!(user_info.get("department").unwrap(), "IT dep of admin");
    assert_eq!(user_info.get("name").unwrap(), "admin");
}

#[tokio::test]
async fn test_fail_auth() {
    init_wait_for_ready().await;
    let token = "olia";
    let client = create_client();
    let url = get_auth_service_url();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    let response = client
        .get(&url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout() {
    init_wait_for_ready().await;
    let token = get_session_id("").await;
    let client = create_client();
    logout(&client, &token).await;
    let url = get_auth_service_url();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_successful_auth_ip() {
    init_wait_for_ready().await;
    let ip = "2.2.2.2";
    let token = get_session_id(ip).await;
    let client = create_client();
    let url = get_auth_service_url();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header(IP_HEADER_KEY, make_ip_header(ip))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    tracing::info!("Test successful_auth_ip passed");
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    tracing::info!("Test no ip passed");
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .header(IP_HEADER_KEY, make_ip_header("any"))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    tracing::info!("Test wrong ip passed");
}
