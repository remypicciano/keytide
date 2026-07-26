# KeyTide container format v2 — review draft

Status: **cryptographic design draft; not implemented or production-ready**. This document freezes no compatibility promise yet. The version byte, algorithm-suite identifiers, KDF parameters, and test vectors must remain experimental until an independent review resolves every item in the release gate.

Version 2 adds ordinary-file key carriers and an authenticated place to record optional metadata-sanitization results. It does not modify the stable v1 parser. A v1 reader must reject v2, and a v2 implementation must continue to read v1 exactly as specified in [coffer-format-v1.md](coffer-format-v1.md).

## Design goals

- Encrypt every file with a fresh random 256-bit data-encryption key (DEK).
- Let an exact ordinary file derive a key-encryption key (KEK) without modifying or embedding data in that carrier.
- Keep carrier-only and carrier-plus-passphrase security profiles visibly distinct.
- Authenticate all visible parameters, the wrapped DEK, encrypted filename metadata, sanitization records, and file bytes.
- Make algorithm and resource limits explicit so parsers never guess.
- Preserve strict no-clobber, authenticate-before-write, and local-only behavior from v1.
- Provide deterministic fixtures and an independent implementation before enabling the UI.

## Non-goals

- Hiding the approximate source size without an explicit padding profile.
- Making a public or easily obtained carrier secret in carrier-only mode.
- Recovering a lost carrier or passphrase.
- Surviving malware that controls the active user session.
- Modifying a carrier to hide, append, or embed KeyTide data.
- Claiming that arbitrary document formats can be sanitized reliably.

## Terminology

- **DEK:** random AES-256-GCM key used only for one protected payload.
- **KEK:** 256-bit key derived from carrier material and used only to wrap the DEK.
- **Carrier:** an unchanged ordinary file whose exact bytes contribute to KEK derivation.
- **Wrap context:** visible header bytes authenticated while wrapping or unwrapping the DEK.
- **Full header:** wrap context followed by the wrapped DEK; authenticated with the payload.

## Cryptographic suites

Suite identifiers are a single byte. Unknown suites are rejected before any expensive KDF runs.

### Suite 1: carrier-only

- Carrier digest: SHA-256 over the complete carrier bytes.
- KEK derivation: HKDF-SHA-256 as defined by [RFC 5869](https://www.rfc-editor.org/rfc/rfc5869), 32-byte random salt, 32-byte output.
- HKDF input key material: the 32-byte carrier digest.
- HKDF info, exactly: ASCII `COFFER-V2-CARRIER-ONLY-KEK\0` followed by the complete wrap context.
- DEK wrapping: AES-256-GCM with a fresh 96-bit wrapping nonce.
- Payload encryption: AES-256-GCM with a distinct fresh 96-bit payload nonce.

Possession of the exact carrier is equivalent to possession of the wrapping secret. This profile is for channel separation and inconspicuous transport, not for making a published image secret.

### Suite 2: carrier plus passphrase

Suite 2 is reserved but not yet frozen. The intended construction combines a domain-separated encoding of the carrier digest and UTF-8 passphrase with Argon2id, following [RFC 9106](https://www.rfc-editor.org/rfc/rfc9106), then derives a wrapping KEK. Before assignment, review must fix:

- the unambiguous password-material encoding;
- minimum and default memory, iteration, and parallelism parameters;
- cross-platform denial-of-service limits;
- Unicode passphrase handling and whether normalization is forbidden;
- deterministic test vectors from an implementation independent of the production code.

No implementation may emit suite `2` until those fields and limits are normative. Parsers must reject it in the meantime.

## Binary container layout

All integers are unsigned and big-endian. The initial draft uses a fixed suite-1 header to keep parsing small. Offsets are included to make review concrete.

| Offset | Size | Field | Required value or limit |
| ---: | ---: | --- | --- |
| 0 | 8 | Magic | ASCII `COFFER\0\x02` |
| 8 | 1 | Format version | `2` |
| 9 | 1 | Suite | `1` for the only currently specified profile |
| 10 | 1 | Flags | `0`; unknown bits rejected |
| 11 | 1 | Reserved | `0` |
| 12 | 2 | Header length | `128` for suite 1 |
| 14 | 12 | Payload nonce | random; distinct from wrapping nonce |
| 26 | 8 | Payload ciphertext length | encrypted payload length including its 16-byte tag |
| 34 | 32 | Carrier KDF salt | random HKDF salt |
| 66 | 12 | DEK wrapping nonce | random; distinct from payload nonce |
| 78 | 2 | Wrapped DEK length | `48` for 32-byte DEK plus GCM tag |
| 80 | 48 | Wrapped DEK | AES-256-GCM output |
| 128 | declared | Payload ciphertext | AES-256-GCM output ending at EOF |

Bytes `0..80` are the **wrap context**. Bytes `0..128` are the **full header**. The container must end exactly after the declared payload ciphertext. Trailing or missing bytes are invalid.

### Construction order

1. Validate the source filename and calculate the encoded payload length with checked arithmetic.
2. Generate independent random DEK, payload nonce, KDF salt, and wrapping nonce values.
3. Encode the wrap context, including the already-computable payload ciphertext length.
4. Hash the complete carrier and derive the KEK using the selected suite.
5. Wrap the DEK with AES-256-GCM using the wrapping nonce and the complete wrap context as associated authenticated data. The result must be exactly 48 bytes.
6. Append the wrapped DEK to form the full header.
7. Encrypt the payload with the DEK, payload nonce, and complete full header as associated authenticated data.
8. Zeroize the carrier digest, KEK, DEK, encoded plaintext payload, and temporary passphrase material as soon as each is no longer needed.
9. Write through an exclusive temporary output and perform a no-clobber commit only after encryption completes.

Unwrapping reverses these steps. A wrong carrier, wrong passphrase, changed parameter, changed wrapped DEK, changed ciphertext, or changed tag must collapse to one public authentication error. Internal diagnostics may identify a stable stage code but must never log paths, filenames, digests, keys, salts, nonces, or passphrases.

## Encrypted payload layout

The decrypted payload is independently versioned so later v2-compatible records can be added without reinterpretation.

| Field | Size | Value or limit |
| --- | ---: | --- |
| Payload schema | 1 byte | `1` |
| Payload flags | 1 byte | bit 0 = sanitized-copy record present; other bits rejected |
| Filename length | 2 bytes | UTF-8 byte length; 1–1,024 |
| Original filename | variable | safe filename only, never a path |
| Restored file size | 8 bytes | exact file-byte count |
| Record count | 2 bytes | 0–32 |
| Authenticated records | variable | length-delimited TLV records |
| File bytes | declared size | exact protected content |

Each record uses a 2-byte type, 4-byte length, then the declared value. Record lengths and their total must be validated with checked arithmetic before slicing or allocation. Duplicate singleton records are invalid. Unknown critical record types (high bit set) are rejected; unknown non-critical records may be preserved by tooling but must not silently affect restore behavior.

### Record type 1: sanitization report

This singleton record is present only when the protected bytes came from a separately created sanitized copy.

| Field | Size | Meaning |
| --- | ---: | --- |
| Sanitizer profile | 2 bytes | versioned, format-specific profile identifier |
| Source media family | 2 bytes | registered format family, not a filename extension |
| Removed classes | 4 bytes | bitset of metadata classes removed and verified absent |
| Preserved classes | 4 bytes | bitset intentionally retained for correct rendering |
| Unverified classes | 4 bytes | bitset that could not be proven absent |

The report is an authenticated statement about the exact encrypted bytes, not proof about the original source. A user interface must display all three categories and must not describe a file as “clean” when any relevant class remains unverified.

## Carrier handling

- Read the complete carrier as bytes without decoding, re-encoding, metadata editing, normalization, or timestamp changes.
- Reject a carrier that is the same filesystem object as the source, destination, or temporary output.
- Permit any non-empty carrier size the implementation can process under documented resource limits.
- Report progress while hashing large carriers and remain cancellable before output commit.
- Display a comparison fingerprint derived as the first 10 bytes of `SHA-256("COFFER-V2-CARRIER-FINGERPRINT\0" || carrier_digest)`, encoded as uppercase unpadded base32. The fingerprint is never stored in the carrier.
- Describe fingerprints only as byte-identity checks; they are not unlock keys and do not make a weak or public carrier secret.

Carriers must be transferred as file/document attachments or inside a lossless archive. Inline photo, gallery, social-media, optimization, export, and “remove metadata” flows commonly rewrite bytes and invalidate the carrier.

## Parser and resource requirements

- Reject an invalid magic, version, suite, flags, reserved byte, header length, wrapped-key length, or ciphertext length before hashing a carrier.
- Set platform-tested maximum input, carrier, record, and allocation sizes.
- Never allocate directly from an unchecked container length.
- Never run a passphrase KDF with parameters outside explicit minimum and maximum bounds.
- Keep payload and wrapping nonces in separate fields and test that an encoder never reuses them.
- Authenticate and validate the complete payload before creating plaintext output.
- Apply the stable v1 filename and no-clobber rules to restored outputs.
- Remove all temporary output after cancellation, authentication failure, or I/O failure.

## Required test vectors

Before implementation can be enabled outside tests, publish byte-for-byte vectors for:

- suite-1 carrier digest, HKDF PRK/OKM, wrap context, wrapped DEK, full header, payload, and complete container;
- independent decryption using a second language or library;
- every one-byte header mutation;
- truncated and trailing data at every field boundary;
- wrong carrier, carrier changed by one byte, wrong passphrase, and wrong suite;
- wrapped-key, payload, filename, record, size, and authentication-tag corruption;
- unreasonable lengths and allocation-overflow attempts on 32-bit and 64-bit targets;
- cancellation during carrier hashing, KDF, encryption, authentication, writing, and final commit;
- Windows, Linux, and macOS round trips using identical fixtures.

## Release gate

Version 2 remains experimental until all of the following are evidenced:

- focused independent review of the suite construction, domain separation, parser, and filesystem lifecycle;
- a published threat-model review against [threat-model.md](threat-model.md);
- stable Rust and independent implementation vectors;
- coverage-guided fuzzing for header, record, payload, key-unwrapping, and filename parsers;
- bounded resource-pressure testing for large and adversarial inputs;
- zeroization review for every secret-bearing allocation;
- accessible UI copy that distinguishes carrier-only from carrier-plus-passphrase risk;
- passing Windows, Linux, macOS, advisory, secret, dependency, and provenance workflows.

Until this gate is complete, the application must not label a v2 workflow stable, must not emit suite 2, and must not imply that key carriers provide steganography or secrecy when the carrier is public.
