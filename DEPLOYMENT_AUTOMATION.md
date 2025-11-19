# Eryndor MMO - 1-Button Deployment Guide

## Overview

This guide provides everything needed for automated deployment via GitHub Actions.

**Architecture:**
```
WASM Client → Cloudflare (free SSL) → Docker Container on Droplet
```

**Why This Setup:**
- ✅ Free SSL via Cloudflare
- ✅ No nginx to configure
- ✅ Simple Docker deployment
- ✅ Easy for future developers
- ✅ Scalable (add load balancer later with zero code changes)

---

## Prerequisites Setup (One-Time, 30 minutes)

### 1. Cloudflare Setup (Free SSL)

**Step 1: Add Domain to Cloudflare**
1. Go to https://www.cloudflare.com and create free account
2. Click "Add a Site" → Enter `eryndor-online.com`
3. Select Free plan
4. Cloudflare will scan your DNS records
5. Click Continue

**Step 2: Update Nameservers**
1. Cloudflare will show you 2 nameservers (e.g., `bob.ns.cloudflare.com`)
2. Go to your domain registrar (where you bought eryndor-online.com)
3. Update nameservers to the ones Cloudflare provided
4. Wait 5-10 minutes for propagation

**Step 3: Configure DNS**
1. In Cloudflare → DNS → Records
2. Add A record:
   ```
   Type: A
   Name: @
   IPv4 address: 165.227.217.144
   Proxy status: Proxied (orange cloud - this is important!)
   TTL: Auto
   ```

**Step 4: SSL Settings**
1. In Cloudflare → SSL/TLS
2. Set encryption mode to **Flexible**
3. Ensure **WebSockets** is enabled (Network tab)

**That's it for Cloudflare!** It now handles SSL termination automatically.

### 2. Generate JWT Secret

```bash
openssl rand -base64 32
```

Save this output - you'll need it for GitHub Secrets.

### 3. Setup GitHub Secrets

Go to: `https://github.com/YOUR_USERNAME/eryndor-mmo/settings/secrets/actions`

Create these secrets:

| Secret Name | Value | Where to Get It |
|------------|-------|-----------------|
| `DO_API_TOKEN` | Your DigitalOcean API token | https://cloud.digitalocean.com/account/api/tokens |
| `DO_SSH_PRIVATE_KEY` | Your SSH private key | Run `cat ~/.ssh/id_rsa` and copy entire output |
| `DO_DROPLET_IP` | `165.227.217.144` | Your droplet IP |
| `JWT_SECRET` | Output from step 2 | The generated secret |
| `GOOGLE_CLIENT_ID` | (Optional) Google OAuth | https://console.cloud.google.com/ |
| `GOOGLE_CLIENT_SECRET` | (Optional) Google OAuth | https://console.cloud.google.com/ |

### 4. Setup DigitalOcean Droplet

SSH into your droplet and install Docker:

```bash
ssh root@165.227.217.144

# Install Docker
curl -fsSL https://get.docker.com | sh

# Create data directories
mkdir -p /var/lib/eryndor
mkdir -p /var/backups/eryndor

# Exit
exit
```

**That's all the droplet needs!** No nginx, no certbot, no SSL certificates.

### 5. Create Workflow Files

Create these 3 files:

---

## File 1: `.github/workflows/deploy-server.yml`

```yaml
name: Deploy Server

on:
  push:
    branches: [ main ]
  workflow_dispatch:

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}-server

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to GitHub Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha
            type=raw,value=latest

      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Backup database
        uses: appleboy/ssh-action@v1.0.0
        with:
          host: ${{ secrets.DO_DROPLET_IP }}
          username: root
          key: ${{ secrets.DO_SSH_PRIVATE_KEY }}
          script: |
            mkdir -p /var/backups/eryndor
            if [ -f /var/lib/eryndor/eryndor.db ]; then
              cp /var/lib/eryndor/eryndor.db /var/backups/eryndor/eryndor.db.$(date +%Y%m%d_%H%M%S)
              find /var/backups/eryndor -name "*.db.*" -mtime +7 -delete
              echo "Database backed up successfully"
            fi

      - name: Deploy to DigitalOcean Droplet
        uses: appleboy/ssh-action@v1.0.0
        with:
          host: ${{ secrets.DO_DROPLET_IP }}
          username: root
          key: ${{ secrets.DO_SSH_PRIVATE_KEY }}
          script: |
            # Login to GitHub Container Registry
            echo ${{ secrets.GITHUB_TOKEN }} | docker login ${{ env.REGISTRY }} -u ${{ github.actor }} --password-stdin

            # Pull new image
            docker pull ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest

            # Stop and remove old container
            docker stop eryndor-server || true
            docker rm eryndor-server || true

            # Run new container
            docker run -d \
              --name eryndor-server \
              --restart unless-stopped \
              -p 5001:5001/udp \
              -p 5003:5003 \
              -v /var/lib/eryndor:/data \
              -e SERVER_ADDR=0.0.0.0 \
              -e DATABASE_PATH=/data/eryndor.db \
              -e JWT_SECRET=${{ secrets.JWT_SECRET }} \
              -e GOOGLE_CLIENT_ID=${{ secrets.GOOGLE_CLIENT_ID }} \
              -e GOOGLE_CLIENT_SECRET=${{ secrets.GOOGLE_CLIENT_SECRET }} \
              -e RUST_LOG=info \
              ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:latest

            # Cleanup old images
            docker image prune -af

      - name: Health check
        uses: appleboy/ssh-action@v1.0.0
        with:
          host: ${{ secrets.DO_DROPLET_IP }}
          username: root
          key: ${{ secrets.DO_SSH_PRIVATE_KEY }}
          script: |
            sleep 10
            if docker ps | grep eryndor-server; then
              echo "✓ Server is running"
              docker logs eryndor-server --tail 20
            else
              echo "✗ Server failed to start!"
              docker logs eryndor-server --tail 50
              exit 1
            fi
```

---

## File 2: `.github/workflows/deploy-client.yml`

```yaml
name: Deploy Client

on:
  push:
    branches: [ main ]
  workflow_dispatch:

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-wasm-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install Bevy CLI
        run: cargo install bevy_cli

      - name: Build WASM client
        run: |
          cd crates/eryndor_client
          bevy build web
        env:
          SERVER_WS_URL: wss://eryndor-online.com/

      - name: Install doctl
        uses: digitalocean/action-doctl@v2
        with:
          token: ${{ secrets.DO_API_TOKEN }}

      - name: Deploy to DigitalOcean App Platform
        run: |
          # Note: You'll need to create the app first via UI or doctl
          # Then get the app ID and store it as a GitHub secret
          doctl apps create-deployment ${{ secrets.DO_APP_ID }} --wait
```

---

## File 3: `.do/app.yaml`

```yaml
name: eryndor-mmo-client
region: nyc

static_sites:
  - name: web
    github:
      repo: YOUR_USERNAME/eryndor-mmo
      branch: main
      deploy_on_push: true

    source_dir: crates/eryndor_client/bevy_web/web-release/client

    routes:
      - path: /

    environment_slug: html

    cors:
      allow_origins:
        - prefix: https://
      allow_methods:
        - GET
        - POST
        - OPTIONS
      allow_headers:
        - Content-Type
        - Authorization
```

---

## File 4: `scripts/backup-database.sh`

```bash
#!/bin/bash
# Database backup script

BACKUP_DIR="/var/backups/eryndor"
DB_PATH="/var/lib/eryndor/eryndor.db"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

if [ -f "$DB_PATH" ]; then
    echo "Backing up database..."
    cp "$DB_PATH" "$BACKUP_DIR/eryndor.db.$TIMESTAMP"
    echo "Backup created: $BACKUP_DIR/eryndor.db.$TIMESTAMP"

    # Keep last 7 days
    find "$BACKUP_DIR" -name "eryndor.db.*" -mtime +7 -delete
    echo "Old backups cleaned up"
else
    echo "Database not found at $DB_PATH"
fi
```

Make executable: `chmod +x scripts/backup-database.sh`

---

## File 5: `.env.production`

```bash
# Production Environment Configuration
SERVER_ADDR=0.0.0.0
SERVER_PORT=5001
SERVER_PORT_WEBSOCKET=5003
DATABASE_PATH=/data/eryndor.db
RUST_LOG=info

# IMPORTANT: Replace with your generated secret
JWT_SECRET=REPLACE_WITH_YOUR_GENERATED_SECRET

# Optional OAuth
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
```

---

## How to Deploy

### First Deployment:

1. Create all the files above
2. Commit and push:

```bash
git add .
git commit -m "Setup automated deployment"
git push origin main
```

3. Watch GitHub Actions: `https://github.com/YOUR_USERNAME/eryndor-mmo/actions`

### Every Future Deployment:

```bash
git push
```

That's it! GitHub Actions handles:
- Running tests
- Building Docker image
- Backing up database
- Deploying server
- Building WASM client
- Deploying client

**Deployment takes ~8 minutes**

---

## Architecture Details

### How It Works:

```
1. Developer pushes code to GitHub

2. GitHub Actions:
   ├─ Runs cargo test
   ├─ Builds server Docker image
   ├─ Pushes to ghcr.io
   ├─ SSHs to droplet
   ├─ Backs up database
   ├─ Deploys new container
   └─ Verifies health

3. Cloudflare:
   ├─ Provides SSL/TLS
   ├─ Routes wss://eryndor-online.com → ws://165.227.217.144:5003
   └─ Caches static assets

4. Client:
   ├─ Builds WASM with bevy CLI
   └─ Deploys to DigitalOcean App Platform
```

### Network Flow:

```
Browser
  │
  ├─ HTTPS → eryndor-mmo-web-app-f6fit.ondigitalocean.app (client)
  │
  └─ WSS → eryndor-online.com
            │
            └─ Cloudflare (SSL termination)
                 │
                 └─ WS → 165.227.217.144:5003 (server)
```

---

## For Future Developers

### Simple Mental Model:

1. **Code**: Just push to main branch
2. **Server**: Docker container on single droplet
3. **Client**: Static WASM on App Platform
4. **SSL**: Cloudflare handles it (free)
5. **Deployment**: Automatic via GitHub Actions

### To Understand the System:

```bash
# View server logs
ssh root@165.227.217.144 'docker logs eryndor-server --tail 100'

# Check server status
ssh root@165.227.217.144 'docker ps'

# Manual deployment (if GitHub Actions fails)
ssh root@165.227.217.144
docker pull ghcr.io/YOUR_USERNAME/eryndor-mmo-server:latest
docker stop eryndor-server
docker rm eryndor-server
docker run -d --name eryndor-server \
  --restart unless-stopped \
  -p 5001:5001/udp \
  -p 5003:5003 \
  -v /var/lib/eryndor:/data \
  -e SERVER_ADDR=0.0.0.0 \
  -e DATABASE_PATH=/data/eryndor.db \
  -e JWT_SECRET=your-secret \
  ghcr.io/YOUR_USERNAME/eryndor-mmo-server:latest
```

---

## Scaling Path (When You Need It)

### Current Setup (1-100 players):
- Single droplet
- Cloudflare SSL
- $12-18/month

### When You Need More (100-1000 players):

1. **Create DigitalOcean Load Balancer** ($12/mo)
2. **Clone droplet 2-3 times** (identical containers)
3. **Add droplets to load balancer**
4. **Update DNS**: Point to load balancer IP instead
5. **No code changes needed!**

### When You Need Even More (1000+ players):

1. Separate database to managed PostgreSQL
2. Use Redis for session storage
3. Add more droplets to load balancer
4. Consider regional deployment

**But start simple.** Current setup handles hundreds of players easily.

---

## Troubleshooting

### Server not starting?
```bash
ssh root@165.227.217.144 'docker logs eryndor-server'
```

### Client can't connect?
1. Check Cloudflare DNS (orange cloud enabled?)
2. Check server is running: `docker ps`
3. Check WebSocket port: `nc -zv 165.227.217.144 5003`

### Deployment failing?
1. Check GitHub Actions logs
2. Verify GitHub Secrets are set
3. Verify SSH key works: `ssh root@165.227.217.144`

### Database issues?
```bash
# List backups
ssh root@165.227.217.144 'ls -lh /var/backups/eryndor/'

# Restore backup
ssh root@165.227.217.144 'cp /var/backups/eryndor/eryndor.db.TIMESTAMP /var/lib/eryndor/eryndor.db'
ssh root@165.227.217.144 'docker restart eryndor-server'
```

---

## Cost Breakdown

- **Droplet**: $12-18/month (Basic/Regular)
- **App Platform**: $0 (static site free tier)
- **Cloudflare**: $0 (free plan)
- **GitHub**: $0 (free plan, 2000 Actions minutes/month)
- **Docker Registry**: $0 (GitHub Container Registry free)

**Total**: $12-18/month

---

## Summary

You now have:
- ✅ 1-command deployments (`git push`)
- ✅ Automated testing
- ✅ Database backups
- ✅ Health checks
- ✅ Free SSL
- ✅ Simple infrastructure
- ✅ Easy to scale later

**Happy deploying!**
