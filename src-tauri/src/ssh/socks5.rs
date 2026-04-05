use anyhow::Result;
use russh::client;
use std::net::Ipv6Addr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::session::{proxy_channel, ClientHandler};

const SOCKS_VERSION: u8 = 0x05;
const CMD_CONNECT: u8 = 0x01;
const ATYP_IPV4: u8 = 0x01;
const ATYP_DOMAIN: u8 = 0x03;
const ATYP_IPV6: u8 = 0x04;
const MAX_AUTH_METHODS: usize = 255;

const REPLY_SUCCESS: [u8; 10] = [0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0];
const REPLY_CONN_REFUSED: [u8; 10] = [0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0];

/// Handle a single SOCKS5 client connection.
pub async fn handle_client(
    handle: Arc<client::Handle<ClientHandler>>,
    mut stream: TcpStream,
) -> Result<()> {
    negotiate_auth(&mut stream).await?;
    let (host, port) = read_connect_request(&mut stream).await?;

    match handle
        .channel_open_direct_tcpip(&host, port as u32, "127.0.0.1", 0)
        .await
    {
        Ok(channel) => {
            stream.write_all(&REPLY_SUCCESS).await?;
            proxy_channel(channel, stream).await?;
        }
        Err(_) => {
            stream.write_all(&REPLY_CONN_REFUSED).await?;
        }
    }
    Ok(())
}

async fn negotiate_auth(stream: &mut TcpStream) -> Result<()> {
    let mut header = [0u8; 2];
    let n = stream.read(&mut header).await?;
    if n < 2 || header[0] != SOCKS_VERSION {
        anyhow::bail!("Not a SOCKS5 request");
    }
    let nmethods = header[1] as usize;
    if nmethods == 0 || nmethods > MAX_AUTH_METHODS {
        anyhow::bail!("Invalid SOCKS5 auth method count: {}", nmethods);
    }
    // Read method bytes into a dynamically-sized buffer (not fixed array)
    let mut methods = vec![0u8; nmethods];
    stream.read_exact(&mut methods).await?;
    // Reply: no auth required
    stream.write_all(&[SOCKS_VERSION, 0x00]).await?;
    Ok(())
}

async fn read_connect_request(stream: &mut TcpStream) -> Result<(String, u16)> {
    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    if header[0] != SOCKS_VERSION || header[1] != CMD_CONNECT {
        anyhow::bail!("Unsupported SOCKS5 command: {:#x}", header[1]);
    }

    let host = match header[3] {
        ATYP_IPV4 => {
            let mut addr = [0u8; 4];
            stream.read_exact(&mut addr).await?;
            format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3])
        }
        ATYP_DOMAIN => {
            let mut len_buf = [0u8; 1];
            stream.read_exact(&mut len_buf).await?;
            let len = len_buf[0] as usize;
            if len == 0 {
                anyhow::bail!("Empty domain name in SOCKS5 request");
            }
            let mut domain = vec![0u8; len];
            stream.read_exact(&mut domain).await?;
            String::from_utf8(domain)?
        }
        ATYP_IPV6 => {
            let mut addr = [0u8; 16];
            stream.read_exact(&mut addr).await?;
            Ipv6Addr::from(addr).to_string()
        }
        _ => anyhow::bail!("Unsupported SOCKS5 address type: {:#x}", header[3]),
    };

    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;
    let port = u16::from_be_bytes(port_buf);

    Ok((host, port))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    /// Helper: create connected TcpStream pair via localhost listener.
    async fn tcp_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let client = TcpStream::connect(addr).await.unwrap();
        let (server, _) = listener.accept().await.unwrap();
        (client, server)
    }

    #[tokio::test]
    async fn negotiate_auth_valid() {
        let (mut client, mut server) = tcp_pair().await;
        // Client sends: version=5, nmethods=1, method=0 (no auth)
        tokio::spawn(async move {
            client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
            // Read reply
            let mut reply = [0u8; 2];
            client.read_exact(&mut reply).await.unwrap();
            assert_eq!(reply, [0x05, 0x00]);
        });
        negotiate_auth(&mut server).await.unwrap();
    }

    #[tokio::test]
    async fn negotiate_auth_invalid_version() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            client.write_all(&[0x04, 0x01, 0x00]).await.unwrap();
        });
        let result = negotiate_auth(&mut server).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not a SOCKS5"));
    }

    #[tokio::test]
    async fn negotiate_auth_zero_methods() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            client.write_all(&[0x05, 0x00]).await.unwrap();
        });
        let result = negotiate_auth(&mut server).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("method count"));
    }

    #[tokio::test]
    async fn read_connect_ipv4() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            // CONNECT to 192.168.1.1:80
            let mut packet = vec![0x05, 0x01, 0x00, 0x01]; // ver, cmd=CONNECT, rsv, atyp=IPv4
            packet.extend_from_slice(&[192, 168, 1, 1]); // addr
            packet.extend_from_slice(&80u16.to_be_bytes()); // port
            client.write_all(&packet).await.unwrap();
        });
        let (host, port) = read_connect_request(&mut server).await.unwrap();
        assert_eq!(host, "192.168.1.1");
        assert_eq!(port, 80);
    }

    #[tokio::test]
    async fn read_connect_domain() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            let domain = b"example.com";
            let mut packet = vec![0x05, 0x01, 0x00, 0x03]; // atyp=Domain
            packet.push(domain.len() as u8);
            packet.extend_from_slice(domain);
            packet.extend_from_slice(&443u16.to_be_bytes());
            client.write_all(&packet).await.unwrap();
        });
        let (host, port) = read_connect_request(&mut server).await.unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 443);
    }

    #[tokio::test]
    async fn read_connect_ipv6() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            let mut packet = vec![0x05, 0x01, 0x00, 0x04]; // atyp=IPv6
            // ::1
            let mut addr = [0u8; 16];
            addr[15] = 1;
            packet.extend_from_slice(&addr);
            packet.extend_from_slice(&8080u16.to_be_bytes());
            client.write_all(&packet).await.unwrap();
        });
        let (host, port) = read_connect_request(&mut server).await.unwrap();
        assert_eq!(host, "::1");
        assert_eq!(port, 8080);
    }

    #[tokio::test]
    async fn read_connect_empty_domain_error() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            let packet = vec![0x05, 0x01, 0x00, 0x03, 0x00]; // domain len=0
            client.write_all(&packet).await.unwrap();
        });
        let result = read_connect_request(&mut server).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty domain"));
    }

    #[tokio::test]
    async fn read_connect_unsupported_atyp() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            let packet = vec![0x05, 0x01, 0x00, 0xFF]; // invalid atyp
            client.write_all(&packet).await.unwrap();
        });
        let result = read_connect_request(&mut server).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported"));
    }

    #[tokio::test]
    async fn read_connect_unsupported_command() {
        let (mut client, mut server) = tcp_pair().await;
        tokio::spawn(async move {
            let packet = vec![0x05, 0x02, 0x00, 0x01]; // cmd=BIND (not CONNECT)
            client.write_all(&packet).await.unwrap();
        });
        let result = read_connect_request(&mut server).await;
        assert!(result.is_err());
    }
}
