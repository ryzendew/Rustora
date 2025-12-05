# Technical Details

Technical information about Rustora's architecture and implementation.

## Architecture

- **Language**: Rust
- **GUI Framework**: Iced 0.12
- **Package Management**: DNF (via command-line interface)
- **Async Runtime**: Tokio
- **Serialization**: Serde

## Data Storage

- **Settings**: `~/.config/rustora/settings.json`
- **Cache**: `~/.cache/rustora/proton_builds.json`
- **Themes**: `~/.config/rustora/themes/*.json`

## Performance

- **Caching**: Proton/Wine builds cached locally for fast loading
- **Background updates**: Cache updated in background without blocking UI
- **Debounced search**: Optimized search performance
- **Lazy loading**: Tabs load data only when accessed

## Development

Want to contribute or just build it yourself?

```bash
# Build in debug mode
cargo build

# Run with GUI
cargo run

# Run with specific command
cargo run -- search firefox

# Run tests
cargo test

# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

## System Requirements

- **OS**: Fedora 38+ or Nobara (Fedora-based)
- **Architecture**: x86_64
- **Display**: 720p minimum (higher is better)
- **Dependencies**: See [Installation Guide](INSTALLATION.md) for full list

