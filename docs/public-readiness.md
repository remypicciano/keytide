# Public repository readiness

Last reviewed: July 13, 2026

## Completed

- Full reachable Git history scanned with Gitleaks 8.30.1: 23 commits, no leaks found.
- Tracked-path audit found no local AI-assistant configuration, instruction files, keys, containers, environment files, or private-key files.
- Rust formatting, all tests, strict Clippy, and RustSec audit pass.
- Linux aarch64 and x86_64 builds run natively in GitHub Actions.
- GitHub Actions receive read-only repository contents permission and cannot approve pull requests.
- Third-party Actions are pinned to reviewed full commit SHAs.
- Dependabot monitors Cargo and GitHub Actions dependencies weekly.
- Security, support, contribution, ownership, and issue-reporting policies are present.
- Public bug forms explicitly prohibit keys and confidential data.
- Repository visibility is public; secret scanning, push protection, Dependabot alerts and security updates, and private vulnerability reporting are enabled.
- The project has an explicit MIT license.
- The tracked splash image has no ancillary metadata, and empty icon placeholders were removed.
- Windows x86_64, Linux x86_64 and ARM64, and macOS Intel and Apple Silicon artifacts build successfully.
- `main` is protected by an active ruleset requiring pull requests, all native builds, and all security checks.
- Obsolete remote branches were verified as merged and removed.
- The public project site is deployed through GitHub Pages with HTTPS enforcement and automated validation.
- Repository description, homepage, and security/privacy topics are configured.

## Repository administration follow-up

- Periodically review ruleset check names as workflows evolve so protections cannot become stale.
- Enable Apple notarization and Microsoft code signing when protected publisher identities are available.
- Revisit non-provider secret patterns and validity checks if the repository plan exposes those GitHub controls.

## Required before official production releases

- Add coverage-guided fuzzing for container, payload, and key parsers.
- Complete Kali runtime tracing, hardening, resource-pressure, and memory-residue tests.
- Sign Windows and macOS releases with publisher identities and notarize macOS builds.
- Sign Linux release checksums or provenance with a separately protected identity.
- Define release support windows and a vulnerability-response process.
- Obtain independent review of cryptographic construction, parsing, and filesystem behavior.

Changing visibility is not a release. Source may become public while downloadable builds remain explicitly marked prerelease and unsigned.
