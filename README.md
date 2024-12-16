# My Hacker News Reader
- View top/best/new/ask/show/job stories.
- Each category is indexed locally.
- Read comments and nested comments.
- Search comments.

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
cargo install --path hacker-news-iced --bin hacker-news-iced
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
