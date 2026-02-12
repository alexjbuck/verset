# Verset

Universal changeset management for all languages.

## Installation

```bash
# Install with uv (recommended)
uv tool install verset

# Or with pip
pip install verset
```

The appropriate pre-compiled binary for your platform is bundled with the wheel, so no additional download is required.

## Usage

After installation, you can use verset from the command line:

```bash
# Initialize verset in your project
verset init

# Add a changeset
verset add

# Check status
verset status

# Apply changesets
verset apply
```

## How it Works

This package distributes platform-specific wheels, each containing the pre-compiled verset binary for that platform. When you install verset, pip automatically selects and downloads the appropriate wheel for your system.

The binary is installed to your Python environment's scripts directory, making it available in your PATH when the environment is activated.

## Environment Support

- **Virtual Environments**: When installed in a virtual environment, the binary is installed to the virtual environment's Scripts/bin directory
- **System Python**: When installed globally, the binary is installed to the system Python's Scripts/bin directory
- **uv**: Fully compatible with uv package manager and can be installed as a tool

## Supported Platforms

Platform-specific wheels are available for:
- macOS x64 (Intel)
- macOS arm64 (Apple Silicon)
- Linux x64
- Linux arm64
- Windows x64

## Building from Source

If a pre-built wheel is not available for your platform, you can build from source:

```bash
# Clone the repository
git clone https://github.com/alexjbuck/verset.git
cd verset

# Build the Rust binary
cargo build --release

# Install the Python package
cd deploy/python
pip install -e .
```

For more information, visit [https://github.com/alexjbuck/verset](https://github.com/alexjbuck/verset) 