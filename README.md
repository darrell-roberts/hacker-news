# Hacker News Reader
- View top/best/new/ask/show/job stories.
- Search text in title.
- Read comments and nested comments.
- View user information.
- Highlights Rust articles with badge.
- Track visisted/Filter visisted/Reset visisted.
- Filter by article type (job, poll, story).

## Install prerequisites
Each install method will build and package from source. You'll first need to clone this repo.

```bash
git clone https://github.com/darrell-roberts/hacker-news.git
cd hacker-news
```

You'll need the Rust compiler [toolchain](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Install as binary

```bash
cargo install --path hacker-news-egui --bin hacker-news-egui
```
## Install as Mac app
Run the provided `Makefile`


```bash
make
```

This will create a `dist/Hacker News.dmg` file and open/mount it. Simply copy the contents into your `Application` folder.

### Articles
<img width="909" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/c2d0eb69-76a8-434a-9ada-3e4a7988eac8">

### Title search
<img width="953" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/53ed8fdc-63f6-461c-9d9c-f7df3a2c930b">

### View comments
<img width="953" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/dad05444-2278-4ce3-99c2-3eb74fa5ff17">
