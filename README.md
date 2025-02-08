# My Hacker News Reader
- View top/best/new/ask/show/job stories.
- Each category is indexed locally.
- Read comments and nested comments.
- Search stories.
- Search comments.
- Watch stories via server side events.

# Install Options
## MacOS (ARM)
Download [prebuilt dmg](https://github.com/darrell-roberts/hacker-news/releases) from releases.

Will require allowing non app store & known developers when launching. Open System Settings -> Privacy & Security and under Security allow running app.

## Linux (X86_64)
Download Linux app image, flatpak or debian package [from releases](https://github.com/darrell-roberts/hacker-news/releases).

### App image
Unzip and grant execute permission to the app image and [run it](https://docs.appimage.org/user-guide/faq.html#question-how-do-i-run-an-appimage).

### Flatpak
Download the `hacker-news.flatpak` file and run `flatpak install hacker-news.flatpak`.

## Build from source.

### Build prerequisites
Each install method will build and package from source. You'll first need to clone this repo.

```bash
git clone https://github.com/darrell-roberts/hacker-news.git
cd hacker-news
```

You'll need the Rust compiler [toolchain](https://rustup.rs/).

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build & Install as binary

```bash
cargo install --path hacker-news-iced --bin hacker-news-iced
```
### Build & Install as Mac app
Run the provided `Makefile` target

```bash
make install
```

This will create a `dist/Hacker News.dmg` file and open/mount it. Simply copy the contents into your `Application` folder.

### Build & Install as Linux Desktop App.
Run the provided `Makefile` target

```bash
make install
```

This copies the binary and other assets into your `~/.local`.

# Screenshots
### MacOS dark mode theme
<img width="1786" alt="Screenshot 2025-02-08 at 3 55 25 PM" src="https://github.com/user-attachments/assets/5f572710-bddd-419f-a10a-081df83c1093" />


### Linux light mode theme
<img width="1786" alt="Screenshot 2025-02-08 at 3 55 25 PM" src="https://github.com/user-attachments/assets/1ba09e03-efa1-4d92-ae46-7d79231db45c" />







