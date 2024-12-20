# My Hacker News Reader
- View top/best/new/ask/show/job stories.
- Each category is indexed locally.
- Read comments and nested comments.
- Search stories.
- Search comments.
- Watch stories via server side events.

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
### MacOS Dark mode theme
<img width="1840" alt="Screenshot 2024-12-20 at 5 17 14 PM" src="https://github.com/user-attachments/assets/30d5d7e2-645d-4838-8e9e-b073d1c0745e" />

### Linux Light mode theme
![Screenshot from 2024-12-20 13-00-27](https://github.com/user-attachments/assets/1340c952-ec0c-4fb7-b5f9-368cd3aa326d)

### Windows Dark mode theme
![Screenshot 2024-12-20 130915](https://github.com/user-attachments/assets/8477397a-dc70-4861-ab8c-4eaf6d2dfc54)




