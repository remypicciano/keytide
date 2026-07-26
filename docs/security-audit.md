# Security audit notes

## Dependency review

Last reviewed: 2026-07-13

The locked dependency graph is checked with:

```sh
cargo audit
```

Two `quick-xml` advisories are temporarily accepted in `.cargo/audit.toml`:

- `RUSTSEC-2026-0194`
- `RUSTSEC-2026-0195`

The affected `quick-xml` release is a build dependency of `wayland-scanner`, reached through the Linux windowing and clipboard stack. It parses protocol XML bundled by the Wayland crates during compilation. KeyTide does not expose this parser to selected files, filenames, containers, keys, metadata, network responses, or other runtime input.

The upstream `wayland-scanner` release currently constrains `quick-xml` to the affected `0.39` line, so Cargo cannot select the fixed `0.41` release. The exceptions must be removed as soon as the windowing dependency accepts the fixed parser. A dependency update must rerun `cargo audit`, all tests, and strict Clippy.

These exceptions do not apply to a runtime parser and must not be broadened to cover unrelated advisories.

## Recurring public-repository checks

Every pull request and push to `main`, plus a weekly schedule, runs:

- Gitleaks against the complete reachable Git history;
- RustSec against the locked Cargo dependency graph;
- cargo-deny policy for dependency licenses, registries, Git sources, and duplicate-version visibility;
- GitHub dependency review for newly introduced vulnerable packages.

GitHub secret scanning and push protection are enabled, as are Dependabot vulnerability alerts and automated security updates. Local release review must still scan the full history and inspect tracked binary metadata.

## Logging boundary

KeyTide logs application lifecycle and generic operation state. It records operation names and stable error codes only. It must not log:

- source, destination, container, or key paths;
- original or restored filenames;
- plaintext or ciphertext content;
- encryption keys, nonces, or key-file bytes.

## External review

Passing automated checks is not a substitute for independent cryptographic and parser review. The v1 format and implementation should receive focused review before KeyTide is represented as externally audited software.

The repository-wide [threat model](threat-model.md) records stable v1 residual risks and the questions that must be resolved before the [v2 draft](coffer-format-v2.md) becomes an implementation target.
