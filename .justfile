set shell := ["fish", "-c"]

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

install:
    #!/usr/bin/env fish
    switch {{ os() }}
        case linux
            echo "Installing for linux"
            just install-linux
        case macos
            echo "Installing for macos"
        case '*'
            echo "Unsupported OS: {{ os() }}"
    end

deb := `ls target/debian/hacker-news-reader*.deb`

install-linux: linux-debian
    # deb := `ls target/debian/*.deb`
    echo "Installing {{ deb }}"
