PLATFORM := $(shell uname)

all: build

clean-dist:
	rm -rf dist/

check:
	cargo clippy

build: check
	cargo build --release  --bin hacker-news-iced

bundle-mac: clean-dist build
	# Create necessary directories
	mkdir -p "dist/dmg"
	mkdir -p "dist/Hacker News.app/Contents/MacOS"
	mkdir -p "dist/Hacker News.app/Contents/Resources"

	# Copy application files
	cp assets/icon.icns "dist/Hacker News.app/Contents/Resources"
	cp assets/Info.plist "dist/Hacker News.app/Contents"
	cp target/release/hacker-news-iced "dist/Hacker News.app/Contents/MacOS"
	chmod +x "dist/Hacker News.app/Contents/MacOS/hacker-news-iced"

	codesign --sign "Darrell Roberts" "dist/Hacker News.app"

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
	hdiutil convert "dist/temp.dmg" -format UDZO -imagekey zlib-level=9 -o "dist/Hacker News.dmg"

	# Clean up
	rm "dist/temp.dmg"
	cd dist && zip -y "Hacker_News.zip" "Hacker News.dmg"

linux-app-image: clean-dist build
	echo "Building linux app image"
	rm -rf dist/AppDir
	# create new AppDir
	linuxdeploy-x86_64.AppImage --appdir dist/AppDir

	# Copy contents into AppDir
	cp target/release/hacker-news-iced dist/AppDir/usr/bin
	cp assets/hacker-news.desktop dist/AppDir/usr/share/applications
	tar zxvf assets/icons.tar.gz -C dist/AppDir/usr/share

	# Create app image
	linuxdeploy-x86_64.AppImage --appdir dist/AppDir --output appimage

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
