<div align="center">
<img src="img/icon.png" height="100"/>
<h1 style="margin-top: 0">Qb-downloader</h1>
<img src="img/home.png" />
</div>

## Description

Qb-downloader allows you to complete your qBittorrent tasks part by part and upload them to your cloud drive. This enables you to download large torrent tasks even if they exceed your maximum available storage.

Supported uploaders: [Rclone rcd](https://rclone.org/commands/rclone_rcd/)
A typical qb-downloader workflow includes:
1. Splitting the torrent task into multiple parts.
2. Downloading each part sequentially.
3. Running interval tasks between each part, such as seeding, uploading to your cloud drive, etc.

## TODO

- [x] Adding support for customizing torrent contents.
- [ ] Forced continuation when error occurs.

## Usage

### prerequisite

This tool requires qBittorrent and an uploader to be running. Make sure you have configured them properly.

> [!IMPORTANT]
> make sure you haven't enabled options like "Delete torrent after completion" in your qBittorrent.

Example [rclone rcd](https://rclone.org/commands/rclone_rcd/) service configuration:
```ini
[Unit]
Description=Rclone Remote Control (rcd)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/rclone rcd --rc-addr=:5572  --rc-user=admin --rc-pass="password"
User=rclone
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

### Installation

1. **Download the binary**
   
   Download the latest executable for your platform from the [releases page](https://github.com/uchouT/qb-downloader-rust/releases).

2. **Run the service**
   ```bash
   # Default port (7845)
   ./qb-downloader-rust
   
   # Custom port
   ./qb-downloader-rust --port 8080
   # or
   ./qb-downloader-rust -p 8080
   ```

3. **Access the web interface**
   
   Open your web browser and navigate to `http://localhost:7845` (or your custom port).
   
   **Default credentials:**
   - Username: `admin`
   - Password: `adminadmin`

### Uninstall

To completely remove qb-downloader from your system:

1. Stop the running service
2. Remove the executable file
3. Clean up configuration and data directories:
   ```bash
   rm -rf ~/.config/qb-downloader
   rm -rf ~/.local/share/qb-downloader
   ```