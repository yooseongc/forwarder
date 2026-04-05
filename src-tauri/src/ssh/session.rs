use anyhow::{Context, Result};
use async_trait::async_trait;
use russh::client;
use russh::keys::key;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::config::types::{AuthMethod, ConnectionProfile, ForwardingKind, ForwardingRule};
use crate::credential;
use crate::state::TunnelError;

use super::socks5;

// ── Client handler with host key + reverse forwarding ──

/// Shared state for reverse forwarding: maps (address, port) → local target.
type ReverseMap = Arc<Mutex<HashMap<(String, u32), (String, u16)>>>;

pub(crate) struct ClientHandler {
    /// Maps remote (bind_address, bind_port) to local (target_host, target_port)
    reverse_targets: ReverseMap,
}

impl ClientHandler {
    fn new(reverse_targets: ReverseMap) -> Self {
        Self { reverse_targets }
    }
}

#[async_trait]
impl client::Handler for ClientHandler {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TOFU (Trust On First Use) policy — accept all keys.
        // TODO: implement known_hosts file checking (%APPDATA%/forwarder/known_hosts)
        log::warn!("Accepting SSH host key (TOFU policy)");
        Ok(true)
    }

    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        channel: russh::Channel<client::Msg>,
        connected_address: &str,
        connected_port: u32,
        _originator_address: &str,
        _originator_port: u32,
        _session: &mut client::Session,
    ) -> Result<(), Self::Error> {
        let targets = self.reverse_targets.lock().await;
        let key = (connected_address.to_string(), connected_port);
        if let Some((local_host, local_port)) = targets.get(&key) {
            let local_addr = format!("{}:{}", local_host, local_port);
            let local_addr_clone = local_addr.clone();
            tokio::spawn(async move {
                match TcpStream::connect(&local_addr_clone).await {
                    Ok(stream) => {
                        if let Err(e) = proxy_channel(channel, stream).await {
                            log::debug!("Reverse forward proxy ended: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to connect to local target {}: {}", local_addr_clone, e);
                    }
                }
            });
        } else {
            log::warn!(
                "Received forwarded connection for unknown target {}:{}",
                connected_address,
                connected_port
            );
        }
        Ok(())
    }
}

// ── SSH Session ──

pub struct SshSession {
    handle: Arc<client::Handle<ClientHandler>>,
    tunnel_tasks: Vec<tokio::task::JoinHandle<()>>,
    cancel: CancellationToken,
}

impl SshSession {
    pub async fn connect(
        profile: &ConnectionProfile,
        remote_rules: &[ForwardingRule],
    ) -> Result<Self> {
        let config = Arc::new(client::Config::default());

        // Build reverse forwarding target map
        let reverse_targets: ReverseMap = Arc::new(Mutex::new(HashMap::new()));
        for rule in remote_rules.iter().filter(|r| r.enabled && r.kind == ForwardingKind::Remote) {
            reverse_targets.lock().await.insert(
                (rule.bind_address.clone(), rule.bind_port as u32),
                (rule.remote_host.clone(), rule.remote_port),
            );
        }

        let handler = ClientHandler::new(reverse_targets);
        let addr = format!("{}:{}", profile.host, profile.port);

        let mut handle = client::connect(config, &addr, handler)
            .await
            .context(format!("Failed to connect to {}", addr))?;

        authenticate(&mut handle, profile).await?;

        // Register remote forwarding BEFORE wrapping in Arc (requires &mut)
        for rule in remote_rules.iter().filter(|r| r.enabled && r.kind == ForwardingKind::Remote) {
            let port = handle
                .tcpip_forward(&rule.bind_address, rule.bind_port as u32)
                .await
                .context(format!(
                    "Failed to request remote forwarding {}:{}",
                    rule.bind_address, rule.bind_port
                ))?;
            log::info!(
                "Remote forward registered: {}:{} (assigned port {}) -> {}:{}",
                rule.bind_address,
                rule.bind_port,
                port,
                rule.remote_host,
                rule.remote_port
            );
        }

        Ok(Self {
            handle: Arc::new(handle),
            tunnel_tasks: Vec::new(),
            cancel: CancellationToken::new(),
        })
    }

    pub async fn start_tunnels(
        &mut self,
        rules: &[ForwardingRule],
        tunnel_errors: Arc<Mutex<Vec<TunnelError>>>,
    ) -> Result<()> {
        for rule in rules.iter().filter(|r| r.enabled) {
            self.spawn_tunnel(rule, tunnel_errors.clone());
        }
        Ok(())
    }

    /// Start a single additional tunnel at runtime.
    pub async fn start_single_tunnel(
        &mut self,
        rule: &ForwardingRule,
        tunnel_errors: Arc<Mutex<Vec<TunnelError>>>,
    ) -> Result<()> {
        self.spawn_tunnel(rule, tunnel_errors);
        Ok(())
    }

    /// Spawn a single tunnel task and track its handle.
    fn spawn_tunnel(&mut self, rule: &ForwardingRule, tunnel_errors: Arc<Mutex<Vec<TunnelError>>>) {
        let handle = self.handle.clone();
        let rule = rule.clone();
        let cancel = self.cancel.clone();

        let task = tokio::spawn(async move {
            let rule_id = rule.id.clone();
            let result = match rule.kind {
                ForwardingKind::Local => run_local_forward(handle, &rule, cancel).await,
                ForwardingKind::Remote => run_remote_keepalive(handle, &rule, cancel).await,
                ForwardingKind::Dynamic => run_dynamic_forward(handle, &rule, cancel).await,
            };
            if let Err(e) = result {
                log::error!("Tunnel error ({}): {}", rule_id, e);
                let mut errs = tunnel_errors.lock().await;
                if let Some(entry) = errs.iter_mut().find(|t| t.rule_id == rule_id) {
                    entry.message = Some(e.to_string());
                }
            }
        });
        self.tunnel_tasks.push(task);
    }

    /// Start a health check task that monitors the SSH connection.
    /// Returns a JoinHandle that resolves when the connection drops.
    pub fn start_health_check(&self) -> tokio::task::JoinHandle<()> {
        let handle = self.handle.clone();
        let cancel = self.cancel.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => return,
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                        if handle.is_closed() {
                            log::warn!("SSH connection health check: connection lost");
                            return;
                        }
                    }
                }
            }
        })
    }

    pub fn disconnect(&mut self) {
        self.cancel.cancel();
        for task in self.tunnel_tasks.drain(..) {
            task.abort();
        }
    }
}

impl Drop for SshSession {
    fn drop(&mut self) {
        self.disconnect();
    }
}

// ── Authentication ──

async fn authenticate(
    handle: &mut client::Handle<ClientHandler>,
    profile: &ConnectionProfile,
) -> Result<()> {
    let auth_ok = match &profile.auth_method {
        AuthMethod::Password => {
            let password = credential::get_password(&profile.id)?
                .context("No password saved for this profile")?;
            handle
                .authenticate_password(&profile.username, &password)
                .await?
        }
        AuthMethod::KeyFile { path } => {
            let key = load_key_pair(path, None).await?;
            handle
                .authenticate_publickey(&profile.username, Arc::new(key))
                .await?
        }
        AuthMethod::KeyFileWithPassphrase { path } => {
            let passphrase = credential::get_password(&profile.id)?
                .context("No passphrase saved for this profile")?;
            let key = load_key_pair(path, Some(&passphrase)).await?;
            handle
                .authenticate_publickey(&profile.username, Arc::new(key))
                .await?
        }
    };
    if !auth_ok {
        anyhow::bail!("Authentication failed");
    }
    Ok(())
}

async fn load_key_pair(path: &str, passphrase: Option<&str>) -> Result<key::KeyPair> {
    let key_data = tokio::fs::read_to_string(path)
        .await
        .context(format!("Failed to read key file: {}", path))?;
    Ok(russh_keys::decode_secret_key(&key_data, passphrase)?)
}

// ── Local forwarding ──

async fn run_local_forward(
    handle: Arc<client::Handle<ClientHandler>>,
    rule: &ForwardingRule,
    cancel: CancellationToken,
) -> Result<()> {
    let bind = format!("{}:{}", rule.bind_address, rule.bind_port);
    let listener = TcpListener::bind(&bind)
        .await
        .context(format!("Failed to bind {}", bind))?;

    log::info!("Local forward: {} -> {}:{}", bind, rule.remote_host, rule.remote_port);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                log::info!("Local forward stopped: {}", bind);
                return Ok(());
            }
            result = listener.accept() => {
                let (local_stream, _) = result?;
                let handle = handle.clone();
                let remote_host = rule.remote_host.clone();
                let remote_port = rule.remote_port;

                tokio::spawn(async move {
                    match handle
                        .channel_open_direct_tcpip(&remote_host, remote_port as u32, "127.0.0.1", 0)
                        .await
                    {
                        Ok(channel) => {
                            if let Err(e) = proxy_channel(channel, local_stream).await {
                                log::debug!("Local forward proxy ended: {}", e);
                            }
                        }
                        Err(e) => log::error!("Failed to open direct-tcpip channel: {}", e),
                    }
                });
            }
        }
    }
}

// ── Remote forwarding (keep-alive monitor) ──

/// The actual tcpip_forward is registered during connect().
/// This task just monitors the connection and the cancellation token.
async fn run_remote_keepalive(
    handle: Arc<client::Handle<ClientHandler>>,
    rule: &ForwardingRule,
    cancel: CancellationToken,
) -> Result<()> {
    log::info!(
        "Remote forward active: {}:{} -> {}:{}",
        rule.bind_address, rule.bind_port, rule.remote_host, rule.remote_port
    );

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                // Cancel the remote forwarding on the server
                let _ = handle
                    .cancel_tcpip_forward(&rule.bind_address, rule.bind_port as u32)
                    .await;
                log::info!("Remote forward stopped: {}:{}", rule.bind_address, rule.bind_port);
                return Ok(());
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                if handle.is_closed() {
                    anyhow::bail!("SSH connection closed");
                }
            }
        }
    }
}

// ── Dynamic forwarding (SOCKS5) ──

async fn run_dynamic_forward(
    handle: Arc<client::Handle<ClientHandler>>,
    rule: &ForwardingRule,
    cancel: CancellationToken,
) -> Result<()> {
    let bind = format!("{}:{}", rule.bind_address, rule.bind_port);
    let listener = TcpListener::bind(&bind)
        .await
        .context(format!("Failed to bind SOCKS5 proxy on {}", bind))?;

    log::info!("Dynamic forward (SOCKS5): {}", bind);

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                log::info!("Dynamic forward stopped: {}", bind);
                return Ok(());
            }
            result = listener.accept() => {
                let (stream, _) = result?;
                let handle = handle.clone();
                tokio::spawn(async move {
                    if let Err(e) = socks5::handle_client(handle.clone(), stream).await {
                        log::debug!("SOCKS5 session ended: {}", e);
                    }
                });
            }
        }
    }
}

// ── Channel proxy ──

/// Bidirectional proxy between an SSH channel and a TCP stream.
pub(crate) async fn proxy_channel(
    channel: russh::Channel<client::Msg>,
    mut stream: TcpStream,
) -> Result<()> {
    let channel_stream = channel.into_stream();
    let (mut chan_reader, mut chan_writer) = tokio::io::split(channel_stream);
    let (mut tcp_reader, mut tcp_writer) = stream.split();

    let up = tokio::io::copy(&mut tcp_reader, &mut chan_writer);
    let down = tokio::io::copy(&mut chan_reader, &mut tcp_writer);

    tokio::select! {
        r = up => { r?; }
        r = down => { r?; }
    }
    Ok(())
}
