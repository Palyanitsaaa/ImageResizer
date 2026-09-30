```markdown
# Image Resizer Service

A lightweight, high-performance image resizing microservice written in Rust using Axum and Tokio. The service validates incoming requests using an HMAC-SHA1 signature, resizes images proportionally on the fly, and serves them via an NGINX reverse proxy with caching support.

---

## Features

- **Proportional Resizing**: Scales images to fit within target dimensions while maintaining aspect ratio (powered by the `image` crate).
- **URL Tamper Protection**: Validates HMAC-SHA1 URL signatures before processing requests.
- **Asynchronous Execution**: Uses `tokio::task::spawn_blocking` for CPU-intensive image processing to keep the I/O event loop non-blocking.
- **Configuration Management**: Loads runtime settings from environment variables via `.env`.
- **Production-Ready**: Designed to sit behind NGINX as a caching reverse proxy and run as a `systemd` daemon.

---

## Architecture Overview


```

Client (Browser)
│
▼
[NGINX] ──(Cache Hit?)──► Serve cached image immediately
│
(Cache Miss)
▼ proxy_pass
[Rust Resizer Service]
│
├─► 1. Parse URL parameters (token, dimensions, image path)
├─► 2. Verify HMAC-SHA1 signature
├─► 3. Read source image from disk
├─► 4. Resize image proportionally
└─► 5. Return JPEG payload to NGINX

```

---

## URL Format

The service expects URLs in the following structure:


```

/images/{token}/{width}x{height}/{path_to_image}

```

- **`token`**: A 12-character signature derived from HMAC-SHA1 with base64 character replacement (`+` -> `-`, `/` -> `_`, `=` -> `,`).
- **`{width}x{height}`**: Target bounding box dimensions (e.g., `100x100`).
- **`path_to_image`**: Relative path to the original image file.

**Example Request:**

```

[https://cdn.example.com/images/Ytdgfay_Bchx/100x100/upload/images/product/718_2.jpg](https://www.google.com/search?q=https://cdn.example.com/images/Ytdgfay_Bchx/100x100/upload/images/product/718_2.jpg)

```

---

## Configuration

Create a `.env` file in the project root (or point to an environment file in production):

```env
SECRET_KEY_IMAGE_RESIZE=your_secret_key_here
BASE_IMAGE_PATH=/var/www/www-root/[example.com/](https://example.com/)

```

| Variable | Description |
| --- | --- |
| `SECRET_KEY_IMAGE_RESIZE` | Secret salt used to compute and verify the HMAC-SHA1 token. |
| `BASE_IMAGE_PATH` | Absolute path to the directory where source images reside on disk. |

---

## Building and Running

### Development

```bash
# Run tests
cargo test

# Start the development server (listens on 127.0.0.1:3000)
cargo run

```

### Production Build

```bash
cargo build --release
sudo cp target/release/image_resizer /usr/local/bin/image_resizer

```

---

## NGINX Configuration

To route requests properly and prevent static file rules from intercepting dynamic requests, use the `^~` prefix matching modifier:

```nginx
# Define the cache zone inside http { ... }
# proxy_cache_path /var/cache/nginx/images levels=1:2 keys_zone=img_cache:20m max_size=10g inactive=30d use_temp_path=off;

server {
    server_name cdn.example.com;

    # Pass resizing requests directly to the Rust backend
    location ^~ /images/ {
        proxy_pass [http://127.0.0.1:3000](http://127.0.0.1:3000);

        # Caching configuration
        proxy_cache img_cache;
        proxy_cache_key "$scheme$request_method$host$uri";
        proxy_cache_valid 200 30d;
        proxy_cache_valid 404 1m;

        add_header X-Cache-Status $upstream_cache_status;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    # Fallback rule for static assets
    location ~* \.(?:jpe?g|png|webp)$ {
        try_files $uri =404;
    }
}

```

---

## Systemd Service

To run the application as a background daemon, create `/etc/systemd/system/image_resizer.service`:

```ini
[Unit]
Description=Rust Image Resizer Service
After=network.target

[Service]
Type=simple
User=www-data
Group=www-data
EnvironmentFile=/etc/image_resizer.env
ExecStart=/usr/local/bin/image_resizer
Restart=always
RestartSec=3

NoNewPrivileges=true
ProtectSystem=full

[Install]
WantedBy=multi-user.target

```

Reload and start the daemon:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now image_resizer

```

---

## License

MIT OR Apache-2.0

```

```