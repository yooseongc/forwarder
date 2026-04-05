#!/bin/sh
# Start a simple TCP echo server on port 9999 for tunnel testing
socat TCP-LISTEN:9999,fork,reuseaddr EXEC:cat &

# Start SSH daemon in foreground
exec /usr/sbin/sshd -D -e
