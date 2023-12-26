# Hacker News Reader
This repo has two user interfaces you can build. One is a tauri desktop app and the other is an egui/eframe desktop app. There are shared crates that support making api calls to the hacker news firebase backend and for parsing the payloads.


## hacker-news Tauri
Hacker news top stories dashboard reader.

- Subscribes to firebase hacker news top stories event-source api.
- Shows ranking updates as put events arrive.
- Read comments and nested comments.
- View item author information.
- Highlights rust articles with badge.

<img width="2049" alt="Screenshot 2023-10-23 at 1 33 27 PM" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/72a1626b-a097-4f23-8e3b-289880269c20">

### Install
Setup [Tauri build](https://tauri.app/v1/guides/getting-started/prerequisites).

Build Tauri App.

```bash
cargo tauri build
```

## hacker-news egui
- View top/best/new stories.
- Search text in title.
- Read comments and nested comments.
- View user information.
- Highlights Rust articles with badge.
- Track visisted/Filter visisted/Reset visisted.

### Install prerequisites
Each install method will build and package from source. You'll first need to clone this repo.

```bash
git clone https://github.com/darrell-roberts/hacker-news.git
cd hacker-news
```

You'll need the Rust compiler [toolchain](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install as binary

```bash
cargo install --path hacker-news-egui --bin hacker-news-egui
```
### Install as Mac app
Run the provided `Makefile`


```bash
make
```

This will create a `dist/Hacker News.dmg` file and open/mount it. Simply copy the contents into your `Application` folder.

### Articles
<img width="1057" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/0c245a88-dd2a-4ae3-9b61-20d262fc1477">

### Title search
<img width="1057" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/cc15b4bc-7f29-449b-b1b6-fafd0c064942">

### View comments
<img width="1057" alt="image" src="https://github.com/darrell-roberts/hacker-news/assets/33698065/eed0e011-a9b3-4462-aa67-71153f1321d6">



