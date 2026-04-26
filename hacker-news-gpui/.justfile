clean-dist:
    rm -rf dist/

check:
    cargo clippy

build: check
    cargo build --release --bin hacker-news-dashboard

linux-debian: build
    mkdir -p dist
    tar zxvf assets/dashboard-icons.tar.gz -C dist
    cargo deb -p hacker-news-gpui
