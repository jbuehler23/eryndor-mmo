#!/bin/bash
# Quick setup script for deployment automation

set -e

echo "=== Eryndor MMO Deployment Setup ==="
echo ""

# Check if running in git repository
if [ ! -d ".git" ]; then
    echo "Error: This script must be run from the root of the git repository"
    exit 1
fi

# Create directory structure
echo "Creating directory structure..."
mkdir -p .github/workflows
mkdir -p .do
mkdir -p scripts

# Generate JWT secret
echo ""
echo "Generating JWT secret..."
JWT_SECRET=$(openssl rand -base64 32)
echo "Your JWT_SECRET: $JWT_SECRET"
echo "IMPORTANT: Save this securely - you'll need it for GitHub Secrets!"
echo ""

# Create .env.production
echo "Creating .env.production..."
cat > .env.production << EOF
# Production Environment Configuration
SERVER_ADDR=0.0.0.0
SERVER_PORT=5001
SERVER_PORT_WEBSOCKET=5003
DATABASE_PATH=/data/eryndor.db
JWT_SECRET=$JWT_SECRET
RUST_LOG=info
GOOGLE_CLIENT_ID=
GOOGLE_CLIENT_SECRET=
EOF

echo ".env.production created with generated JWT secret"
echo ""

# Create GitHub Secrets checklist
echo "Creating github-secrets-checklist.md..."
cat > github-secrets-checklist.md << 'EOF'
# GitHub Secrets Checklist

Go to: https://github.com/YOUR_USERNAME/eryndor-mmo/settings/secrets/actions

Create these secrets:

## Required Secrets

- [ ] `DO_API_TOKEN` - DigitalOcean API token
  - Get from: https://cloud.digitalocean.com/account/api/tokens
  - Create a new token with read+write permissions

- [ ] `DO_SSH_PRIVATE_KEY` - Your SSH private key for the droplet
  - Run: `cat ~/.ssh/id_rsa` (or your key file)
  - Copy the ENTIRE key including BEGIN and END lines

- [ ] `DO_DROPLET_IP` - Your droplet IP address
  - Value: 165.227.217.144

- [ ] `JWT_SECRET` - Server JWT secret
  - Value: [Copy from .env.production file]

## Optional Secrets (for OAuth)

- [ ] `GOOGLE_CLIENT_ID` - Google OAuth client ID
- [ ] `GOOGLE_CLIENT_SECRET` - Google OAuth client secret

## Verification

After adding all secrets, push to main branch to trigger deployment:

```bash
git add .
git commit -m "Setup automated deployment"
git push origin main
```

Then check: https://github.com/YOUR_USERNAME/eryndor-mmo/actions
EOF

echo "github-secrets-checklist.md created"
echo ""

# Summary
echo "=== Setup Complete! ==="
echo ""
echo "Next steps:"
echo "1. Review and follow the instructions in DEPLOYMENT_AUTOMATION.md"
echo "2. Add GitHub Secrets using github-secrets-checklist.md as a guide"
echo "3. Your JWT secret is in .env.production (keep this file secure!)"
echo "4. Create the remaining workflow files from DEPLOYMENT_AUTOMATION.md"
echo "5. Push to main branch to trigger your first automated deployment!"
echo ""
echo "JWT Secret (save this): $JWT_SECRET"
echo ""
