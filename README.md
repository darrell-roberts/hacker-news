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
<img width="1840" alt="Screenshot 2024-12-19 at 5 04 05 PM" src="https://github.com/user-attachments/assets/b69fc62e-e408-4c98-be89-9b5bb3a8d10e" />

### Linux Light mode theme
![Screenshot from 2024-12-19 15-03-21](https://github.com/user-attachments/assets/6cf0cc19-64ff-49de-aece-f52bc60afcda)

### Windows Dark mode theme
<img width="1248" alt="Screenshot 2024-12-18 093635" src="https://github.com/user-attachments/assets/99d17244-6c7a-4c44-b130-9ed0cf95dcfc" />



