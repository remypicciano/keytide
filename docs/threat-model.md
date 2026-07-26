# KeyTide threat model

Last reviewed: July 13, 2026

This document defines the security boundary for stable format v1 and the review target for format v2. It is not a claim of independent audit.

## Assets

- plaintext file contents and authenticated original filename;
- v1 `.cofferkey` material;
- v2 carrier bytes, passphrases, carrier digests, KEKs, and random DEKs;
- restored plaintext and optional in-memory previews;
- integrity and availability of protected containers and backups.

## Trust boundary

KeyTide trusts the local operating system, cryptographic random source, process isolation, Rust toolchain, reviewed dependencies, and release pipeline while they are operating correctly. File and carrier contents, filenames, containers, keys, output paths, environment variables, and removable media are untrusted inputs.

The network is not inside the runtime security boundary. Core protection and restoration do not require an account or network connection. GitHub is part of source distribution and release provenance, not file processing.

## Threat actors and scenarios

| Scenario | In scope | Expected control |
| --- | :---: | --- |
| Attacker obtains only a v1 container | Yes | AES-256-GCM confidentiality; approximate size remains visible |
| Attacker modifies a container or key | Yes | strict parsing or uniform authentication failure; no plaintext output |
| Attacker races an output path | Yes | exclusive temporary creation and no-clobber commit |
| Attacker supplies malformed lengths or filenames | Yes | checked parsing, bounded allocation, safe filename validation |
| Attacker obtains both v1 container and key | No | possession of both authorizes decryption |
| Attacker obtains v2 container and exact carrier-only carrier | No | carrier-only possession authorizes key unwrapping by design |
| Attacker obtains v2 carrier but not its passphrase | Planned | carrier-plus-passphrase KDF must resist offline guessing within stated limits |
| Messaging service recompresses a carrier | Availability | fingerprint mismatch and clear transfer guidance; rewritten carrier cannot unlock |
| Malware controls the active session | No | may read inputs, outputs, keystrokes, process memory, or screen contents |
| Compromised OS, compiler, dependency, or CI runner | Supply chain | recurring scans and provenance reduce risk but do not make compromise impossible |
| Device loss, deletion, ransomware, or missing backups | Availability | user-managed separate backups; no escrow or recovery backdoor |
| Traffic analysis or observation of file sizes | Partly | processing is local; container size still approximates source size |

## Security invariants

1. A new random encryption key and nonce are generated for every protected payload.
2. Authentication succeeds before any plaintext destination is created.
3. Existing destination files are never silently replaced.
4. Parsers reject unknown versions, algorithms, flags, malformed lengths, truncation, and trailing data.
5. Embedded filenames are treated as untrusted and never become paths.
6. Keys, passphrases, plaintext, sensitive paths, carrier digests, and identifying fingerprints are not logged.
7. Temporary files are removed after cancellation and failure.
8. Secret-bearing buffers are zeroized where the language and dependency APIs permit.
9. v1 behavior remains frozen; advanced modes require an explicit new format version.
10. UI language never implies a recovery backdoor, steganography, independent audit, or protection from a compromised session.

## v1 residual risks

- Complete files are held in memory, so very large files can exhaust resources and plaintext may be observable to privileged memory inspection.
- Container size approximates plaintext size because v1 has no padding.
- Unix key files request `0600`; effective Windows and macOS access still depends on the user profile and filesystem ACLs.
- Initial release archives have GitHub provenance attestations and checksums but are not Apple-notarized or Microsoft-code-signed.
- The emergency Python recovery tool cannot promise deterministic memory zeroization because of Python’s immutable byte objects and runtime.

## v2-specific review questions

- Does carrier hashing use the exact complete byte sequence on every supported platform?
- Are HKDF domains and AES-GCM associated-data boundaries unambiguous and independently reproducible?
- Are wrapping and payload nonces independently generated and impossible to confuse?
- Can hostile header parameters cause expensive work, unbounded allocation, or KDF denial of service before validation?
- Does carrier-plus-passphrase encoding avoid concatenation ambiguity and Unicode normalization surprises?
- Does the UI prevent users from mistaking a public carrier for a second secret?
- Do fingerprints reveal only a comparison value and avoid being stored in the carrier or logs?
- Can a sanitization profile prove the stated metadata classes absent without damaging required rendering data?
- Are cancellation and crash paths free of committed partial containers, plaintext, keys, and reports?

## Verification program

- Unit and integration tests for every invariant and negative parser case.
- Deterministic cross-language compatibility fixtures.
- Coverage-guided fuzzing of container, key, payload, record, and filename parsers.
- Dependency advisories, license/source policy, secret scanning, and pinned Actions on every pull request and weekly.
- Native Windows, Linux, and macOS tests and release builds.
- Release checksums and GitHub build-provenance attestations.
- Focused independent cryptographic, parser, filesystem, and UX review before v2 is stable.

## Vulnerability response

Suspected vulnerabilities must be reported through [private vulnerability reporting](../SECURITY.md). Reports must use synthetic data and must not contain real keys, carriers, passphrases, private containers, or confidential plaintext.
