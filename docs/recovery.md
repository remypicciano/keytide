# Cross-platform recovery and decryption

This document is the long-term recovery path for KeyTide v1 data if the desktop application is unavailable or discontinued. Keep the `.coffer` container and its matching `.cofferkey` file separate, and retain a copy of this repository or the published format specification.

For ordinary use, prefer the native archives attached to the [v1.0.0 release](https://github.com/remypicciano/keytide/releases/tag/v1.0.0). Verify the accompanying SHA-256 checksum. The initial archives are not platform-code-signed.

The v1 format is platform-independent. A container created on macOS can be decrypted on Windows or Linux, and the reverse is also true. KeyTide uses standard AES-256-GCM; no device identifier, operating-system credential, online account, server, or KeyTide-controlled service is required.

The v2 carrier design remains a review draft and is not covered by this recovery utility. Keep v1 recovery materials and binaries available even while v2 is being developed.

## Preferred recovery: build KeyTide from source

Install the stable Rust toolchain from <https://rustup.rs/>, obtain this repository, and run:

```sh
cargo build --locked --release
cargo run --locked --release
```

Select **Restore**, choose the `.coffer` container and matching `.cofferkey`, choose a new destination, and restore the file. The application authenticates the entire container before writing plaintext and refuses to replace an existing file.

Native executable locations after building:

- macOS and Linux: `target/release/keytide`
- Windows: `target\release\keytide.exe`

Linux may require distribution packages for X11 development. On Debian, Ubuntu, or Kali:

```sh
sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libxkbcommon-dev
cargo build --locked --release
./target/release/keytide
```

## Emergency recovery without the GUI

The repository includes [`tools/recover_v1.py`](../tools/recover_v1.py), an independent parser for the documented v1 format. It requires Python 3 and the widely used `cryptography` package.

### macOS and Linux

```sh
python3 -m venv .recovery-venv
. .recovery-venv/bin/activate
python -m pip install 'cryptography>=42,<47'
python tools/recover_v1.py vault.coffer vault.cofferkey --output restored-file
```

### Windows PowerShell

```powershell
py -3 -m venv .recovery-venv
.\.recovery-venv\Scripts\Activate.ps1
python -m pip install "cryptography>=42,<47"
python tools\recover_v1.py vault.coffer vault.cofferkey --output restored-file
```

Omit `--output` to use the authenticated original filename in the current directory. The utility refuses to overwrite an existing output. A wrong key, corruption, or modification causes AES-GCM authentication to fail without producing plaintext.

## Independent implementation details

The normative byte layout and deterministic known-answer vector are in [`coffer-format-v1.md`](coffer-format-v1.md). In summary:

- The 30-byte visible container prefix is authenticated as AES-GCM associated data.
- Bytes `10..22` of that prefix are the 12-byte nonce.
- Bytes `22..30` are the big-endian ciphertext length.
- The last 16 bytes of the ciphertext are the GCM authentication tag.
- Bytes `12..44` of a validated 44-byte `.cofferkey` are the raw 256-bit AES key.
- The decrypted payload contains a two-byte filename length, UTF-8 filename, eight-byte original size, and original file bytes; all integers are big-endian.

Do not use a raw key extracted from a file until the complete key header has been validated. Never release plaintext before GCM authentication succeeds, never trust the embedded filename as a path, and never overwrite an existing destination silently.

## Recovery verification

Before relying on a copied recovery environment, run:

```sh
cargo test --locked
python -m py_compile tools/recover_v1.py
```

The Rust suite locks a deterministic compatibility vector byte-for-byte. The same vector is published in the format specification so another AES-GCM implementation can be verified independently.
