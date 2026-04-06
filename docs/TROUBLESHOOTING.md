# Troubleshooting Guide

## Error Codes

### AUTH_FAILED
**Meaning**: SSH authentication failed.

**Common causes**:
- Wrong password or passphrase
- Key file doesn't match the server's authorized_keys
- Server doesn't allow the selected auth method (e.g., password auth disabled)

**Solutions**:
1. Verify the password/passphrase is correct
2. For key files: ensure you're selecting the **private key** (not the `.pub` file)
3. Supported key formats: OpenSSH, PEM, PuTTY PPK v2/v3
4. Check the server's `sshd_config` for `PasswordAuthentication` or `PubkeyAuthentication`

---

### CONNECTION_FAILED
**Meaning**: Could not establish an SSH connection to the server.

**Common causes**:
- Server is unreachable (wrong host/port, firewall)
- SSH service not running on the server
- DNS resolution failed
- Network timeout

**Solutions**:
1. Use the **Ping** button to test basic connectivity
2. Verify the host address and port (default SSH port: 22)
3. Check firewall rules on both local and remote sides
4. Try connecting with a standard SSH client (e.g., `ssh user@host -p port`)

---

### HOST_KEY_MISMATCH
**Meaning**: The server's SSH host key has changed since the last connection.

**Common causes**:
- Server was reinstalled or its SSH keys were regenerated
- Potential man-in-the-middle (MITM) attack

**Solutions**:
1. If you **trust** the server (e.g., it was reinstalled): click **Reset Host Key** in the error message
2. If unsure: verify the server's fingerprint with the administrator
3. To reset all saved host keys: Settings > Security > **Reset All Host Keys**

**Host key file location**: `%APPDATA%/forwarder/known_hosts`

---

### TUNNEL_BIND_FAILED
**Meaning**: Could not bind to the specified local port for forwarding.

**Common causes**:
- Port is already in use by another application
- Insufficient permissions (ports below 1024 may require admin)

**Solutions**:
1. Check if the port is in use: `netstat -ano | findstr :PORT`
2. Choose a different local port
3. Close the application using the port, then retry

---

### CONFIG_ERROR
**Meaning**: Configuration file is corrupted or unreadable.

**Common causes**:
- Config JSON was manually edited with invalid syntax
- File permissions changed
- Disk corruption

**Solutions**:
1. The app automatically backs up to `config.json.bak` on corruption
2. Config file location: `%APPDATA%/forwarder/config.json`
3. To fully reset: delete `config.json` and restart the app
4. To restore from backup: rename `config.json.bak` to `config.json`

---

### CREDENTIAL_ERROR
**Meaning**: Could not access Windows Credential Manager.

**Common causes**:
- Credential Manager service is not running
- Insufficient permissions

**Solutions**:
1. Open Windows Services (`services.msc`) and check "Credential Manager" is running
2. Try saving the credential again
3. As a workaround: re-enter the password each time instead of saving

---

## Common Issues

### Connection times out
- Verify the host and port are correct
- Check if a firewall or VPN is blocking the connection
- Try the Ping button for basic TCP connectivity test
- Default timeout is 5 seconds

### Tunnels show "active" but traffic doesn't flow
- For **Local** forwarding: ensure the remote target (host:port) is accessible from the SSH server
- For **Remote** forwarding: ensure the SSH server allows `GatewayPorts` if binding to non-localhost
- For **Dynamic** (SOCKS5): configure your browser/application to use the SOCKS5 proxy address

### Remote forwarding rules can't be enabled at runtime
Remote forwarding requires server-side registration (`tcpip-forward`) during the initial SSH connection. To change remote forwarding rules:
1. Disconnect the profile
2. Edit the forwarding rules
3. Reconnect

### App doesn't start (single instance)
SSH Forwarder enforces single-instance mode. If the app appears stuck:
1. Check the system tray for an existing instance
2. If not visible: open Task Manager and end `forwarder.exe`
3. Restart the app

### Auto-reconnect doesn't work
- Ensure **Auto-reconnect on disconnect** is enabled in the profile settings
- Auto-reconnect attempts up to 5 times with exponential backoff (1s → 30s)
- If all attempts fail, the status shows "Auto-reconnect failed after 5 attempts"
- Manual disconnect cancels any ongoing auto-reconnect

---

## File Locations

| File | Path |
|------|------|
| Config | `%APPDATA%/forwarder/config.json` |
| Config backup | `%APPDATA%/forwarder/config.json.bak` |
| Known hosts | `%APPDATA%/forwarder/known_hosts` |
| Credentials | Windows Credential Manager (service: `ssh-forwarder`) |

---

## Supported Key Formats

| Format | Extension | Header |
|--------|-----------|--------|
| OpenSSH | (various) | `-----BEGIN OPENSSH PRIVATE KEY-----` |
| RSA PEM | `.pem` | `-----BEGIN RSA PRIVATE KEY-----` |
| PKCS#8 | `.pem` | `-----BEGIN PRIVATE KEY-----` |
| EC PEM (SEC1) | `.pem` | `-----BEGIN EC PRIVATE KEY-----` |
| PuTTY PPK v2 | `.ppk` | `PuTTY-User-Key-File-2:` |
| PuTTY PPK v3 | `.ppk` | `PuTTY-User-Key-File-3:` |

**Not supported**: Public key files (`.pub`, SSH2 format). If you see "The selected file is a public key", select the corresponding **private** key file instead.
