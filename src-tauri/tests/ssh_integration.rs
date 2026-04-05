//! SSH integration tests — require Docker SSH server on localhost:2222.
//!
//! Run: `docker compose -f tests/docker-compose.yml up -d`
//! Then: `cargo test --test ssh_integration`

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const SSH_HOST: &str = "127.0.0.1";
const SSH_PORT: u16 = 2222;
const SSH_USER: &str = "testuser";
const SSH_PASS: &str = "testpass123";
const ECHO_PORT: u16 = 9999;

/// Check if Docker SSH server is reachable.
async fn ssh_server_available() -> bool {
    tokio::time::timeout(
        Duration::from_secs(2),
        TcpStream::connect(format!("{}:{}", SSH_HOST, SSH_PORT)),
    )
    .await
    .is_ok()
}

/// Macro to skip test if SSH server is not available.
macro_rules! require_ssh_server {
    () => {
        if !ssh_server_available().await {
            eprintln!("SKIP: Docker SSH server not available on {}:{}", SSH_HOST, SSH_PORT);
            return;
        }
    };
}

// ── SSH client handler (same as in production code but standalone for tests) ──

struct TestHandler;

#[async_trait::async_trait]
impl russh::client::Handler for TestHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _key: &russh::keys::key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

// ── Tests ──

#[tokio::test]
async fn connect_with_password() {
    require_ssh_server!();

    let config = Arc::new(russh::client::Config::default());
    let addr = format!("{}:{}", SSH_HOST, SSH_PORT);
    let mut handle = russh::client::connect(config, &addr, TestHandler)
        .await
        .expect("Failed to connect");

    let auth_ok = handle
        .authenticate_password(SSH_USER, SSH_PASS)
        .await
        .expect("Auth call failed");
    assert!(auth_ok, "Password authentication should succeed");
}

#[tokio::test]
async fn connect_with_wrong_password_fails() {
    require_ssh_server!();

    let config = Arc::new(russh::client::Config::default());
    let addr = format!("{}:{}", SSH_HOST, SSH_PORT);
    let mut handle = russh::client::connect(config, &addr, TestHandler)
        .await
        .expect("Failed to connect");

    let auth_ok = handle
        .authenticate_password(SSH_USER, "wrong_password")
        .await
        .expect("Auth call failed");
    assert!(!auth_ok, "Wrong password should be rejected");
}

#[tokio::test]
async fn connect_with_key_file() {
    require_ssh_server!();

    let key_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/fixtures/test_key");
    let key_data = tokio::fs::read_to_string(key_path)
        .await
        .expect("Failed to read test key");
    let key = russh_keys::decode_secret_key(&key_data, None).expect("Failed to decode key");

    let config = Arc::new(russh::client::Config::default());
    let addr = format!("{}:{}", SSH_HOST, SSH_PORT);
    let mut handle = russh::client::connect(config, &addr, TestHandler)
        .await
        .expect("Failed to connect");

    let auth_ok = handle
        .authenticate_publickey(SSH_USER, Arc::new(key))
        .await
        .expect("Auth call failed");
    assert!(auth_ok, "Key file authentication should succeed");
}

#[tokio::test]
async fn local_forward_echo() {
    require_ssh_server!();

    // Connect and authenticate
    let config = Arc::new(russh::client::Config::default());
    let addr = format!("{}:{}", SSH_HOST, SSH_PORT);
    let mut handle = russh::client::connect(config, &addr, TestHandler)
        .await
        .expect("Failed to connect");
    handle
        .authenticate_password(SSH_USER, SSH_PASS)
        .await
        .expect("Auth failed");

    // Open direct-tcpip channel to echo server inside Docker
    let channel = handle
        .channel_open_direct_tcpip("127.0.0.1", ECHO_PORT as u32, "127.0.0.1", 0)
        .await
        .expect("Failed to open channel");

    let mut stream = channel.into_stream();

    // Write test data
    let test_data = b"Hello SSH Tunnel!";
    stream.write_all(test_data).await.expect("Write failed");
    stream.flush().await.expect("Flush failed");

    // Give the echo server time to respond
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Read echo back
    let mut buf = vec![0u8; test_data.len()];
    let n = tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buf))
        .await
        .expect("Read timeout")
        .expect("Read failed");

    assert_eq!(&buf[..n], test_data);
}
