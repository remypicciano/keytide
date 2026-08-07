# Changelog

<span style="color:#56802f">Every meaningful change to KeyTide, recorded so you never have to guess what shipped.</span>

The project follows semantic versioning once tagged releases begin.

## Unreleased

### Added

- Review draft of the KeyTide v2 carrier-only container, authenticated sanitization records, parser limits, and release gate.
- Repository threat model covering v1 guarantees, residual risks, v2 review questions, and the verification program.
- Public GitHub Pages site for the security boundary, format, downloads, and roadmap.

## [1.0.0] - 2026-07-13

### Added

- Production AES-256-GCM implementation of the version-one `.coffer` container.
- Strict parser for authenticated filenames, declared sizes, versions, algorithms, and trailing data.
- Versioned 44-byte `.cofferkey` encoding with owner-only Unix permissions.
- Same-directory temporary writes, no-clobber commits, cancellation cleanup, and rollback for paired outputs.
- Background Protect and Open workers connected to the desktop workflow.
- Zeroization for plaintext working buffers, key-file bytes, and in-memory key material.
- Generic lifecycle and operation logging that excludes paths, filenames, keys, and file contents.
- Deterministic compatibility fixture and negative tests for malformed, truncated, changed, and mismatched inputs.
- Plain-language attachment warnings for the planned v2 key-carrier feature.
- Documented dependency-advisory review and narrowly scoped build-time exceptions.
- Native release artifacts for Windows x86_64, Linux x86_64 and ARM64, and macOS Intel and Apple Silicon.
- Scheduled full-history secret scanning, dependency policy checks, and pull-request dependency review.
- MIT license and public vulnerability-reporting policy.

### Changed

- Generated key files now use the `.cofferkey` extension.
- Updated egui/eframe and the native file dialog to their current compatible releases.
- Replaced settings that had no behavioral effect with non-interactive safeguard descriptions.
- Random keys, nonces, and temporary identifiers now come directly from the operating system.
- Sanitized tracked image metadata and removed empty placeholder icon files.

### Removed

- Simulated timer-based protection and restoration results.
- Prototype messages claiming that no files were read or written.

[1.0.0]: https://github.com/remypicciano/keytide/releases/tag/v1.0.0
