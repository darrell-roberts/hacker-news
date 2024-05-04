# Hacker News Reader
- View top/best/new/ask/show/job stories.
- Search text in title.
- Read comments and nested comments.
- View user information.
- Highlights Rust articles with badge.
- Track visisted/Filter visisted/Reset visisted.
- Filter by article type (job, poll, story).
- Adjust zoom (font sizes) with ctrl + and ctrl - keys.
- Open Search via cmd + f, close via escape.

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
Run the provided `Makefile` target


```bash
make install
```

This will create a `dist/Hacker News.dmg` file and open/mount it. Simply copy the contents into your `Application` folder.

## Install as Linux Desktop App.
Run the provided `Makefile` target

```bash
make install
```

This copies the binary and other assets into your `~/.local`.

## Screenshots

### Articles
<img width="1159" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/045db0d1-fcb8-4b43-9954-2af0256676e1">


### Title search
<img width="1159" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/819ac36f-7300-45f6-8b08-72dc2a459b84">


### View comments
<img width="1159" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/04be44ed-8532-497c-b265-e33995445a61">

