# xKippo-tui

A terminal user interface for monitoring and managing Cowrie honeypots.

![xKippo-tui Dashboard](https://example.com/screenshot.png)

## Features

- Real-time monitoring of Cowrie honeypot logs
- Interactive dashboard with activity overview
- Detailed session analysis and command history
- Geographic visualization of attack sources
- Advanced filtering and search capabilities
- Configurable alerts for suspicious activities
- Session replay functionality
- Extensive configuration options

## Installation

### Requirements

- Rust 1.56 or later
- A Cowrie honeypot installation

### Building from source

```bash
# Clone the repository
git clone https://github.com/yourusername/xKippo-tui.git
cd xKippo-tui

# Build the project
cargo build --release

# Install (optional)
cargo install --path .
```

## Usage

```bash
# Run with default settings
xkippo-tui

# Specify a custom configuration file
xkippo-tui -c /path/to/config.toml

# Run setup script to configure
xkippo-tui --setup

# Enable verbose logging
xkippo-tui -vv
```

### Key bindings

- `Tab` / `Shift+Tab`: Navigate between tabs
- `1-4`: Select tab directly
- `q`: Quit the application
- `?`: Show help dialog

#### Logs view
- `↑`/`↓`: Navigate logs
- `Enter`: View details
- `Esc`: Close details
- `/`: Search logs

#### Sessions view
- `↑`/`↓`: Navigate sessions
- `Enter`: View session details
- `c`/`f`: Switch between Commands and Files tabs

## Configuration

The configuration file is located at `~/.config/xkippo/config.toml` by default. You can specify a different location with the `-c` option.

See the [example configuration](config.toml) for all available options.

### Common log locations

xKippo-tui will automatically try to detect Cowrie log files in common locations:

- `/var/log/cowrie/cowrie.json`
- `/opt/cowrie/var/log/cowrie/cowrie.json`
- `/home/cowrie/cowrie/var/log/cowrie/cowrie.json`
- `/usr/local/cowrie/var/log/cowrie/cowrie.json`

You can also specify log paths manually in the configuration file.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Cowrie Honeypot](https://github.com/cowrie/cowrie) - SSH/Telnet honeypot
- [ratatui](https://github.com/ratatui-org/ratatui) - TUI library for Rust