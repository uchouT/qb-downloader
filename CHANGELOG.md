# Changelog


## v2.2.0

[compare changes](https://github.com/uchouT/qb-downloader/compare/v2.1.1...v2.2.0)

### 🚀 Enhancements

- Add task resume api ([0b8872e](https://github.com/uchouT/qb-downloader/commit/0b8872e))
- Lazily build multipart ([f983aaf](https://github.com/uchouT/qb-downloader/commit/f983aaf))
- Resume task when error occured ([0a708c5](https://github.com/uchouT/qb-downloader/commit/0a708c5))

### 🔥 Performance

- Parallelly set_share_limit and launch torrent task ([a2e0e29](https://github.com/uchouT/qb-downloader/commit/a2e0e29))

### 🩹 Fixes

- Qb request failed if cookie expired ([77767fb](https://github.com/uchouT/qb-downloader/commit/77767fb))
- Qb re-login after cookie expire ([aa97d1f](https://github.com/uchouT/qb-downloader/commit/aa97d1f))
- Completed task was marked error by mistake ([fbfb78e](https://github.com/uchouT/qb-downloader/commit/fbfb78e))
- Rclone run_check before run_interval has stored job_id chore: reduce state write lock retrival ([cb425d3](https://github.com/uchouT/qb-downloader/commit/cb425d3))

### 💅 Refactors

- **request:** Wrap external request lib ([62bbf1d](https://github.com/uchouT/qb-downloader/commit/62bbf1d))
- **error:** Use thiserror ([451f883](https://github.com/uchouT/qb-downloader/commit/451f883))
- App error structure ([5e18bd7](https://github.com/uchouT/qb-downloader/commit/5e18bd7))
- Add more detailed error info ([0abaa13](https://github.com/uchouT/qb-downloader/commit/0abaa13))

### 🏡 Chore

- **release:** V2.1.1 ([6692086](https://github.com/uchouT/qb-downloader/commit/6692086))
- Reduce unnecessary String clone ([83adda9](https://github.com/uchouT/qb-downloader/commit/83adda9))
- App structure ([2d72498](https://github.com/uchouT/qb-downloader/commit/2d72498))
- Avoid unnecessary clone ([aa2108e](https://github.com/uchouT/qb-downloader/commit/aa2108e))
- Modify log format chore: modify log in config api ([d6f8b40](https://github.com/uchouT/qb-downloader/commit/d6f8b40))
- Reduce continuous TorrentNotFound log ([762a5e0](https://github.com/uchouT/qb-downloader/commit/762a5e0))

### 🤖 CI

- Generate changelog first ([0644ff2](https://github.com/uchouT/qb-downloader/commit/0644ff2))
- Use upx to compress executable ([c6dbb18](https://github.com/uchouT/qb-downloader/commit/c6dbb18))

## v2.1.2

[compare changes](https://github.com/uchouT/qb-downloader/compare/v2.1.1...v2.1.2)

### 💅 Refactors

- **request:** Wrap external request lib ([62bbf1d](https://github.com/uchouT/qb-downloader/commit/62bbf1d))

### 🏡 Chore

- **release:** V2.1.1 ([6692086](https://github.com/uchouT/qb-downloader/commit/6692086))
- Reduce unnecessary String clone ([83adda9](https://github.com/uchouT/qb-downloader/commit/83adda9))
- App structure ([2d72498](https://github.com/uchouT/qb-downloader/commit/2d72498))
- Avoid unnecessary clone ([aa2108e](https://github.com/uchouT/qb-downloader/commit/aa2108e))

### 🤖 CI

- Generate changelog first ([8dbc622](https://github.com/uchouT/qb-downloader/commit/8dbc622))

## v2.1.1

[compare changes](https://github.com/uchouT/qb-downloader/compare/v2.1.0...v2.1.1)

### 🩹 Fixes

- Panic when passing None seeding_time_limit or ratio_limit ([8daef6f](https://github.com/uchouT/qb-downloader/commit/8daef6f))

### 🏡 Chore

- Structure optimize ([4066ae5](https://github.com/uchouT/qb-downloader/commit/4066ae5))

### 🤖 CI

- Add changelog ([28a0d0a](https://github.com/uchouT/qb-downloader/commit/28a0d0a))

## v2.1.0

[compare changes](https://github.com/uchouT/qb-downloader/compare/v2.0.1...v2.1.0)

### 🩹 Fixes

- Extract info hash from magnet url omit base32 ([8368761](https://github.com/uchouT/qb-downloader/commit/8368761))
- QB login status wouldn't update after config change ([45af5f4](https://github.com/uchouT/qb-downloader/commit/45af5f4))

### 💅 Refactors

- Config ([9b89506](https://github.com/uchouT/qb-downloader/commit/9b89506))
- Task structure ([7fa62ad](https://github.com/uchouT/qb-downloader/commit/7fa62ad))

### 🏡 Chore

- Handle sigterm ([f22c674](https://github.com/uchouT/qb-downloader/commit/f22c674))

### 🤖 CI

- Launch action on tag v* ([aecd894](https://github.com/uchouT/qb-downloader/commit/aecd894))

## v2.0.0...v2.0.1

[compare changes](https://github.com/uchouT/qb-downloader/compare/v2.0.0...v2.0.1)

### 🚀 Enhancements

- Add support for qbittorrent version 4.1+ ([705b893](https://github.com/uchouT/qb-downloader/commit/705b893))

### 🩹 Fixes

- Http cancel, cleanup export ([2a44116](https://github.com/uchouT/qb-downloader/commit/2a44116))
- Panic when cancel adding torrent by url ([d07c894](https://github.com/uchouT/qb-downloader/commit/d07c894))

### 🤖 CI

- Add aarch64-unknown-linux-gnu target ([711d6c7](https://github.com/uchouT/qb-downloader/commit/711d6c7))

