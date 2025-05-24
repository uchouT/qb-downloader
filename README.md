# Qb-downloader

Currently in development...

## Description

Qb-downloader allows you to complete your qBittorrent tasks part by part and upload them to your cloud drive. This enables you to download large torrent tasks even if they exceed your maximum available storage.

Supported uploader: [Rclone rcd](https://rclone.org/commands/rclone_rcd/) and [Alist](https://alistgo.com/).

A typical qb-downloader workflow includes:
1. Splitting the torrent task into multiple parts.
2. Downloading each part sequentially.
3. Running interval tasks between each part, such as seeding, uploading to your cloud drive, etc.