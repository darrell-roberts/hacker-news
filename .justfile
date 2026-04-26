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

run:
    cargo run -p hacker-news-iced

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

install-linux *deb=`ls target/debian/hacker-news-reader*.deb`: clean-dist linux-debian
    echo "Installing {{ deb }}"
    sudo apt reinstall "./{{ deb }}"

bundle-macos: clean-dist build
    # Create necessary directories
    mkdir -p "dist/dmg"
    mkdir -p "dist/Hacker News.app/Contents/MacOS"
    mkdir -p "dist/Hacker News.app/Contents/Resources"

    # Copy application files
    cp assets/icon.icns "dist/Hacker News.app/Contents/Resources"
    cp assets/Info.plist "dist/Hacker News.app/Contents"
    cp target/release/hacker-news-reader "dist/Hacker News.app/Contents/MacOS"
    chmod +x "dist/Hacker News.app/Contents/MacOS/hacker-news-reader"

    # codesign --sign "MyApps" "dist/Hacker News.app"

    # Copy app to DMG staging area
    cp -r "dist/Hacker News.app" "dist/dmg"

    # Create temporary DMG
    hdiutil create -size 100m -fs HFS+ -volname "Hacker News" -o "dist/temp.dmg"

    # Mount temporary DMG
    hdiutil attach "dist/temp.dmg" -mountpoint "/Volumes/Hacker News"

    # Copy contents to DMG
    cp -r "dist/dmg/Hacker News.app" "/Volumes/Hacker News"

    # Create Applications shortcut
    ln -s /Applications "/Volumes/Hacker News/Applications"

    # Unmount
    hdiutil detach "/Volumes/Hacker News"

    # Convert to compressed DMG
    hdiutil convert "dist/temp.dmg" -format UDZO -imagekey zlib-level=9 -o "dist/HackerNews.dmg"

    # Clean up
    rm "dist/temp.dmg"
    # cd dist && zip -y "Hacker_News_aarch64.dmg.zip" "Hacker News.dmg"
