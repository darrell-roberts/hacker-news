all: bundle-mac

clean-dist:
	rm -rf dist/

check:
	cargo check 

build: check
	cargo build --release  --bin hacker-news-egui

bundle-mac: clean-dist build
	mkdir -p dist
	mkdir -p dist/HackerNews.app/Contents/MacOS
	mkdir -p dist/HackerNews.app/Contents/Resources
	cp hacker-news-tauri/src-tauri/icons/icon.icns dist/HackerNews.app/Contents/Resources
	cp hacker-news-egui/assets/Info.plist dist/HackerNews.app/Contents
	cp target/release/hacker-news-egui dist/HackerNews.app/Contents
	hdiutil create -fs HFS+ -volname "Hacker News" -srcfolder "dist/HackerNews.app" "dist/HackerNews.dmg"


