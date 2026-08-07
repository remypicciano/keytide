# Contributing to KeyTide

Thanks for wanting to improve KeyTide. The project stays honest only because people read it carefully — cryptography, parsing, key handling, filesystem commits, and container compatibility get extra scrutiny for a reason. If you're here to help make that stronger, welcome aboard.

## Before opening a change

- Use a focused branch and explain the user or security problem being solved.
- Discuss container-format changes before implementation. Version 1 readers must remain strict and compatible.
- Treat [the v2 format](docs/coffer-format-v2.md) as a review draft: cryptographic-suite changes require deterministic vectors, threat-model analysis, and focused review before production code.
- Report vulnerabilities privately according to [SECURITY.md](SECURITY.md).
- Never commit real keys, containers, plaintext samples, credentials, environment files, logs, local assistant configuration, generated builds, or private test fixtures.

## Required checks

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo audit
cargo deny check bans licenses sources
gitleaks git --redact .
```

Use synthetic fixtures. New parsers and security boundaries need negative tests for truncation, corruption, unsupported versions, unreasonable lengths, unsafe paths, cancellation, and no-clobber behavior as applicable.

## Pull requests

Keep commits intentional and describe:

- what changed and why;
- security and compatibility impact;
- validation performed;
- documentation or migration requirements.

Pull requests must not weaken authentication errors, overwrite protection, key separation, zeroization, or the offline security boundary without an explicit design review.

All third-party GitHub Actions must be pinned to a reviewed full commit SHA. Do not commit generated executables; release artifacts are produced by GitHub Actions from reviewed commits.
