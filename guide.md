# W9 - Short Link & File Sharer
# Stack: Frontend: React + Vite; Backend: Rust + Axum
# All-in-one: URL shortening, QR code generation, file uploads, admin panel

## Architecture
- **Frontend**: React SPA (served at `/`)
- **Backend**: Rust API-only (port 10105)
- **Proxy**: Nginx reverse proxy (port 80)
- **SSL**: Cloudflare handles HTTPS
- **Database**: SQLite

## Deployment
Deploy to VPS with:
```bash
sudo ./deploy/install.sh
```

## Routes
- `/` → Frontend (React SPA)
- `/api/*` → Backend API endpoints
- `/admin/*` → Admin interface (via React)
- `/r/:code` → Redirect endpoint
- `/s/:code` → Short link redirect
- `/files/*` → File uploads

## Status: ✅ Complete
- Backend: API-only (no duplicate UI)
- Frontend: Single React SPA
- Build: Automated in install.sh
- Deployment: One command
