all: bundle-mac

clean-dist:
	rm -rf dist/

check:
	cargo clippy

build: check
	cargo build --release  --bin hacker-news-egui

bundle-mac: clean-dist build
	mkdir -p "dist/Hacker News.app/Contents/MacOS"
	mkdir -p "dist/Hacker News.app/Contents/Resources"
	cp hacker-news-egui/assets/icon.icns "dist/Hacker News.app/Contents/Resources"
	cp hacker-news-egui/assets/Info.plist "dist/Hacker News.app/Contents"
	cp target/release/hacker-news-egui "dist/Hacker News.app/Contents/MacOS"
	hdiutil create -fs HFS+ -volname "Hacker News" -srcfolder "dist/Hacker News.app" "dist/Hacker News.dmg"
	open "dist/Hacker News.dmg"
