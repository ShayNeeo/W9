# W9 Project Migration Guide

## Project Rename: ping0 → w9

This document outlines all changes made during the project rename from **ping0** to **w9**.

### Summary of Changes

1. **Project Naming**
   - Server package: `ping0` → `w9`
   - App package: `ping0-app` → `w9-app`
   - Frontend package: `ping0-frontend` → `w9-frontend`
   - Webpage package: `webpage` → `w9-webpage`

2. **Service Configuration**
   - Service name: `ping0` → `w9`
   - Binary name: `ping0` → `w9`
   - Service user: `ping0` → `w9`
   - Install directory: `/opt/ping0` → `/opt/w9`

3. **Database Changes**
   - Default database: `data/ping0.db` → `data/w9.db`
   - All table schemas remain unchanged
   - Database structure is forward-compatible

4. **Domain & Infrastructure**
   - All services configured for: `w9.se`
   - Admin cookie: `ping0_admin` → `w9_admin`
   - Base URL: `https://w9.se`

5. **UI/Branding**
   - Page titles updated from "ping0" to "w9"
   - Health check service name updated
   - Frontend headers and titles updated

### Deployment Instructions

#### For Existing Deployments

If you have an existing `ping0` deployment and want to migrate:

1. **Backup your existing database:**
   ```bash
   cp data/ping0.db data/ping0.db.backup
   ```

2. **Migrate to new setup:**
   ```bash
   # Use the new install.sh with existing database
   DATABASE_PATH=/path/to/old/ping0.db sudo ./deploy/install.sh
   ```

3. **Verify migration:**
   - Check that existing short links still work
   - Verify admin access with existing credentials
   - Ensure file uploads are accessible

#### For New Installations

Simply run the standard installation:
```bash
sudo ./deploy/install.sh
```

The new installation will use `data/w9.db` by default.

### Files Modified

#### Core Backend
- `server/Cargo.toml` - Package name
- `server/src/main.rs` - Database path, health check
- `server/src/handlers.rs` - Admin cookies, template imports
- `server/src/templates.rs` - Page titles

#### Frontend
- `frontend/src/App.tsx` - UI text
- `frontend/index.html` - Page title
- `frontend/package.json` - Package name
- `frontend/README.md` - Documentation

#### Application
- `app/Cargo.toml` - Package name
- `app/src/lib.rs` - UI text

#### Deployment
- `deploy/install.sh` - Service names, paths, database
- `Dockerfile` - Binary names, user names
- `docker-compose.yml` - Service name

#### Documentation
- `README.md` - Project name, links, paths
- `static/index.html` - UI text
- `webpage/package.json` - Package name

### Database Migration Details

The database tables remain unchanged:
- `items` - Stores URLs and file metadata
- `admin` - Admin user credentials
- `sessions` - Active admin sessions

### Breaking Changes

- Service name changed from `ping0` to `w9`
- Systemd unit name: `ping0.service` → `w9.service`
- Environment file: `/etc/default/ping0` → `/etc/default/w9`
- Install directory: `/opt/ping0` → `/opt/w9`

### Backward Compatibility Notes

- All generated short links will continue to work
- Existing databases can be migrated by specifying `DATABASE_PATH`
- All URL routes remain unchanged (`/r/:code`, `/s/:code`, etc.)
- Admin credentials from old database remain valid if using same DB

### Next Steps

1. Replace `ping0` repository with `w9`
2. Run deployment script: `sudo ./deploy/install.sh`
3. Monitor service health: `sudo systemctl status w9`
4. Verify functionality with existing short links

### Verification Checklist

- [ ] Service is running: `sudo systemctl status w9`
- [ ] Database accessible: Check `/var/opt/w9/data/`
- [ ] Existing links work: Test a short link
- [ ] Admin login works: Visit `/admin/login`
- [ ] New uploads work: Create a new short link
- [ ] Health check passes: `curl http://localhost:8080/health`

---

**Migration Date:** 2025-11-17
**From:** ping0 v0.1.0
**To:** w9 v0.1.0

