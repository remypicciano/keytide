# Coffer

Coffer is a local-first desktop application for protecting files with a separate unlock key. Protection and restoration happen on-device using the documented authenticated container format. There is no account, cloud sync, or recovery backdoor.

Version 1.0.0 is the first stable release. Download release archives from [GitHub Releases](https://github.com/remypicciano/coffer/releases) and verify the included SHA-256 checksum before use.

Visit the [project site](https://remypicciano.github.io/coffer/) for the security model, platform builds, and format roadmap.

## Usage

1. Run the app with `cargo run`.
2. Choose a file to protect.
3. Save the generated container and unlock key in separate places.
4. Use the matching key to open the container later.

For the public-facing workflow and contact details, see [the site contact page](https://remypicciano.github.io/coffer/contact.html).

## Developer conventions

- Use a focused branch for any change set.
- Keep commits intentional and concise.
- Update docs and the site together when behavior or messaging changes.
- Never commit real keys, containers, plaintext samples, credentials, logs, or generated binaries.
- Prefer synthetic fixtures for tests and examples.
- Use these checks before opening a change:

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo audit
python3 tools/check_docs.py
python3 tools/check_site.py
python3 -m py_compile tools/*.py
```

## Build notes

### macOS

```sh
cargo build --release
cp target/release/coffer dist/macos/Coffer.app/Contents/MacOS/coffer
open dist/macos/Coffer.app
```

### Windows x64

```sh
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

### Linux x64

```sh
sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libxkbcommon-dev
cargo build --locked --release
./target/release/coffer
```

## Repository map

- [Application source](src/)
- [Site](site/)
- [Docs](docs/)
- [Workflow helpers](tools/)
- [Security policy](SECURITY.md)
- [Support policy](SUPPORT.md)
- [Contributing](CONTRIBUTING.md)
- [Public readiness review](docs/public-readiness.md)

## Name ideas

If you ever want to rebrand the repository, these are stronger and more distinctive options than a generic product name:

- Vaultline
- SplitKey
- KeyHarbor
- Latchroom
- CipherDock
- AnchorVault
- Lockframe
- SafeSep
- StorePair
- HarborKey

## Security and license

Report vulnerabilities privately through [GitHub Security Advisories](https://github.com/remypicciano/coffer/security/advisories/new). Never attach real keys, private containers, or confidential plaintext to an issue. Coffer is available under the [MIT License](LICENSE).
