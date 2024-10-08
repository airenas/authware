use std::{env, time::Duration};

use reqwest::{Client, StatusCode};
use serde_json::json;
use tokio::time::sleep;

fn get_auth_service_url() -> String {
    env::var("AUTH_SERVICE_URL").expect("AUTH_SERVICE_URL not set")
}

fn create_client() -> Client {
    Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build reqwest client")
}

async fn get_session_id() -> String {
    let client = create_client();
    let url = format!("{}/login", get_auth_service_url());
    let payload = json!({
        "user": "admin",
        "pass": "admin"
    });

    let response = client
        .post(url)
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
                println!("Server is ready!");
                return;
            }
            Ok(_) => {
                println!("Server not ready yet, retrying...");
            }
            Err(e) => {
                println!("Error connecting to server: {:?}", e);
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    panic!("Server did not become ready within {} seconds", timeout_sec);
}

#[tokio::test]
async fn test_live() {
    wait_for_server_ready().await;
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
    wait_for_server_ready().await;
    get_session_id().await;
}

#[tokio::test]
async fn test_failed_login() {
    wait_for_server_ready().await;
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
    wait_for_server_ready().await;
    let token = get_session_id().await;
    let client = create_client();
    let url = get_auth_service_url();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_successful_auth_query() {
    wait_for_server_ready().await;
    let token = get_session_id().await;
    let client = create_client();
    let url = format!("{}?token={}", get_auth_service_url(), token);
    let response = client
        .get(&url)
        .header("x-forwarded-uri", url)
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
}

#[tokio::test]
async fn test_fail_auth() {
    wait_for_server_ready().await;
    let token = "olia";
    let client = create_client();
    let url = get_auth_service_url();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
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
    wait_for_server_ready().await;
    let token = get_session_id().await;
    let client = create_client();
    let url = format!("{}/logout", get_auth_service_url());
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let url = get_auth_service_url();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Failed to send request");
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
}
