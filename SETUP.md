# FlowVersion Setup Guide

## Installing Rust (Required)

FlowVersion is written in Rust and requires the Rust toolchain to compile and run.

### Windows Installation

1. **Download Rustup (recommended method):**
   - Visit https://rustup.rs/
   - Download `rustup-init.exe`
   - Run the installer and follow the prompts

2. **Alternative: Using chocolatey:**
   ```powershell
   choco install rust
   ```

3. **Alternative: Using winget:**
   ```powershell
   winget install Rust.Rustup
   ```

4. **Windows C++ Build Tools (MSVC) — required for default toolchain:**
    Rust の既定ターゲット（x86_64-pc-windows-msvc）では、Microsoft のリンカ `link.exe` が必要です。ビルド時に
    `link.exe not found` と出た場合は、次のいずれかで Visual Studio Build Tools を導入してください。

    - winget（推奨・管理者 PowerShell）
       ```powershell
       winget install --id Microsoft.VisualStudio.2022.BuildTools -e
       # サイレントで C++ ツール一式（推奨）
       winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
       ```

    - 公式サイトからセットアップ（"Desktop development with C++" / "C++ build tools" を選択）
       https://visualstudio.microsoft.com/ja/downloads/

    インストール後は「Developer PowerShell for VS 2022」を開くか、新しいターミナルを起動してからビルドしてください。

    代替案: MSVC を入れたくない場合は GNU ツールチェーンへ切替可能です。
    ```powershell
    rustup toolchain install stable-x86_64-pc-windows-gnu
    rustup default stable-x86_64-pc-windows-gnu
    ```

### Verify Installation

After installation, open a new terminal and run:
```bash
rustc --version
cargo --version
```

Both commands should display version information.

## Building FlowVersion

1. **Clone or ensure you have the FlowVersion source code**
2. **Navigate to the project directory:**
   ```bash
   cd C:\dev\FlowVersion
   ```

3. **Build the project:**
   ```bash
   cargo build --release
   ```

4. **The executable will be created at:**
   ```
   target/release/flow.exe  (Windows)
   target/release/flow      (Linux/macOS)
   ```

## Running Tests

Run the test suite to ensure everything works correctly:
```bash
cargo test
```

## Optional: Install Globally

To use FlowVersion from anywhere on your system:
```bash
cargo install --path .
```

This installs the `flow` command globally.

## Quick Start

1. **Initialize a new repository:**
   ```bash
   flow init --name "my-project"
   ```

2. **Add files:**
   ```bash
   flow add main.rs --intention "Add main application file"
   ```

3. **Create a commit:**
   ```bash
   flow commit --intention "Initial implementation" --confidence 0.8
   ```

4. **View history:**
   ```bash
   flow log
   ```

## Troubleshooting

### Common Issues

**"cargo: command not found"**
- Ensure Rust is properly installed
- Restart your terminal after installation
- Check that `~/.cargo/bin` (or equivalent) is in your PATH

**Build failures**
- Ensure you have the latest stable Rust version: `rustup update`
- Check for missing system dependencies
- On Linux, you may need: `build-essential`, `pkg-config`, `libssl-dev`

**Permission errors**
- On Windows, run terminal as Administrator if needed
- On Linux/macOS, check file permissions

### Getting Help

- Run `flow --help` for command usage
- Check the project's README.md for detailed documentation
- Review test_scenario.md for usage examples

## Development Setup

If you plan to contribute to FlowVersion:

1. **Install additional tools:**
   ```bash
   rustup component add clippy rustfmt
   cargo install cargo-watch
   ```

2. **Run development checks:**
   ```bash
   cargo clippy          # Linting
   cargo fmt             # Formatting
   cargo test            # Tests
   cargo bench           # Benchmarks
   ```

3. **Continuous development:**
   ```bash
   cargo watch -x test   # Re-run tests on file changes
   ```

This completes the setup process for FlowVersion development and usage.