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
<img width="1840" alt="Screenshot 2024-12-17 at 5 04 22 PM" src="https://github.com/user-attachments/assets/79d44bb6-f507-41b1-92a3-d92719604454" />

### Linux Light mode theme
![Screenshot from 2024-12-16 15-47-32](https://github.com/user-attachments/assets/328cc63a-6a16-4ed1-8bfb-b8baf62206dd)



