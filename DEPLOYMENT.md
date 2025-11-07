# Eryndor MMO Deployment Guide

Complete guide for deploying Eryndor MMO with browser support at $0-1/month using free cloud infrastructure.

## Table of Contents
- [Architecture Overview](#architecture-overview)
- [Quick Start Options](#quick-start-options)
- [Production Deployment](#production-deployment)
- [Code Changes Required](#code-changes-required)
- [Troubleshooting](#troubleshooting)
- [Cost Breakdown](#cost-breakdown)

---

## Architecture Overview

### Recommended Free Setup

```
┌─────────────────────────────────────────────────────────┐
│ Oracle Cloud Always Free VM (ARM, 4 cores, 24GB RAM)   │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────────────────────────────┐           │
│  │ Game Server (renet2)                     │           │
│  │  - UDP transport (port 5000)             │           │
│  │  - WebTransport server (port 5001)       │           │
│  │  - SQLite database                       │           │
│  └──────────────────────────────────────────┘           │
│                                                          │
│  ┌──────────────────────────────────────────┐           │
│  │ nginx + Let's Encrypt SSL                │           │
│  │  - Reverse proxy for WebTransport        │           │
│  │  - HTTPS on port 443                     │           │
│  └──────────────────────────────────────────┘           │
│                                                          │
└─────────────────────────────────────────────────────────┘
                          ↑
                          │
        ┌─────────────────┴─────────────────┐
        │                                    │
┌───────┴─────────┐               ┌─────────┴──────────┐
│ Native Clients  │               │ Browser Clients    │
│   (UDP)         │               │   (WebTransport)   │
│                 │               │                    │
│ Download from:  │               │ Served via:        │
│ itch.io or      │               │ Cloudflare Pages   │
│ GitHub Releases │               │ (Free CDN)         │
└─────────────────┘               └────────────────────┘
```

### Stack
- **Server**: Bevy 0.17 + renet2 + bevy_replicon
- **Transports**: UDP (native) + WebTransport (browser)
- **Database**: SQLite (good for 10-50 players)
- **Client Hosting**: Cloudflare Pages (WASM files)
- **Total Cost**: $0/month

---

## Quick Start Options

### Option A: Immediate Testing (No Cloud Setup)

**Best for**: Testing with 2-5 friends this week
**Cost**: $0
**Setup Time**: 30 minutes
**Code Changes**: Minimal (1 constant)

#### Using Tailscale Virtual LAN

1. **Install Tailscale** on your PC and testers' PCs
   - Download: https://tailscale.com/download
   - Sign up (free, supports 100 devices)

2. **Start Tailscale** and get your IP
   ```bash
   tailscale ip -4
   # Example output: 100.64.1.2
   ```

3. **Update server address** in `crates/eryndor_shared/src/constants.rs`:
   ```rust
   pub const SERVER_ADDR: &str = "100.64.1.2";  // Your Tailscale IP
   pub const SERVER_PORT: u16 = 5000;
   ```

4. **Build release binaries**:
   ```bash
   cargo build --release
   ```

5. **Run server** on your PC:
   ```bash
   ./target/release/server
   ```

6. **Distribute client** via itch.io or Google Drive:
   - Upload `target/release/client.exe` (Windows)
   - Upload `target/release/client` (Linux)

7. **Testers connect**:
   - Install Tailscale
   - Join your Tailscale network
   - Run the client binary

**Limitations**:
- Server only online when your PC is running
- Requires installing Tailscale on each tester's PC
- Not suitable for public access

---

### Option B: Player-Hosted Servers

**Best for**: Community-driven hosting
**Cost**: $0
**Setup Time**: 10 minutes
**Code Changes**: Make IP configurable

1. **Make server address configurable** (see [Code Changes](#code-changes-required))

2. **Distribute both** server and client binaries on itch.io

3. **Players can host** by:
   - Running the server binary
   - Port forwarding UDP 5000 on their router
   - Sharing their public IP with friends

**Limitations**:
- Requires port forwarding knowledge
- Server only runs when host is online
- Security concerns (IP exposure)

---

## Production Deployment

### Overview

This guide covers deploying a production-ready server with browser support using free infrastructure.

**Timeline**: 10-12 hours total
- Phase 1: renet2 migration (3-4 hours)
- Phase 2: WASM build setup (2-3 hours)
- Phase 3: Oracle Cloud setup (2-3 hours)
- Phase 4: Cloudflare Pages (30 minutes)

---

### Phase 1: Migrate to renet2 (3-4 hours)

#### Why renet2?

Your current `bevy_renet` (v3.0) only supports UDP (native clients). renet2 adds:
- WebTransport support for browser clients
- Multiple concurrent transports (UDP + WebTransport simultaneously)
- Same server code handles both native and browser clients

#### Step 1.1: Update Dependencies

**In workspace `Cargo.toml`**:
```toml
[workspace.dependencies]
# Replace these lines:
# bevy_renet = "0.0.15"
# bevy_replicon_renet = "0.5"

# With:
bevy_renet2 = "0.0.5"
bevy_replicon_renet2 = "0.5"
```

**In `crates/eryndor_server/Cargo.toml`**:
```toml
[dependencies]
bevy_renet2 = { workspace = true }
bevy_replicon_renet2 = { workspace = true }
```

**In `crates/eryndor_client/Cargo.toml`**:
```toml
[dependencies]
bevy_renet2 = { workspace = true }
bevy_replicon_renet2 = { workspace = true }

# For WASM builds, add:
[target.'cfg(target_family = "wasm")'.dependencies]
bevy_renet2 = { workspace = true, features = ["wt_client_transport", "ws_client_transport"] }

[target.'cfg(not(target_family = "wasm"))'.dependencies]
bevy_renet2 = { workspace = true, features = ["native_transport"] }
```

#### Step 1.2: Update Imports

**In `crates/eryndor_server/src/main.rs`**:
```rust
// OLD:
// use bevy_renet::renet::{RenetServer, ConnectionConfig};
// use bevy_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
// use bevy_replicon_renet::RepliconRenetPlugins;

// NEW:
use bevy_renet2::renet2::{RenetServer, ConnectionConfig};
use bevy_renet2::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_replicon_renet2::RepliconRenet2Plugins;
```

**In `crates/eryndor_client/src/main.rs`**:
```rust
// OLD:
// use bevy_replicon_renet::RepliconRenetPlugins;

// NEW:
use bevy_replicon_renet2::RepliconRenet2Plugins;
```

**In `crates/eryndor_client/src/game_state.rs`**:
```rust
// OLD:
// use bevy_renet::renet::{RenetClient, ConnectionConfig};
// use bevy_renet::netcode::{NetcodeClientTransport, ClientAuthentication};

// NEW:
use bevy_renet2::renet2::{RenetClient, ConnectionConfig};
use bevy_renet2::netcode::{NetcodeClientTransport, ClientAuthentication};
```

#### Step 1.3: Update Plugin Registration

**In both `crates/eryndor_server/src/main.rs` and `crates/eryndor_client/src/main.rs`**:
```rust
// OLD:
// .add_plugins(RepliconRenetPlugins)

// NEW:
.add_plugins(RepliconRenet2Plugins)
```

#### Step 1.4: Test Native Clients

```bash
# Build and test
cargo build --release
./target/release/server &
./target/release/client
```

Verify everything still works before proceeding to WASM.

---

### Phase 2: WASM Build Setup (2-3 hours)

#### Step 2.1: Install Prerequisites

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Install trunk (WASM build tool)
cargo install --locked trunk
```

#### Step 2.2: Create index.html

**Create `crates/eryndor_client/index.html`**:
```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Eryndor MMO</title>
    <style>
        body {
            margin: 0;
            padding: 0;
            background-color: #000;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            font-family: Arial, sans-serif;
        }
        canvas {
            display: block;
            max-width: 100%;
            max-height: 100%;
        }
        #loading {
            color: white;
            font-size: 24px;
        }
    </style>
</head>
<body>
    <div id="loading">Loading Eryndor...</div>
</body>
</html>
```

#### Step 2.3: Configure WASM Build

**Add to `crates/eryndor_client/Cargo.toml`**:
```toml
[target.wasm32-unknown-unknown]
# Optimize for size
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = "debuginfo"
```

#### Step 2.4: Build WASM Locally

```bash
cd crates/eryndor_client
trunk serve
# Open http://127.0.0.1:8080
```

For production build:
```bash
trunk build --release
# Output in crates/eryndor_client/dist/
```

#### Step 2.5: Test WebTransport Locally

⚠️ WebTransport requires HTTPS. For local testing:

1. Generate self-signed certificate:
```bash
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes
```

2. Configure server to use certificate (modify `main.rs`)

3. Access via `https://localhost:8080` (ignore certificate warning)

---

### Phase 3: Oracle Cloud Setup (2-3 hours)

#### Step 3.1: Create Oracle Cloud Account

1. Visit https://cloud.oracle.com/free
2. Sign up (requires credit card, but won't charge)
3. Verify email and phone
4. Complete account setup

**Note**: Signup can fail with "out of capacity" errors. Try:
- Different regions (Phoenix, Ashburn, Frankfurt)
- Different times of day
- AMD instances instead of ARM (if ARM unavailable)

#### Step 3.2: Provision VM

1. Go to **Compute** > **Instances** > **Create Instance**

2. **Choose ARM** (Always Free eligible):
   - Shape: VM.Standard.A1.Flex
   - Cores: 4 (use all available)
   - RAM: 24 GB (use all available)

3. **OS Image**: Ubuntu 22.04 Minimal

4. **Networking**:
   - Create new VCN (Virtual Cloud Network)
   - Assign public IPv4

5. **SSH Keys**: Upload your public key or generate new pair

6. **Boot volume**: 50 GB (min, free tier includes 200 GB total)

7. Click **Create**

#### Step 3.3: Configure Firewall

**In Oracle Cloud Console**:
1. Go to **Networking** > **Virtual Cloud Networks**
2. Select your VCN > **Security Lists** > **Default Security List**
3. **Add Ingress Rules**:
   - **SSH**: Source 0.0.0.0/0, Port 22
   - **HTTP**: Source 0.0.0.0/0, Port 80
   - **HTTPS**: Source 0.0.0.0/0, Port 443
   - **Game UDP**: Source 0.0.0.0/0, Port 5000, Protocol UDP
   - **WebTransport**: Source 0.0.0.0/0, Port 5001

**On the VM (via SSH)**:
```bash
# SSH into VM
ssh ubuntu@<your-vm-ip>

# Allow ports through iptables
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 80 -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 443 -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p udp --dport 5000 -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 5001 -j ACCEPT

# Save rules
sudo netfilter-persistent save
```

#### Step 3.4: Install Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install nginx
sudo apt install nginx -y

# Install certbot for Let's Encrypt
sudo apt install certbot python3-certbot-nginx -y

# Install required libraries
sudo apt install libssl-dev pkg-config -y
```

#### Step 3.5: Cross-Compile for ARM

**On your development machine**:
```bash
# Add ARM target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compiler
sudo apt install gcc-aarch64-linux-gnu  # Ubuntu/Debian
# or
brew install aarch64-unknown-linux-gnu  # macOS

# Build for ARM
cargo build --release --target aarch64-unknown-linux-gnu --bin server

# Binary at: target/aarch64-unknown-linux-gnu/release/server
```

**Transfer to Oracle VM**:
```bash
scp target/aarch64-unknown-linux-gnu/release/server ubuntu@<vm-ip>:~/
scp eryndor.db ubuntu@<vm-ip>:~/  # If you have existing data
```

#### Step 3.6: Set Up systemd Service

**On the VM**, create `/etc/systemd/system/eryndor-server.service`:
```ini
[Unit]
Description=Eryndor MMO Game Server
After=network.target

[Service]
Type=simple
User=ubuntu
WorkingDirectory=/home/ubuntu/eryndor
Environment="DATABASE_URL=sqlite:eryndor.db"
Environment="RUST_LOG=info"
ExecStart=/home/ubuntu/eryndor/server
Restart=always
RestartSec=10

# Security hardening
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

**Create directory and move files**:
```bash
mkdir -p /home/ubuntu/eryndor
mv /home/ubuntu/server /home/ubuntu/eryndor/
cd /home/ubuntu/eryndor
chmod +x server
```

**Enable and start service**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable eryndor-server
sudo systemctl start eryndor-server

# Check status
sudo systemctl status eryndor-server

# View logs
sudo journalctl -u eryndor-server -f
```

#### Step 3.7: Configure nginx + SSL

**Get Let's Encrypt certificate** (requires domain):
```bash
# If you have a domain pointing to your VM
sudo certbot --nginx -d play.eryndor.com
```

**Configure nginx** in `/etc/nginx/sites-available/eryndor`:
```nginx
server {
    listen 80;
    listen 443 ssl http2;
    server_name play.eryndor.com;  # Your domain

    # SSL certificate (from Let's Encrypt)
    ssl_certificate /etc/letsencrypt/live/play.eryndor.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/play.eryndor.com/privkey.pem;

    # Proxy WebTransport to game server
    location / {
        proxy_pass http://localhost:5001;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

**Enable site**:
```bash
sudo ln -s /etc/nginx/sites-available/eryndor /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

---

### Phase 4: Cloudflare Pages Deployment (30 minutes)

#### Step 4.1: Prepare Repository

**Add `.gitignore` entries**:
```gitignore
# WASM build output
crates/eryndor_client/dist/

# Trunk cache
.trunk/
```

**Commit everything**:
```bash
git add .
git commit -m "Add browser support with renet2"
git push
```

#### Step 4.2: Create Cloudflare Pages Project

1. Go to https://pages.cloudflare.com/
2. Sign up (free)
3. Click **Create a project**
4. Connect GitHub account
5. Select `eryndor-mmo` repository

#### Step 4.3: Configure Build Settings

- **Framework preset**: None
- **Build command**:
  ```bash
  cd crates/eryndor_client && trunk build --release
  ```
- **Build output directory**: `crates/eryndor_client/dist`
- **Root directory**: `/` (leave empty)

#### Step 4.4: Environment Variables

Add in Cloudflare Pages settings:
- `SERVER_ADDR`: Your Oracle VM IP or domain
- `SERVER_PORT`: 5001

#### Step 4.5: Deploy

1. Click **Save and Deploy**
2. Wait for build (~5-10 minutes)
3. Get your URL: `eryndor-mmo.pages.dev`

#### Step 4.6: Custom Domain (Optional)

1. In Cloudflare Pages > **Custom domains**
2. Add `play.eryndor.com` (or your domain)
3. Follow DNS setup instructions
4. Wait for SSL to provision (~5 minutes)

---

## Code Changes Required

### Make Server Address Configurable

**Option A: Environment Variable**

In `crates/eryndor_shared/src/constants.rs`:
```rust
pub fn server_addr() -> String {
    std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

pub fn server_port() -> u16 {
    std::env::var("SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000)
}
```

Then use in client connection code:
```rust
let server_addr = format!("{}:{}", server_addr(), server_port());
```

**Option B: Compile-time Configuration**

Use features in `Cargo.toml`:
```toml
[features]
default = []
production = []

[dependencies]
# In build.rs or constants.rs, check feature flags
```

### SQLite Optimization for Production

In `crates/eryndor_server/src/database.rs`:
```rust
// Enable WAL mode for better concurrency
sqlx::query("PRAGMA journal_mode=WAL")
    .execute(&pool)
    .await?;

// Optimize for speed
sqlx::query("PRAGMA synchronous=NORMAL")
    .execute(&pool)
    .await?;

// Increase cache
sqlx::query("PRAGMA cache_size=10000")
    .execute(&pool)
    .await?;
```

---

## Troubleshooting

### Oracle Cloud Signup Issues

**Problem**: "Out of capacity" error when creating ARM instance

**Solutions**:
1. Try different regions (Phoenix, Ashburn, Frankfurt, London)
2. Try different times (late night/early morning in target region)
3. Use AMD instances instead: VM.Standard.E2.1.Micro (free tier includes 2)
4. Retry multiple times (capacity fluctuates)

### ARM Cross-Compilation Fails

**Problem**: Linking errors when building for `aarch64-unknown-linux-gnu`

**Solution 1**: Use `cross` tool
```bash
cargo install cross
cross build --target aarch64-unknown-linux-gnu --release
```

**Solution 2**: Build on the VM itself
```bash
# On Oracle VM
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
git clone <your-repo>
cd eryndor-mmo
cargo build --release --bin server
```

### WebTransport Connection Fails

**Problem**: Browser can't connect to server

**Checklist**:
1. ✅ Valid SSL certificate (Let's Encrypt)?
2. ✅ Port 5001 open in firewall (Oracle + iptables)?
3. ✅ nginx proxy configured correctly?
4. ✅ Server listening on WebTransport port?
5. ✅ Browser supports WebTransport (Chrome/Firefox/Edge)?

**Debug**:
```bash
# Check if server is listening
sudo netstat -tulpn | grep 5001

# Check nginx logs
sudo tail -f /var/log/nginx/error.log

# Check game server logs
sudo journalctl -u eryndor-server -f
```

### WASM Build Too Large

**Problem**: WASM file over 25MB (Cloudflare Pages limit)

**Solutions**:
1. Enable aggressive optimization:
```toml
[profile.release]
opt-level = "z"
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

2. Use `wasm-opt`:
```bash
wasm-opt -Oz -o optimized.wasm input.wasm
```

3. Dynamic asset loading (don't bundle assets in WASM)

### WASM MIME Type Errors

**Problem**: Browser says "incorrect MIME type" for WASM

**Solution**: Configure nginx:
```nginx
location / {
    root /var/www/eryndor;

    types {
        application/wasm wasm;
    }

    # Enable compression
    gzip on;
    gzip_types application/wasm application/javascript;
}
```

---

## Cost Breakdown

### Free Tier (Recommended)

| Service | Configuration | Monthly Cost |
|---------|--------------|--------------|
| Oracle Cloud VM | 4 ARM cores, 24GB RAM | $0 |
| Oracle Block Storage | 200GB total | $0 |
| Oracle Bandwidth | 10TB egress | $0 |
| Cloudflare Pages | Unlimited bandwidth | $0 |
| Let's Encrypt SSL | Free certificate | $0 |
| **Domain (optional)** | .com domain | ~$1/month |
| **TOTAL** | | **$0-1/month** |

### Paid Alternative (If Oracle Fails)

| Service | Configuration | Monthly Cost |
|---------|--------------|--------------|
| Hetzner CAX11 | 2 ARM cores, 4GB RAM | $4.75 |
| Cloudflare Pages | Unlimited bandwidth | $0 |
| Let's Encrypt SSL | Free certificate | $0 |
| **Domain** | .com domain | ~$1/month |
| **TOTAL** | | **~$6/month** |

### Scalability

**Oracle Free Tier Limits**:
- **10-20 concurrent players**: No issues
- **50+ concurrent players**: May need optimization
- **100+ concurrent players**: Consider upgrading to paid VM

**When to Upgrade**:
- CPU usage consistently > 70%
- RAM usage > 80%
- Database queries slowing down (> 100ms)

**Upgrade Path**:
- Hetzner CCX13: 2 dedicated cores, 8GB RAM (~$30/month)
- Migrate SQLite → PostgreSQL (Neon free tier or self-hosted)
- Add load balancing if needed

---

## Alternative Hosting Options

### Fly.io ($5/month)

**Pros**:
- Managed platform (less work)
- Automatic SSL
- Global deployment
- Good for WebTransport

**Cons**:
- $5/month (not free)
- 1GB RAM limit on basic tier
- Learning curve for fly.toml config

**Setup**:
```bash
# Install flyctl
curl -L https://fly.io/install.sh | sh

# Login
fly auth login

# Deploy
fly launch
```

### Hetzner Cloud ($5/month)

**Pros**:
- Reliable performance
- Fast provisioning
- No capacity issues
- x86 (no cross-compilation)

**Cons**:
- Costs money
- Manual setup like Oracle

**Setup**: Similar to Oracle Cloud (SSH, nginx, systemd)

---

## Future Improvements

### When You Scale Beyond 50 Players

1. **Migrate to PostgreSQL**:
   - Use Neon free tier (0.5GB storage)
   - Or self-host on Oracle VM
   - Better concurrency handling

2. **Add Connection Pooling**:
   - Use PgBouncer for PostgreSQL
   - Limit max connections

3. **Consider Multiple Servers**:
   - Load balancing with nginx
   - Separate game servers per region
   - Shared database

4. **Monitoring**:
   - Grafana + Prometheus for metrics
   - Uptime monitoring (UptimeRobot free tier)
   - Error tracking (Sentry free tier)

### When You Need Better Performance

1. **Optimize WASM**:
   - Code splitting (load game in chunks)
   - Asset streaming (don't bundle everything)
   - Web Workers for background tasks

2. **CDN Optimization**:
   - Use Cloudflare's image optimization
   - Implement asset caching strategies

3. **Server Optimization**:
   - Profile with `perf` or `flamegraph`
   - Optimize hot paths
   - Consider multi-threading for physics

---

## Summary

### Fastest Path to Browser Playtest

1. Migrate to renet2 (3-4 hours)
2. Set up WASM build (2 hours)
3. Deploy to Oracle Cloud (2 hours)
4. Deploy WASM to Cloudflare Pages (30 mins)

**Total: ~10 hours, $0/month**

### Easiest Path (No Browser Yet)

1. Use Tailscale virtual LAN (30 mins)
2. Distribute via itch.io (30 mins)

**Total: ~1 hour, $0/month**

You can always add browser support later.

---

## Additional Resources

- **Bevy Book**: https://bevyengine.org/learn/book/
- **renet2 Repository**: https://github.com/UkoeHB/renet2
- **bevy_replicon**: https://github.com/projectharmonia/bevy_replicon
- **Trunk Book**: https://trunkrs.dev/
- **Oracle Cloud Free Tier**: https://www.oracle.com/cloud/free/
- **Cloudflare Pages Docs**: https://developers.cloudflare.com/pages/

---

## Questions?

If you run into issues:
1. Check logs: `sudo journalctl -u eryndor-server -f`
2. Test connectivity: `nc -zv <server-ip> 5000`
3. Verify firewall: `sudo iptables -L -n`
4. Check browser console for WASM errors

Good luck deploying Eryndor! 🎮
