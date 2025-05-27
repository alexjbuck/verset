# Verset CLI

Universal changeset management for all languages.

## Installation

### Global Installation

```bash
# With npm
npm install -g @verset/cli

# With Yarn Berry
yarn global add @verset/cli

# With pnpm
pnpm add -g @verset/cli
```

### Local Installation (Project-specific)

```bash
# With npm
npm install --save-dev @verset/cli

# With Yarn Berry
yarn add -D @verset/cli

# With pnpm
pnpm add -D @verset/cli
```

## How it Works

This package uses a dual approach for maximum reliability:

1. **optionalDependencies**: Platform-specific packages (e.g., `@verset/darwin-x64`) are listed as optional dependencies. Package managers automatically install only the package matching your OS and architecture.

2. **postinstall fallback**: If optionalDependencies are disabled (e.g., with `--no-optional`), the postinstall script downloads the binary from npm or GitHub releases.

This approach ensures the binary is available even in restricted environments where optionalDependencies or postinstall scripts might be disabled.

## Usage

### Global Installation
After global installation, you can use verset directly:

```bash
verset init
verset add
verset status
verset apply
```

### Local Installation
For local installations, use one of these methods:

```bash
# Using yarn
yarn verset init

# Using npx
npx verset init

# Or add to package.json scripts
{
  "scripts": {
    "changeset": "verset add",
    "version": "verset apply"
  }
}
```

## Troubleshooting

If the verset binary is not found:

1. **Enable optionalDependencies**: If you installed with `--no-optional`, try reinstalling:
   ```bash
   npm install --no-save --no-optional=false @verset/cli
   ```

2. **Check postinstall**: Ensure postinstall scripts are enabled in your environment.

3. **Manual download**: Download the binary manually from [GitHub releases](https://github.com/alexjbuck/verset/releases).

## Architecture Support

The following platform-specific packages are available:
- `@verset/darwin-x64` - macOS Intel
- `@verset/darwin-arm64` - macOS Apple Silicon
- `@verset/linux-x64` - Linux x64
- `@verset/linux-arm64` - Linux ARM64
- `@verset/win32-x64` - Windows x64

## For Package Maintainers

This package follows the pattern described in [Sentry's blog post on publishing binaries on npm](https://sentry.engineering/blog/publishing-binaries-on-npm).

For more information, visit [https://github.com/alexjbuck/verset](https://github.com/alexjbuck/verset) 