# Eryndor MMO - Production Deployment Guide

Complete step-by-step guide for deploying Eryndor MMO to DigitalOcean (server) and Cloudflare Pages (web client).

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Phase 1: Domain Setup](#phase-1-domain-setup)
4. [Phase 2: Server Infrastructure](#phase-2-server-infrastructure)
5. [Phase 3: Server Deployment](#phase-3-server-deployment)
6. [Phase 4: Client Deployment](#phase-4-client-deployment)
7. [Phase 5: Verification](#phase-5-verification)
8. [Operations](#operations)
9. [Troubleshooting](#troubleshooting)

---

## Overview

**Architecture:**
```
┌──────────────────┐         HTTPS/WSS         ┌────────────────────┐
│ Cloudflare Pages │ ←──────────────────────→ │  DigitalOcean      │
│  (WASM Client)   │                           │  Droplet (Server)  │
│                  │                           │  ├─ nginx (SSL)    │
│  Free CDN        │                           │  ├─ Game Server    │
│  Global Edge     │                           │  └─ SQLite DB      │
└──────────────────┘                           └────────────────────┘
```

**Cost:** ~$13/month (DigitalOcean $12 + Domain $1/month)

---

## Prerequisites

**Local Machine:**
- Rust 1.70+ (`cargo --version`)
- wasm32 target: `rustup target add wasm32-unknown-unknown`
- trunk: `cargo install trunk`
- Git

**Accounts:**
- DigitalOcean account
- Domain registrar (Namecheap, GoDaddy, etc.)
- Cloudflare account

---

## Phase 1: Domain Setup

### 1.1 Register Domain
Register a domain like `yourgame.com` through any registrar.

### 1.2 Configure DNS

**For Game Server:**
Point your domain to DigitalOcean droplet:
```
Type    Name    Value                 TTL
A       game    <droplet-ip>          3600
A       @       <droplet-ip>          3600
```

**For Web Client:**
Will be configured via Cloudflare Pages (automatic).

---

## Phase 2: Server Infrastructure

### 2.1 Create DigitalOcean Droplet

**Recommended specs:**
- Size: Basic - $12/month (2GB RAM, 1 vCPU)
- Image: Ubuntu 24.04 LTS
- Region: Closest to your players
- Auth: SSH keys

```bash
# Via web console or CLI:
doctl compute droplet create eryndor-server \
  --image ubuntu-24-04-x64 \
  --size s-1vcpu-2gb \
  --region nyc1 \
  --ssh-keys <your-key-fingerprint>
```

Note the droplet IP for DNS setup.

### 2.2 Initial Server Setup

SSH into droplet:
```bash
ssh root@<droplet-ip>
```

**Update system:**
```bash
apt update && apt upgrade -y
```

**Create user:**
```bash
adduser eryndor
usermod -aG sudo eryndor
```

**Configure firewall:**
```bash
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 5003/tcp
ufw enable
```

---

## Phase 3: Server Deployment

### 3.1 Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install nginx & certbot
sudo apt install nginx certbot python3-certbot-nginx -y

# Install build tools
sudo apt install build-essential pkg-config libssl-dev -y
```

### 3.2 Build Server

**On local machine:**
```bash
cargo build --release --bin server
```

**Transfer to server:**
```bash
# Create directories
ssh eryndor@<droplet-ip> "sudo mkdir -p /opt/eryndor /var/lib/eryndor /etc/eryndor && sudo chown -R eryndor:eryndor /opt/eryndor /var/lib/eryndor /etc/eryndor"

# Upload binary
scp target/release/server eryndor@<droplet-ip>:/opt/eryndor/

# Make executable
ssh eryndor@<droplet-ip> "chmod +x /opt/eryndor/server"
```

### 3.3 Configure Environment

```bash
ssh eryndor@<droplet-ip>
nano /etc/eryndor/.env
```

**Production .env:**
```bash
SERVER_ADDR=0.0.0.0
SERVER_PORT=5001
SERVER_PORT_WEBSOCKET=5003
SERVER_PORT_WEBTRANSPORT=5002
SERVER_CERT_PORT=8080

# CRITICAL: Change these!
JWT_SECRET=<generate-with-openssl-rand-hex-64>
GOOGLE_CLIENT_ID=your-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-secret

DATABASE_PATH=/var/lib/eryndor/eryndor.db
RUST_LOG=info
```

Generate JWT secret:
```bash
openssl rand -hex 64
```

### 3.4 Setup systemd Service

```bash
sudo cp deploy/eryndor.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable eryndor
sudo systemctl start eryndor
sudo systemctl status eryndor
```

### 3.5 Setup SSL

```bash
sudo certbot certonly --nginx -d game.yourdomain.com
```

### 3.6 Configure nginx

```bash
sudo cp deploy/nginx.conf /etc/nginx/sites-available/eryndor

# Edit file - replace YOURDOMAIN.COM
sudo nano /etc/nginx/sites-available/eryndor

# Enable
sudo ln -s /etc/nginx/sites-available/eryndor /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

## Phase 4: Client Deployment

### 4.1 Build WASM Client

```bash
cd crates/eryndor_client

# Set production WebSocket URL
export SERVER_WS_URL="wss://game.yourdomain.com/ws"

# Build
trunk build --release

# Output: dist/ directory
```

### 4.2 Deploy to Cloudflare Pages

**Via Dashboard:**
1. Login to https://dash.cloudflare.com
2. Navigate to "Pages" → "Create a project"
3. Connect Git repo or use Direct Upload
4. Configure build:
   ```
   Build command: cd crates/eryndor_client && trunk build --release
   Build output: crates/eryndor_client/dist
   Env: SERVER_WS_URL=wss://game.yourdomain.com/ws
   ```
5. Deploy & configure custom domain

**Via CLI:**
```bash
npm install -g wrangler
wrangler login
cd crates/eryndor_client
SERVER_WS_URL="wss://game.yourdomain.com/ws" trunk build --release
wrangler pages deploy dist --project-name=eryndor-game
```

---

## Phase 5: Verification

### 5.1 Test Connection

1. Open `https://yourdomain.com`
2. Check browser console (F12)
3. Look for "Connecting via WebSocket to wss://game.yourdomain.com/ws"
4. Create account & character
5. Test gameplay

### 5.2 Monitor Logs

```bash
# Server logs
sudo journalctl -u eryndor -f

# nginx logs
sudo tail -f /var/log/nginx/error.log
```

---

## Operations

### Backups

Create backup script:
```bash
sudo nano /opt/eryndor/backup.sh
```

```bash
#!/bin/bash
BACKUP_DIR="/var/backups/eryndor"
DB_PATH="/var/lib/eryndor/eryndor.db"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR
sqlite3 $DB_PATH ".backup '$BACKUP_DIR/eryndor_$DATE.db'"
find $BACKUP_DIR -name "eryndor_*.db" -mtime +7 -delete
```

```bash
sudo chmod +x /opt/eryndor/backup.sh
sudo crontab -e
# Add: 0 2 * * * /opt/eryndor/backup.sh
```

### Updates

**Server update:**
```bash
# Local: rebuild
cargo build --release --bin server

# Transfer & restart
scp target/release/server eryndor@<droplet-ip>:/tmp/
ssh eryndor@<droplet-ip>
sudo systemctl stop eryndor
sudo cp /tmp/server /opt/eryndor/server
sudo chmod +x /opt/eryndor/server
sudo systemctl start eryndor
```

**Client update:**
```bash
cd crates/eryndor_client
SERVER_WS_URL="wss://game.yourdomain.com/ws" trunk build --release
wrangler pages deploy dist --project-name=eryndor-game
```

---

## Troubleshooting

### Server Won't Start
```bash
sudo journalctl -u eryndor -n 50
```
Check:
- Environment variables in `/etc/eryndor/.env`
- Database permissions
- Port conflicts

### WebSocket Connection Fails
```bash
sudo nginx -t
sudo tail -f /var/log/nginx/error.log
```
Verify:
- SSL certificate: `sudo certbot certificates`
- Firewall: `sudo ufw status`

### Client Build Fails
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

---

## Security Checklist

- [ ] Changed JWT_SECRET
- [ ] SSL certificates installed
- [ ] Firewall enabled
- [ ] SSH password auth disabled
- [ ] Database backups scheduled
- [ ] Running as non-root
- [ ] .env not in git
- [ ] Database not in git

---

## Cost Breakdown

- DigitalOcean: $12/month
- Domain: ~$12/year (~$1/month)
- Cloudflare Pages: Free
- SSL: Free (Let's Encrypt)

**Total: ~$13/month**
