# KeyTide feature catalog

Last updated: July 13, 2026

KeyTide is a local-first desktop application for protecting individual files with authenticated encryption and a separate unlock key. Version 1 is intentionally small and auditable. Advanced carrier files, passphrases, sanitization, and cloud-related concepts are reserved for a future version of the container format.

## Version 1: implemented features

### Local file protection

- Protects any local file type as binary data.
- Creates a separate encrypted `.coffer` container without changing the original file.
- Creates a new `.cofferkey` unlock key for every protection operation.
- Uses a distinct random key for each container, limiting the effect of one lost or exposed key.
- Allows the user to review and choose the output directory and container filename.
- Refuses unsafe output names and requires the `.coffer` extension.
- Never silently replaces an existing container, key, or restored file.

### Authenticated encryption

- Encrypts with AES-256-GCM.
- Generates 256-bit keys directly from the operating system's secure random source.
- Generates a new random 96-bit nonce for every encryption.
- Authenticates the complete visible container prefix as associated data.
- Authenticates encrypted content, the original filename, and the exact original file size.
- Detects an incorrect key, modified container, corruption, and authentication-tag failure.
- Uses one safe public authentication error instead of revealing which authentication check failed.
- Zeroizes raw keys, key-file buffers, plaintext working buffers, AES-GCM state, and the expanded AES key schedule when they are dropped.

### Versioned `.coffer` containers

- Uses an explicit magic value, format version, and algorithm identifier.
- Stores only the version, algorithm, nonce, ciphertext length, and authenticated ciphertext visibly.
- Keeps the original filename and exact file size inside the encrypted payload.
- Rejects truncated input, trailing bytes, inconsistent lengths, unsupported versions, and unsupported algorithms.
- Validates every declared length before slicing or using it.
- Includes a deterministic compatibility vector for cross-platform and non-Rust implementations.
- Reveals an approximation of the original size because v1 does not add padding.

### Versioned `.cofferkey` files

- Uses a strict, versioned 44-byte binary key format.
- Includes a key magic value, version, algorithm identifier, reserved bytes, and 32 bytes of key material.
- Rejects incorrect lengths, altered markers, unknown versions, unknown algorithms, and nonzero reserved bytes.
- Does not identify which container the key opens.
- Requests owner-only `0600` permissions on Unix-like operating systems.
- Accepts only `.cofferkey` files in the v1 key-selection interface.
- Has no recovery copy, escrow key, master key, or backdoor.

### Safe restoration

- Requires both a `.coffer` container and its matching `.cofferkey`.
- Authenticates the complete container before creating any plaintext destination file.
- Validates the authenticated original filename even though it came from an encrypted payload.
- Rejects path separators, directory components, NUL characters, `.` and `..` filenames.
- Restores only to the destination and filename reviewed by the user.
- Requires the restored byte count to match the authenticated original size exactly.
- Optionally offers an in-memory preview for supported UTF-8 text files up to 1 MiB.
- Clears the in-memory preview when the secure viewer closes.

### Filesystem safety

- Writes through randomly named temporary files in the selected destination directory.
- Uses exclusive temporary-file creation to avoid collisions.
- Flushes complete temporary output before committing it.
- Uses a no-clobber commit strategy so a race cannot replace an existing destination.
- Rolls back the paired key if the container cannot be committed.
- Removes incomplete temporary output after errors and cancellation.
- Waits for the background worker to finish cleanup during cancellation and normal shutdown.
- Tests output-existence races and incomplete-output cleanup.

### Responsive desktop workflow

- Provides separate Protect, Open, Security, and Settings areas.
- Uses guided, non-clickable progress rails for protection and restoration steps.
- Supports drag-and-drop and native file-selection dialogs.
- Shows explicit review screens before protection or restoration starts.
- Runs file operations on background workers so the interface remains responsive.
- Displays estimated activity progress, cancellation, success, and contextual error states. Real byte-level progress for large files is tracked as v1 follow-up work.
- Preserves file selections when navigating between application areas.
- Shows planned and completed output paths.
- Can reveal completed output destinations in Finder, Explorer, or the platform file manager.
- Can copy completed output paths.
- Supports opening another file after completion.

### Appearance and presentation

- Includes light and dark themes with a horizontal theme switch.
- Uses contrast-tested text and primary-button colors in both themes.
- Includes a full-window animated splash screen with click-to-skip behavior.
- Scales between wide desktop and compact window layouts.
- Uses hidden visual scrollbars while preserving wheel and trackpad scrolling.
- Distinguishes interactive controls from informational surfaces and workflow status.
- Includes authorship and copyright attribution for Rémy Picciano.

### Settings and security information

- Provides an optional text-preview preference.
- Clearly identifies non-configurable safeguards such as no-clobber output and no location history.
- Explains local processing, authentication, key separation, malware limitations, and the lack of a recovery backdoor.
- Warns that malware already controlling the user's session can access plaintext or keys while they are in use.
- Explains that cryptographic algorithms, nonce behavior, and key sizes are not user-configurable preferences.

### Logging and diagnostics

- Logs application startup and normal or abnormal shutdown.
- Logs generic Protect and Restore operation lifecycle events.
- Logs stable error codes without sensitive context.
- Does not log source paths, destination paths, filenames, keys, nonces, plaintext, ciphertext contents, or key-file bytes.
- Supports runtime filtering through the standard tracing environment configuration.

### Verification and repository quality

- Includes tests for format parsing, key parsing, binary round trips, wrong keys, tampering, truncation, trailing data, unsupported fields, unsafe filenames, file-size mismatches, permissions, cancellation, filesystem races, unique keys, and UI-to-backend integration.
- Locks a deterministic AES-GCM compatibility fixture byte for byte.
- Has independently verified that fixture with Node's AES-256-GCM implementation.
- Passes strict Clippy with warnings treated as errors.
- Passes formatting and Git diff-integrity checks.
- Uses a locked Rust dependency graph and RustSec advisory scanning.
- Documents narrow build-time advisory exceptions separately; they do not cover runtime file parsing.
- Keeps generated builds, release artifacts, local assistant configuration, encrypted containers, keys, environment files, logs, and temporary files out of Git.
- Runs recurring full-history secret scans, RustSec advisory checks, dependency license/source policy, and pull-request dependency review.
- Publishes native archives for Windows x86_64, Linux x86_64 and ARM64, and macOS Intel and Apple Silicon.

## Version 1: deliberate limitations

- Protects one file per container; folders and archives are not constructed automatically.
- Loads a complete file into memory because chunked encryption is outside the v1 format.
- Does not pad containers, so encrypted size approximates plaintext size.
- Does not reuse unlock keys between newly protected files.
- Does not include passwords, passphrases, key carriers, secret sharing, or recovery keys.
- Does not sanitize metadata before encryption.
- Does not provide cloud storage, synchronization, or remote recovery.
- Does not protect plaintext from malware, screen capture, keyloggers, memory inspection, or another process already controlling the user's account.
- Has automated verification but has not yet been represented as independently audited cryptographic software.

## Version 2: specified for review, not implemented

These features are now defined in the [v2 format review draft](coffer-format-v2.md) and assessed in the [threat model](threat-model.md). They remain outside the production application until independent review, test vectors, fuzzing, resource limits, and cross-platform tests satisfy the documented release gate.

### Ordinary-file key carriers

- Use an unchanged ordinary image, document, or other file as a key carrier.
- Keep the carrier valid in its normal application without embedding or appending a KeyTide payload.
- Derive wrapping material from the carrier's exact bytes using a standardized, domain-separated KDF.
- Generate a random AES data-encryption key and store only its authenticated wrapped form in the v2 container.
- Offer a convenient carrier-only profile where possession of the exact carrier is equivalent to possession of the key.
- Offer a stronger carrier-plus-passphrase profile using Argon2id and versioned parameters.
- Support a short local fingerprint for confirming that backup or transferred carriers are byte-identical.
- Consider multiple authorized carriers and separately reviewed recovery designs in later revisions.
- Use a fixed carrier-only header with SHA-256, HKDF-SHA-256 domain separation, independent wrapping and payload nonces, and AES-256-GCM DEK wrapping.
- Reserve carrier-plus-passphrase mode until its Argon2id encoding, parameters, denial-of-service limits, Unicode behavior, and independent vectors are frozen.

### Critical carrier-transfer warning

An image or other carrier must arrive **byte for byte unchanged**.

- Send it with **Attach file** or **Send as document**.
- A lossless ZIP attachment is also appropriate.
- Do not paste an image inline into an email.
- Do not use a messaging application's ordinary photo or gallery button.
- Do not upload and redownload it through social media.
- Do not crop, rotate, resize, optimize, resave, or remove metadata from it.

Those actions may rewrite the file even when it looks identical. A rewritten copy will not work as the key. In carrier-only mode, the exact carrier is sensitive because anyone with both it and the `.coffer` container can attempt to unlock the container. Email and messaging providers may retain attachments in mailboxes, backups, synchronized devices, and notification systems.

### Optional metadata sanitization

- Create a separate sanitized copy rather than modifying the source in place.
- Begin with formats where removal can be performed and verified reliably.
- Preserve metadata required for correct rendering, such as necessary color information.
- Report what was removed and what could not be verified.
- Defer formats such as PDF or RAW when removal may be incomplete, reversible, or damaging.
- Keep sanitization behavior in the authenticated v2 specification rather than adding ambiguous v1 flags.
- Encode removed, preserved, and unverified metadata classes in an authenticated, versioned sanitization record.

### Future release engineering

- Independent cryptographic and parser review.
- Reproducible, signed installers for supported operating systems.
- Expanded cross-platform continuous integration and test fixtures.
- Documented vulnerability-reporting and release-support policies.
- Continued dependency, license, and security-advisory review.

## Safety summary

- Keep each `.cofferkey` separate from its matching `.coffer` file.
- Back up keys carefully; KeyTide cannot recreate a lost key.
- Do not treat a renamed key as cryptographically hidden.
- For future carriers, send exact files as attachments, never inline.
- Use carrier-plus-passphrase mode when an openly shared carrier must not be sufficient by itself.
- Keep the operating system updated and use reputable endpoint protection; encryption cannot defend against malware already controlling the active session.
