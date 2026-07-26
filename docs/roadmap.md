# Product roadmap

This roadmap separates interface work from cryptographic implementation. Security-sensitive milestones require format fixtures, negative tests, and review before the UI presents them as production-ready.

## Milestone 1: Desktop interface

Status: released in v1.0.0

- Complete responsive Protect and Open workflows.
- Maintain clear light and dark themes, keyboard navigation, and accessible contrast.
- Finalize native file selection, destination review, progress, cancellation, and completion states.
- Run protection and restoration on background workers connected to the verified v1 core.

## Milestone 2: KeyTide format v1

Status: released in v1.0.0

- Implement the versioned `.coffer` container and separate key format defined in [coffer-format-v1.md](coffer-format-v1.md).
- Encrypt content and authenticated filename metadata with AES-256-GCM.
- Use cryptographically secure random keys and a unique 96-bit nonce for every encryption.
- Parse with strict bounds, reject trailing data, and expose one uniform authentication failure.
- Authenticate completely before committing plaintext to disk.
- Write through owner-restricted temporary files and atomically commit completed output.
- Add cross-platform known-answer, corruption, truncation, wrong-key, and path-safety tests.
- Connect the verified implementation to the existing UI only after the core tests pass.

## Milestone 3: KeyTide v1 release

Status: released in v1.0.0

- Keep v1 deliberately small: one protected file, one separate key, local processing, and explicit destination selection.
- Replace the generic `.key` extension with `.cofferkey` while retaining a versioned binary format.
- Register the file type and application association on supported desktop platforms.
- Apply owner-only permissions where the platform supports them.
- Never store a container filename, container identifier, or key fingerprint in the key file.
- Exclude key carriers, passphrases, metadata sanitization, cloud transport, and other optional modes from v1.

## V1 maintenance and usability

Status: planned follow-up work

- Replace the time-based processing animation with real byte-level progress reported by the background worker for both protection and restoration.
- Calculate progress from bytes read, encrypted or authenticated, and safely written instead of advancing cosmetically to 90% and waiting there.
- Show an indeterminate state only when the operating system or cryptographic stage cannot provide a meaningful total.
- Keep cancellation responsive throughout large-file operations and distinguish authentication, writing, and final commit stages without implying completion early.
- Add automated tests proving that progress is monotonic, never exceeds completed work, reaches 100% only after the output is committed, and resets correctly after cancellation or failure.

## Milestone 4: KeyTide format v2 advanced privacy

Status: format and threat-model draft complete; independent review and implementation required

Version 2 introduces key carriers and optional metadata sanitization. Neither feature may change the v1 parser or be silently enabled for a v1 container.

The normative design work is tracked in [coffer-format-v2.md](coffer-format-v2.md), with security assumptions and review questions in [threat-model.md](threat-model.md). Implementation begins only after the carrier-only suite and test vectors receive focused review.

### Key carrier mode

Key carrier mode allows an unchanged ordinary file, such as a private photograph, to replace the visible `.cofferkey` file. The carrier contains no appended or embedded KeyTide payload.

- Generate a random 256-bit data-encryption key for every protected file.
- Encrypt file content with AES-256-GCM using the random data key.
- Hash the exact carrier bytes and derive a key-encryption key with a standardized, domain-separated KDF and a random public salt.
- Wrap the random data key with authenticated encryption and store the salt, wrapping nonce, and wrapped key in the versioned `.coffer` container.
- Offer two explicit carrier profiles: convenient carrier-only mode and stronger carrier-plus-passphrase mode.
- In carrier-only mode, treat possession of the exact carrier bytes as possession of the key and derive wrapping material with a standardized, domain-separated KDF.
- In carrier-plus-passphrase mode, combine the carrier digest with a passphrase using Argon2id and versioned, platform-tested parameters before deriving the key-encryption key.
- Store only public KDF parameters, salt, wrapping nonce, and the authenticated wrapped data key in the container; never store a derived key or passphrase verifier.
- Keep the carrier byte-for-byte unchanged and never modify its pixels, metadata, timestamps, or filesystem attributes.
- Clearly warn that editing, recompressing, cloud optimization, or line-ending conversion will invalidate a carrier.
- Explain in plain language that images and other carriers must be sent with **Attach file** or **Send as document**. Inline image and photo-sharing modes commonly compress or rewrite files and will not preserve a working carrier.
- Explain that carrier-only mode favors inconspicuous transport and convenience, while the optional passphrase prevents the carrier from being sufficient by itself.
- Provide local carrier fingerprint verification without writing identifying information into the carrier.
- Define this as a new container version rather than adding ambiguous behavior to v1.
- Obtain focused cryptographic review before implementation is marked production-ready.

### Implementation sequence

1. Obtain review of the fixed carrier-only header, HKDF domain separation, AES-GCM associated-data boundaries, and resource limits.
2. Publish deterministic suite-1 fixtures and verify them in an independent language.
3. Implement parser and carrier hashing behind an experimental boundary with fuzz targets and no UI exposure.
4. Add no-clobber protect/restore APIs, cancellation, progress, and cross-platform fixtures.
5. Integrate explicit carrier-only risk language into the UI and keep v1 workflows unchanged.
6. Specify and review carrier-plus-passphrase parameters before assigning suite 2.
7. Add format-specific sanitizers only when removed, preserved, and unverified classes can be reported honestly.

### Optional metadata sanitization

- Create a separate sanitized copy; never alter the source file in place.
- Begin with formats that support reliable metadata removal and verification.
- Preserve metadata required for correct rendering, including necessary color information.
- Report which metadata classes were removed and which could not be verified.
- Defer formats with reversible or incomplete sanitization behavior until format-specific handling is designed.
- Make clear that encryption conceals embedded metadata while protected, whereas sanitization changes what remains after restoration or sharing.
- Record sanitization behavior in the v2 specification and authenticated payload rather than introducing v1 flags.

## Release readiness

- Complete a threat model covering local malware, key theft, temporary files, swap, logs, crash recovery, and filesystem permissions.
- Run dependency, license, and vulnerability audits.
- Produce reproducible signed builds for supported platforms.
- Document the security boundary and unsupported threat models without recovery or backdoor claims.
- Arrange independent review of container parsing, key handling, and cryptographic construction.
