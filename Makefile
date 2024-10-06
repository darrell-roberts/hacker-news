PLATFORM := $(shell uname)

all: build

clean-dist:
	rm -rf dist/

check:
	cargo clippy

build: check
	# cargo build --release  --bin hacker-news-egui
	cargo build --release  --bin hacker-news-iced

bundle-mac: clean-dist build
	mkdir -p "dist/Hacker News.app/Contents/MacOS"
	mkdir -p "dist/Hacker News.app/Contents/Resources"
	cp assets/icon.icns "dist/Hacker News.app/Contents/Resources"
	cp assets/Info.plist "dist/Hacker News.app/Contents"
	# cp target/release/hacker-news-egui "dist/Hacker News.app/Contents/MacOS"
	cp target/release/hacker-news-iced "dist/Hacker News.app/Contents/MacOS"
	hdiutil create -fs HFS+ -volname "Hacker News" -srcfolder "dist/Hacker News.app" "dist/Hacker News.dmg"
	open "dist/Hacker News.dmg"

install-local-linux: build
	echo "Installing for linux"
	mkdir -p ~/.local/share/applications
	mkdir -p ~/.local/bin
	# cp target/release/hacker-news-egui ~/.local/bin
	cp target/release/hacker-news-iced ~/.local/bin
	cp assets/hacker-news.desktop ~/.local/share/applications
	tar zxvf assets/icons.tar.gz -C ~/.local/share

install:
ifeq ($(PLATFORM), Darwin)
	@echo "Installing for Mac"
	@$(MAKE) bundle-mac
else ifeq ($(PLATFORM), Linux)
	@echo "Installing for Linux"
	@$(MAKE) install-local-linux
else
	@echo "Unsupported platform for install: " $(PLATFORM)
endif

.PHONY: all clean-dist check build bundle-mac install-local-linux install
