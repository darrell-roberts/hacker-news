linux-debian:
    cargo deb -p hacker-news-tui

install-linux: linux-debian
    #!/usr/bin/env fish
    set -l deb (ls ../target/debian/hacker-news-tui*.deb)
    echo "Installing $deb"
    sudo apt reinstall "./$deb"

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
