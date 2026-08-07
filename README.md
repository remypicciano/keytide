# KeyTide

```text
#######    ###############################     ######################## ##################################  ################
##   ##   ###  #####             #####   ##   ###  #####             ## ##             ## ##            ### ##             #
##   ##  ###  ### ##   ############ ###   ## ##   ### #######   ####### #######   ####### ##   #######   #####   ###########
##   ## ##   ##   ##   ##             ###  ###  ###        ##   ##           ##   ##      ##   ##   ##   ## ##   ##
##   #####  ###   ##   ########        ##  ###  ##         ##   ##           ##   ##      ##   ##   ##   ## ##   ########
##   ##     ##    ##   ##########       ###   ###          ##   ##           ##   ##      ##   ##   ##   ## ##   #########
##           ##   ##     ########        ### ###           ##   ##           ##   ##      ##   ##   ##   ## ##    ########
##   ####     ### ##   ##########        ### ###           ##   ##           ##   ##      ##   ## #### #### ##   #########
##   #####     #####   ##                ##   ##           ##   ##           ##   ##      ##   #####  ###   ##   ##
##   ## ###     ####   ###########      ###   ###         ###   ###      ######   #####   ##   ####  ###   ###   ##########
##   ##   ##   #####           ##       ###   ###         ###   ###       ##        ###   ##        ##     ##           ###
#######   ####### ###############        #######           #######        ############    ############      ##############
```

> **Protect a file locally. Keep the key apart.**

KeyTide turns one local file into an authenticated container and a separate unlock key — then it gets out of the way. Everything happens on your machine. There is no account, no cloud sync, no master key, and no recovery backdoor, because the whole point is that *you* hold both halves.

[![Version](https://img.shields.io/badge/version-1.0.0-56802f)](https://github.com/remypicciano/keytide/releases)
[![License](https://img.shields.io/badge/license-MIT-235a20)](LICENSE)
[![Platforms](https://img.shields.io/badge/platforms-macOS%20%C2%B7%20Windows%20%C2%B7%20Linux-2e352f)](#)
[![Encryption](https://img.shields.io/badge/encryption-AES--256--GCM-ff5a36)](#)
[![Local-first](https://img.shields.io/badge/local--first-100%25%20offline-8eaa5c)](#)

---

## What this is

Most "encryption tools" want your file and your trust in their servers. KeyTide does the opposite: it reads the file on-device, writes a new encrypted container, and writes a matching key file beside it — two artifacts meant to live **in different places**.

```text
source file
  → protected container   (.coffer)
  → separate unlock key   (.cofferkey)
```

The original file stays untouched. No phone-home, no telemetry, no account. When you want the file back, both halves come together *in your hands*, never in a cloud.

Version **1.0.0** is the first stable release. Grab the archives from [GitHub Releases](https://github.com/remypicciano/keytide/releases) and verify the included SHA-256 checksum before use. The public project site at [https://remypicciano.github.io/keytide/](https://remypicciano.github.io/keytide/) covers the security model, platform builds, and format roadmap.

### Highlights

- Protects **one local file at a time** — small, deliberate, auditable.
- Leaves the original file **completely unchanged**.
- Mints a **fresh key for every protection run** — one leaked key never costs you the rest.
- **Authenticates before restoring** — tampered or wrong-key input never becomes plaintext.
- Refuses **overwrite races** and unsafe output names.
- Stays **100% local** — no network, no account, no backdoor.

---

## Quick start

### 1. Run the app

```sh
cargo run
```

### 2. Protect a file

1. Choose a source file.
2. Choose a destination for the protected container.
3. Save the generated key in a *different* place.

### 3. Restore later

1. Choose the `.coffer` container.
2. Choose the matching key file.
3. Pick a new destination for the restored file.

Run `cargo run -- --help` or poke around the UI to see every flow and safety check.

---

## Desktop workflow

1. Open the app and pick **Protect** or **Restore**.
2. Review the selected file, output name, and destination.
3. Confirm only after verifying the container and key are being written separately.
4. Move the key away from the container after saving.

---

## How it works

1. The app reads the selected file locally.
2. It generates a fresh random AES-256-GCM key for the protected output.
3. The file bytes and filename metadata are encrypted into a versioned container.
4. The matching key is written as a separate file.
5. Restoration verifies authentication **before** writing plaintext.

Under the hood it is small on purpose. The v1 byte layout is frozen, published, and verified by an independent Python recovery tool — read [`docs/coffer-format-v1.md`](docs/coffer-format-v1.md) and [`docs/recovery.md`](docs/recovery.md) if you want the gory details.

---

## Development

Install Rust with edition 2024 support plus the platform dependencies required by `eframe` and `rfd`. Run these checks before opening a change:

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo audit
python3 tools/check_docs.py
python3 tools/check_site.py
python3 -m py_compile tools/*.py
```

---

## Project structure

```text
src/              Rust application and UI code
docs/             Format, security, and development notes
site/             GitHub Pages site
tools/            Recovery and validation helpers
```

---

## About the author

Created by [Rémy Picciano](https://github.com/remypicciano). KeyTide comes from a simple belief: your files should be protected without you having to trust someone else's server with them. If a tool can't explain what it does in plain language, it's not done yet — and if it can't stand to be read line by line, it shouldn't hold your secrets.

---

## Contact

- Email: [remypicciano@icloud.com](mailto:remypicciano@icloud.com)
- GitHub: [github.com/remypicciano](https://github.com/remypicciano)
- Project discussions: [open a GitHub issue](https://github.com/remypicciano/keytide/issues)
- Security issues: use [private vulnerability reporting](https://github.com/remypicciano/keytide/security/advisories/new)

---

## Security and license

Never attach real keys, private containers, or confidential plaintext to an issue. KeyTide is available under the [MIT License](LICENSE).
