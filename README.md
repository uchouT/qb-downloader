<div align="center">
<img src="ui/public/android-chrome-512x512.png" height="100"/>
<h1 style="margin-top: 0">Qb-downloader</h1>
</div>


[中文](README-CN.md)

*Currently in development...*

## Description

Qb-downloader allows you to complete your qBittorrent tasks part by part and upload them to your cloud drive. This enables you to download large torrent tasks even if they exceed your maximum available storage.

Supported uploaders: [Rclone rcd](https://rclone.org/commands/rclone_rcd/) (recommended).
A typical qb-downloader workflow includes:
1. Splitting the torrent task into multiple parts.
2. Downloading each part sequentially.
3. Running interval tasks between each part, such as seeding, uploading to your cloud drive, etc.

## Usage

This tool requires qBittorrent and an uploader to be running. Make sure you have configured them properly.

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

### Docker (Recommended)

```yaml
services:
  qb-downloader:
    network_mode: "host"
    container_name: qb-downloader
    environment:
      - PORT=7845
      - CONFIG=/config
    volumes:
      - ./config/qb-downloader:/config
      - ./downloads:/downloads
    restart: unless-stopped
    image: uchout/qb-downloader:latest
```

## Acknowledgments

- [ani-rss](https://github.com/wushuo894/ani-rss) - Inspired by and borrowed code from this project
- [hutool](https://hutool.cn)
- [rclone](https://rclone.org)
- [qBittorrent](https://github.com/qbittorrent/qBittorrent)
- [Vue.js](https://cn.vuejs.org/)
- [Lombok](https://github.com/projectlombok/lombok)
- [Logback](https://github.com/qos-ch/logback)
- [Maven](https://github.com/apache/maven)
- [Gson](https://github.com/google/gson)