clean-dist:
    rm -rf dist/

check:
    cargo clippy

build: check
    cargo build --release --bin hacker-news-reader

test:
    cargo test

taplo_fmt *args='':
    rg --files -g 'Cargo.toml' -g 'taplo.toml' | sort -u | xargs taplo fmt {{ args }}

fmt: taplo_fmt
    cargo fmt

linux-debian: build
    mkdir -p dist
    tar zxvf assets/icons.tar.gz -C dist
    cargo deb -p hacker-news-iced
