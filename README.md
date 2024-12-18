# My Hacker News Reader
- View top/best/new/ask/show/job stories.
- Each category is indexed locally.
- Read comments and nested comments.
- Search stories.
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

## Screenshot
### MacOS Dark mode theme
<img width="1840" alt="Screenshot 2024-12-17 at 7 45 09 PM" src="https://github.com/user-attachments/assets/51003ae5-f366-4f41-a7ef-05d17e520775" />

### Linux Light mode theme
![Screenshot from 2024-12-18 08-01-53](https://github.com/user-attachments/assets/a45f4328-dad2-4236-9230-c51b3b3f639d)




