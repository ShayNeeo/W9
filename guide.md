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
shayneeo@x1:~/W9$ hostname -I && echo "---" && sudo iptables -L -n 2>/dev/null | grep -E "(80|INPUT)" | head -10
162.43.30.192 
---
Chain INPUT (policy ACCEPT)
shayneeo@x1:~/W9$ git pull && ./deploy/install.sh
remote: Enumerating objects: 13, done.
remote: Counting objects: 100% (13/13), done.
remote: Compressing objects: 100% (3/3), done.
remote: Total 7 (delta 3), reused 7 (delta 3), pack-reused 0 (from 0)
Unpacking objects: 100% (7/7), 736 bytes | 184.00 KiB/s, done.
From https://github.com/ShayNeeo/W9
   6f96698..e3adea7  main       -> origin/main
Updating 6f96698..e3adea7
Fast-forward
 deploy/install.sh      |   4 +
 server/src/handlers.rs | 180 +-------------------------------------------
 2 files changed, 6 insertions(+), 178 deletions(-)
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
   Compiling w9 v0.1.0 (/home/shayneeo/W9/server)
warning: value assigned to `cur` is never read
   --> server/src/handlers.rs:134:27
    |
134 |             let mut cur = std::io::Cursor::new(&mut buf);
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = help: maybe it is overwritten before being read?
    = note: `#[warn(unused_assignments)]` (part of `#[warn(unused)]`) on by default

warning: `w9` (bin "w9") generated 1 warning
    Finished `release` profile [optimized] target(s) in 10.04s
Building frontend...

up to date, audited 31 packages in 782ms

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
✓ built in 1.51s
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
Starting (or restarting) w9 service
● w9.service - w9 - Link & file sharer service
     Loaded: loaded (/etc/systemd/system/w9.service; enabled; preset: enabled)
     Active: active (running) since Mon 2025-11-17 10:23:14 UTC; 36ms ago
 Invocation: eeaa4c83fb9d43c5a9e69d6b96365779
   Main PID: 25387 (w9)
      Tasks: 4 (limit: 4599)
     Memory: 1.3M (peak: 1.8M)
        CPU: 9ms
     CGroup: /system.slice/w9.service
             └─25387 /opt/w9/w9

Nov 17 10:23:14 x1 systemd[1]: Started w9.service - w9 - Link & file sharer service.
Nov 17 10:23:14 x1 w9[25387]: 2025-11-17T10:23:14.623573Z  INFO Base URL: https://w9.se
Nov 17 10:23:14 x1 w9[25387]: 2025-11-17T10:23:14.624090Z  INFO 🚀 Server listening on 0.0.0.0:10105
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