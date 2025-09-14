PLATFORM := $(shell uname)

all: build

clean-dist:
	rm -rf dist/

check:
	cargo clippy

build: check
	cargo build --release  --bin hacker-news-dashboard

bundle-mac: clean-dist build
	# Create necessary directories
	mkdir -p "dist/dmg"
	mkdir -p "dist/Hacker News Dashboard.app/Contents/MacOS"
	mkdir -p "dist/Hacker News Dashboard.app/Contents/Resources"

	# Copy application files
	cp ../assets/icon.icns "dist/Hacker News Dashboard.app/Contents/Resources"
	cp Info.plist "dist/Hacker News Dashboard.app/Contents"
	cp ../target/release/hacker-news-dashboard "dist/Hacker News Dashboard.app/Contents/MacOS"
	chmod +x "dist/Hacker News Dashboard.app/Contents/MacOS/hacker-news-dashboard"

	codesign --sign "MyApps" "dist/Hacker News Dashboard.app"

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
	# cd dist && zip -y "Hacker_News_aarch64.dmg.zip" "Hacker News.dmg"

# linux-app-image: clean-dist build
# 	echo "Building linux app image"
# 	rm -rf dist/AppDir
# 	# create new AppDir
# 	linuxdeploy-x86_64.AppImage --appdir dist/AppDir

# 	# Copy contents into AppDir
# 	cp target/release/hacker-news-iced dist/AppDir/usr/bin
# 	cp assets/io.github.darrellroberts.hacker-news.desktop dist/AppDir/usr/share/applications
# 	tar zxvf assets/icons.tar.gz -C dist/AppDir/usr/share

# 	# Create app image
# 	linuxdeploy-x86_64.AppImage --appdir dist/AppDir --output appimage

# linux-flatpak:
# 	python3 ./flatpak-cargo-generator.py Cargo.lock -o cargo-sources.json
# 	flatpak-builder --force-clean --user --install-deps-from=flathub --repo=repo --install builddir io.github.darrellroberts.hacker-news.yml
# 	flatpak build-bundle repo hacker-news.flatpak io.github.darrellroberts.hacker-news

linux-debian: clean-dist build
	mkdir -p dist
	tar zxvf assets/dashboard-icons.tar.gz -C dist
	cargo deb -p hacker-news-gpui

install-local-linux: linux-debian
	echo "Installing for linux"

# 	mkdir -p ~/.local/share/applications
# 	mkdir -p ~/.local/bin
# 	cp target/release/hacker-news-iced ~/.local/bin
# 	cp assets/io.github.darrellroberts.hacker-news.desktop ~/.local/share/applications
# 	tar zxvf assets/icons.tar.gz -C ~/.local/share

install:
ifeq ($(PLATFORM), Darwin)
	@echo "Installing for Mac"
	@$(MAKE) bundle-mac
	open "dist/HackerNewsDashboard.dmg"
else ifeq ($(PLATFORM), Linux)
	@echo "Installing for Linux"
	@$(MAKE) install-local-linux
else
	@echo "Unsupported platform for install: " $(PLATFORM)
endif

# uninstall-linux:
# 	rm -f ~/.local/share/applications/io.github.darrellroberts.hacker-news.desktop
# 	fd io.github.darrellroberts.hacker-news ~/.local/share/icons | xargs rm -f
# 	rm -f ~/.local/bin/hacker-news-iced

# # Starts the jaeger all-in-one docker container.
# trace:
# 	docker compose up --detach --remove-orphans --wait
# 	cargo run --bin hacker-news-iced --features trace

# # Stops the jaeger all-in-one docker container.
# trace-down:
# 	docker compose down

.PHONY: all clean-dist check build bundle-mac install-local-linux install trace trace-down linux-flatpak uninstall-linux
