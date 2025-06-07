<div align="center">
<img src="ui/public/android-chrome-512x512.png" height="100"/>
<h1 style="margin-top: 0">Qb-downloader</h1>
</div>

# Qb-downloader

[English](README.md)

*当前正在开发中...*

该项目是在阅读学习 [ani-rss](https://github.com/wushuo894/ani-rss) 时受启发而开发的，照搬了挺多代码（

目前还存在很多问题，仅仅到了能用的阶段，大一暑假里会好好优化一下的。我还在学习中，大佬见笑了，有任何错误或者建议还望 issue 中指出~
## 简介

Qb-downloader 允许你分部分完成 qBittorrent 任务并上传到云盘。这使得你可以下载超出本地最大可用存储空间的大型种子任务。

支持的上传工具：[Rclone rcd](https://rclone.org/commands/rclone_rcd/)（推荐）

典型的 qb-downloader 工作流程包括：
1. 将种子任务分割成多个部分
2. 按顺序下载每个部分
3. 在每个部分之间运行间隔任务，如做种、上传到云盘等

## 使用方法

### 前置要求

此工具需要 qBittorrent 和上传工具运行。请确保你已正确配置它们。

[rclone rcd](https://rclone.org/commands/rclone_rcd/) 服务配置示例：
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
### 部署

#### Docker（推荐）

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
#### 手动部署

确保拥有本地有 JRE。

从 [release page](https://github.com/uchouT/qb-downloader/releases/latest) 获取最新 jar 文件, 然后就可以运行 qb-downloader:
```bash
java -jar path/to/qb-downloader.jar
```

qb-downloader 默认运行在 7845 端口，默认账号: `admin`, 默认密码: `adminadmin`。port 和 host 和通过 `--port` 和 `--host` 参数来指定。推荐配置 systemd service 来使用。

## 致谢

- [ani-rss](https://github.com/wushuo894/ani-rss) - 受此项目启发并借用了部分代码
- [hutool](https://hutool.cn)
- [rclone](https://rclone.org)
- [qBittorrent](https://github.com/qbittorrent/qBittorrent)
- [Vue.js](https://cn.vuejs.org/)
- [Lombok](https://github.com/projectlombok/lombok)
- [Logback](https://github.com/qos-ch/logback)
- [Maven](https://github.com/apache/maven)
- [Gson](https://github.com/google/gson)