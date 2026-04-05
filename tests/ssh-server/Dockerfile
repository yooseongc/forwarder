FROM alpine:latest
RUN apk add --no-cache openssh socat python3 && \
    ssh-keygen -A && \
    adduser -D testuser && \
    echo "testuser:testpass123" | chpasswd && \
    mkdir -p /home/testuser/.ssh && \
    chown testuser:testuser /home/testuser/.ssh && \
    chmod 700 /home/testuser/.ssh

# Configure sshd for testing
RUN sed -i 's/^AllowTcpForwarding.*/AllowTcpForwarding yes/' /etc/ssh/sshd_config && \
    sed -i 's/^GatewayPorts.*/GatewayPorts yes/' /etc/ssh/sshd_config && \
    sed -i 's/^X11Forwarding.*/X11Forwarding no/' /etc/ssh/sshd_config && \
    sed -i 's/^#PasswordAuthentication.*/PasswordAuthentication yes/' /etc/ssh/sshd_config && \
    echo "PermitTunnel yes" >> /etc/ssh/sshd_config

COPY authorized_keys /home/testuser/.ssh/authorized_keys
RUN chown testuser:testuser /home/testuser/.ssh/authorized_keys && \
    chmod 600 /home/testuser/.ssh/authorized_keys

# Simple web page for HTTP test
RUN mkdir -p /var/www && \
    echo '<!DOCTYPE html><html><body><h1>SSH Forwarder Test Server</h1><p>If you see this, your tunnel is working!</p></body></html>' > /var/www/index.html

COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 22 8080 9999
CMD ["/entrypoint.sh"]
