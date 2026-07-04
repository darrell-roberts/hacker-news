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
            open "dist/HackerNewsDashboard.dmg"
        case '*'
            echo "Unsupported OS: {{ os() }}"
    end

install-linux: clean-dist linux-debian
    #!/usr/bin/env fish
    set -l deb (ls ../target/debian/hacker-news-dashboard*.deb)
    echo "Installing $deb"
    sudo apt reinstall "./$deb"

bundle-macos: clean-dist build
    mkdir -p "dist/dmg"
    mkdir -p "dist/Hacker News Dashboard.app/Contents/MacOS"
    mkdir -p "dist/Hacker News Dashboard.app/Contents/Resources"

    # Copy application files
    cp ../assets/icon.icns "dist/Hacker News Dashboard.app/Contents/Resources"
    cp Info.plist "dist/Hacker News Dashboard.app/Contents"
    cp ../target/release/hacker-news-dashboard "dist/Hacker News Dashboard.app/Contents/MacOS"
    chmod +x "dist/Hacker News Dashboard.app/Contents/MacOS/hacker-news-dashboard"

    # codesign --sign "MyApps" "dist/Hacker News Dashboard.app"

    # Copy app to DMG staging area
    cp -r "dist/Hacker News Dashboard.app" "dist/dmg"

    # Create temporary DMG
    hdiutil create -size 100m -fs HFS+ -volname "Hacker News Dashboard" -o "dist/temp.dmg"

    # Mount temporary DMG
    hdiutil attach "dist/temp.dmg" -mountpoint "/Volumes/Hacker News Dashboard"

    # Copy contents to DMG
    cp -r "dist/dmg/Hacker News Dashboard.app" "/Volumes/Hacker News Dashboard"

    # Create Applications shortcut
    ln -s /Applications "/Volumes/Hacker News Dashboard/Applications"

    # Unmount
    hdiutil detach "/Volumes/Hacker News Dashboard"

    # Convert to compressed DMG
    hdiutil convert "dist/temp.dmg" -format UDZO -imagekey zlib-level=9 -o "dist/HackerNewsDashboard.dmg"

    # Clean up
    rm "dist/temp.dmg"
