# First Principal: No guide, no .md files, no explainations. Just do what I tell you.
# W9 - Short Link & File Sharer
# Stack: Frontend: React + Vite; Backend: Rust + Axum
# The project will be deployed using install.sh script on my VPS. not this local, don't prompt to run on local.
# Alwways update Auto-setup install.sh script (If needed).
# Update sitemap.xml and robots.txt after create any newpages or transition.


Fix this error: Cloudflare 521:
I already point the domain A to the VPS with proxied.
Why this happen?
```
shayneeo@x1:~/W9$ sudo DOMAIN=w9.se BASE_URL=https://w9.se ./deploy/install.sh
Repo root: /home/shayneeo/W9
Building release (as user: shayneeo)
Installing system packages...
Hit:1 https://deb.debian.org/debian sid InRelease
Reading package lists... Done
Reading package lists... Done
Building dependency tree... Done
Reading state information... Done
build-essential is already the newest version (12.12).
pkg-config is already the newest version (1.8.1-4).
libsqlite3-dev is already the newest version (3.46.1-8).
ca-certificates is already the newest version (20250419).
curl is already the newest version (8.17.0-2).
git is already the newest version (1:2.51.0-1).
nodejs is already the newest version (20.19.5+dfsg+~cs20.19.24-1).
npm is already the newest version (9.2.0~ds1-3).
nginx is already the newest version (1.28.0-6).
ufw is already the newest version (0.36.2-9).
Solving dependencies... Done
0 upgraded, 0 newly installed, 0 to remove and 81 not upgraded.
Building backend release (as user: shayneeo)
warning: value assigned to `cur` is never read
   --> server/src/handlers.rs:134:27
    |
134 |             let mut cur = std::io::Cursor::new(&mut buf);
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: `w9` (bin "w9") generated 1 warning
    Finished `release` profile [optimized] target(s) in 0.16s
Building frontend...

up to date, audited 31 packages in 798ms

4 packages are looking for funding
  run `npm fund` for details

2 moderate severity vulnerabilities

To address all issues (including breaking changes), run:
  npm audit fix --force

Run `npm audit` for details.

> w9-frontend@0.0.0 build
> tsc -b && vite build

vite v5.4.21 building for production...
✓ 31 modules transformed.
dist/index.html                   0.39 kB │ gzip:  0.26 kB
dist/assets/index-9WusT_-b.css    1.52 kB │ gzip:  0.62 kB
dist/assets/index-Sr-nNy_0.js   147.55 kB │ gzip: 47.65 kB
✓ built in 1.49s
Frontend built to /home/shayneeo/W9/frontend/dist
Installing binary to /opt/w9/w9
Installing frontend to /var/www/w9
✓ Frontend installed
Updating /etc/default/w9 (ENV_OVERWRITE enabled)
Writing systemd unit /etc/systemd/system/w9.service
Reloading systemd and enabling service
Configuring nginx for frontend + API on w9.se
nginx: the configuration file /etc/nginx/nginx.conf syntax is ok
nginx: configuration file /etc/nginx/nginx.conf test is successful
✓ Nginx config valid
Skipping adding existing rule
Skipping adding existing rule (v6)
Starting (or restarting) w9 service
● w9.service - w9 - Link & file sharer service
     Loaded: loaded (/etc/systemd/system/w9.service; enabled; preset: enabled)
     Active: active (running) since Mon 2025-11-17 10:39:29 UTC; 27ms ago
 Invocation: 35d7c913b33e4e8ea04ffc9931b4124a
   Main PID: 25990 ((w9))
      Tasks: 1 (limit: 4599)
     Memory: 1.7M (peak: 1.7M)
        CPU: 5ms
     CGroup: /system.slice/w9.service
             └─25990 "(w9)"

Nov 17 10:39:29 x1 systemd[1]: Started w9.service - w9 - Link & file sharer service.
Reloading nginx (if installed)

✓ Done! Service is running.

========== Installation Summary ==========
Domain:      w9.se
Backend:     /opt/w9/w9 (port 10105)
Frontend:    /var/www/w9
Data:        /opt/w9/data
Uploads:     /opt/w9/uploads
Nginx:       Port 80
SSL:         Cloudflare (auto)

Routes:
  /              → Frontend
  /api/*         → Backend API
  /admin/*       → Backend Admin
  /r/:code       → Redirect
  /s/:code       → Short link
  /files/*       → Uploads

View logs: sudo journalctl -u w9 -f
========================================
```