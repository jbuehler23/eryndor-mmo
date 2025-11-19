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
