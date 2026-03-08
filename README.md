# 🦀 Krabby

A modern, beautiful container management utility for Linux, built with Rust.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)

## Features

- 🎯 **System Tray Integration** - Quick access from your system tray
- 📊 **Container Management** - View, sort, filter, start, stop, and restart containers
- 📈 **Live Stats** - CPU and memory usage for running containers
- 🎨 **Beautiful Themes** - Multiple color themes (Midnight, Ocean, Forest, Rose, Amber)
- 📄 **Export** - Export container data as CSV or JSON
- 🐳 **Compose Generation** - Generate docker-compose files from running containers
- 🔔 **Notifications** - Get notified of container events
- ⚡ **Fast & Lightweight** - Built with Rust for performance

## Screenshots

*(Screenshots will be added soon)*

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/krabby.git
cd krabby

# Build and install
cargo build --release
sudo cp target/release/krabby /usr/local/bin/
```

### Prerequisites

- Rust 1.85 or later
- Docker (for container management)
- Linux with GTK3 support (for system tray)

### RPM Package (Fedora/RHEL)

```bash
# Download the RPM from releases
sudo dnf install ./krabby-*.x86_64.rpm
```

## Usage

### Starting the Application

```bash
krabby
```

The application will:
1. Start a system tray icon
2. Open the main window
3. Automatically refresh container data every 5 seconds

Click the 🦀 icon in your system tray to show/hide the window.

### Keyboard Shortcuts

- `Ctrl+R` - Refresh containers
- `Ctrl+F` - Focus search box
- `Ctrl+Q` - Quit application

## Configuration

Configuration is stored at `~/.config/krabby/config.toml`:

```toml
theme = "midnight"
auto_refresh_interval = 5
enable_notifications = true
check_image_updates = false

[columns]
name = { visible = true, width = 200 }
image = { visible = true, width = 250 }
status = { visible = true, width = 120 }
```

## Themes

| Theme | Preview |
|-------|---------|
| Midnight | Deep violet/purple on dark background |
| Ocean | Blue on deep blue background |
| Forest | Green on deep green background |
| Rose | Pink on deep burgundy background |
| Amber | Gold on deep brown background |

## Roadmap

- [x] Basic container management
- [x] System tray integration
- [x] Multiple themes
- [x] CSV/JSON export
- [x] Compose file generation
- [ ] Image update checking
- [ ] Podman support
- [ ] Container logs viewer
- [ ] Resource usage graphs
- [ ] Container recreation GUI

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Iced](https://iced.rs/) - A cross-platform GUI library for Rust
- Docker integration via [Bollard](https://github.com/fussybeaver/bollard)
- Inspired by [Portainer](https://www.portainer.io/) and [Lazydocker](https://github.com/jesseduffield/lazydocker)

---

🦀 Built with Rust, for Linux container enthusiasts!
