#!/bin/sh

# TCP echo server on port 9999 (for raw tunnel testing)
socat TCP-LISTEN:9999,fork,reuseaddr EXEC:cat &

# HTTP server on port 8080 (for Local forward + SOCKS5 testing)
cd /var/www && python3 -m http.server 8080 &

echo "=== Test services started ==="
echo "  Echo server: port 9999"
echo "  HTTP server: port 8080 (http://127.0.0.1:8080)"
echo "  SSH server:  port 22"

# Start SSH daemon in foreground
exec /usr/sbin/sshd -D -e
