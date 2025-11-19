# Eryndor MMO - 1-Button Deployment Quick Start

## What's Been Created

I've set up the foundation for automated deployment:

### Files Created:
1. `.github/workflows/ci.yml` - Automated testing on every push
2. `DEPLOYMENT_AUTOMATION.md` - Complete guide with all deployment files
3. `scripts/setup-deployment.sh` - Quick setup script
4. `QUICK_START.md` - This file

### What You Need to Do:

## Step 1: Run Setup Script (5 minutes)

```bash
cd /c/Dev/Workspace/eryndor-mmo
bash scripts/setup-deployment.sh
```

This will:
- Generate a JWT secret
- Create `.env.production` with the secret
- Create a checklist for GitHub Secrets

## Step 2: Copy Workflow Files (10 minutes)

Open `DEPLOYMENT_AUTOMATION.md` and copy these files:

1. `.github/workflows/deploy-server.yml` (lines 25-140)
2. `.github/workflows/deploy-client.yml` (lines 145-190)
3. `.do/app.yaml` (lines 195-220)
4. `scripts/backup-database.sh` (lines 225-250)
5. `scripts/health-check.sh` (lines 255-275)

Save each to the specified path.

## Step 3: Setup GitHub Secrets (15 minutes)

Go to: https://github.com/YOUR_USERNAME/eryndor-mmo/settings/secrets/actions

Add these secrets (values in `.env.production` and `github-secrets-checklist.md`):

- `DO_API_TOKEN` - From https://cloud.digitalocean.com/account/api/tokens
- `DO_SSH_PRIVATE_KEY` - Your SSH private key
- `DO_DROPLET_IP` - 165.227.217.144
- `JWT_SECRET` - From `.env.production`

## Step 4: Setup DigitalOcean (15 minutes)

### Droplet:
```bash
ssh root@165.227.217.144
curl -fsSL https://get.docker.com | sh
mkdir -p /var/lib/eryndor /var/backups/eryndor
exit
```

### App Platform:
1. Go to https://cloud.digitalocean.com/apps
2. Click "Create App"
3. Connect to GitHub repo: `jbuehler23/eryndor-mmo`
4. Use these settings:
   - Type: Static Site
   - Branch: main
   - Build command: (leave empty)
   - Output directory: `crates/eryndor_client/bevy_web/web-release/client`

## Step 5: Deploy!

```bash
git add .
git commit -m "Setup 1-button deployment automation"
git push origin main
```

Then watch at: https://github.com/YOUR_USERNAME/eryndor-mmo/actions

## What Happens Next

1. GitHub Actions runs CI tests
2. Server deploys to DigitalOcean Droplet (Docker container)
3. Client deploys to DigitalOcean App Platform (static site)
4. Both are live in ~8 minutes

## After First Deployment

Every future deployment is just:

```bash
git push
```

That's it! Server and client both deploy automatically.

## Troubleshooting

### GitHub Actions failing?
- Check Actions tab for error logs
- Verify GitHub Secrets are set correctly

### Server not starting?
```bash
ssh root@165.227.217.144 'docker logs eryndor-server'
```

### Client build failing?
- Check GitHub Actions logs
- Verify `bevy` CLI installed: `cargo install bevy_cli`

## Next Steps

1. Configure your domain DNS to point to:
   - Server (A record): 165.227.217.144
   - Client (CNAME): Your App Platform URL

2. Setup SSL/TLS certificates (Let's Encrypt)

3. Configure NGINX reverse proxy for WebSocket

See `DEPLOYMENT_AUTOMATION.md` for detailed instructions on these steps.

---

**Total setup time**: ~45 minutes
**Future deployments**: `git push` (8 minutes automatic)
