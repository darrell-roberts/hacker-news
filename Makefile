PLATFORM := $(shell uname)

all: build

clean-dist:
	rm -rf dist/

check:
	cargo clippy

build: check
	cargo build --release  --bin hacker-news-iced

bundle-mac: clean-dist build
	mkdir -p "dist/Hacker_News.app/Contents/MacOS"
	mkdir -p "dist/Hacker_News.app/Contents/Resources"
	cp assets/icon.icns "dist/Hacker_News.app/Contents/Resources"
	cp assets/Info.plist "dist/Hacker_News.app/Contents"
	cp target/release/hacker-news-iced "dist/Hacker_News.app/Contents/MacOS"
	hdiutil create -fs HFS+ -volname "Hacker News" -srcfolder "dist/Hacker_News.app" "dist/Hacker_News.dmg"

install-local-linux: build
	echo "Installing for linux"
	mkdir -p ~/.local/share/applications
	mkdir -p ~/.local/bin
	cp target/release/hacker-news-iced ~/.local/bin
	cp assets/hacker-news.desktop ~/.local/share/applications
	tar zxvf assets/icons.tar.gz -C ~/.local/share

install:
ifeq ($(PLATFORM), Darwin)
	@echo "Installing for Mac"
	@$(MAKE) bundle-mac
	open "dist/Hacker News.dmg"
else ifeq ($(PLATFORM), Linux)
	@echo "Installing for Linux"
	@$(MAKE) install-local-linux
else
	@echo "Unsupported platform for install: " $(PLATFORM)
endif

# Starts the jaeger all-in-one docker container.
trace:
	docker compose up --detach --remove-orphans --wait
	cargo run --bin hacker-news-iced --features trace

# Stops the jaeger all-in-one docker container.
trace-down:
	docker compose down

.PHONY: all clean-dist check build bundle-mac install-local-linux install trace trace-down
