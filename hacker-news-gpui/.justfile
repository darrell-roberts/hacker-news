set shell := ["fish", "-c"]

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

install:
    #!/usr/bin/env fish
    switch {{ os() }}
        case linux
            echo "Installing for linux"
            just install-linux
        case macos
            echo "Installing for macos"
            just bundle-macos
            open "dist/HackerNews.dmg"
        case '*'
            echo "Unsupported OS: {{ os() }}"
    end

install-linux: clean-dist linux-debian
    #!/usr/bin/env fish
    set -l deb (ls ../target/debian/hacker-news-dashboard*.deb)
    echo "Installing $deb"
    sudo apt reinstall "./$deb"
