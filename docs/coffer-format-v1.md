# KeyTide container and key format v1

Status: stable as of KeyTide 1.0.0. The production core, filesystem operations, and UI integration follow this format. The deterministic compatibility fixture is enforced by the automated test suite. Version 1 compatibility is frozen; incompatible changes require a new format version.

All integers are unsigned and stored in big-endian byte order. Parsers must reject truncated input, inconsistent lengths, unsupported versions, and unsupported algorithms.

## Design principles

- Store only what is necessary to decrypt and restore a file safely.
- Keep sensitive metadata inside the encrypted payload.
- Use a new random 256-bit key and a new random 96-bit nonce for every encryption.
- Authenticate the visible prefix and encrypted payload with AES-256-GCM.
- Never write plaintext until authentication succeeds.
- Introduce new behavior through a new format version instead of speculative v1 flags.

The separate key file must be stored and transmitted separately from the `.coffer` container. KeyTide has no recovery key or backdoor.

## `.coffer` container

Only the minimum information required to recognize and decrypt the container is visible.

| Field | Size | Value or limit |
| --- | ---: | --- |
| Magic | 8 bytes | ASCII `COFFER\0\x01` |
| Format version | 1 byte | `1` |
| Algorithm | 1 byte | `1` = AES-256-GCM |
| Nonce | 12 bytes | Cryptographically random; never reused with the same key |
| Ciphertext length | 8 bytes | Length of encrypted payload including the GCM tag |
| Ciphertext and tag | declared length | Encrypted payload produced by AES-256-GCM |

The fixed 30-byte visible prefix is passed to AES-GCM as associated authenticated data. It is visible but cannot be modified without authentication failing. The container must end exactly after the declared ciphertext; trailing or missing bytes are invalid.

The container's total size still reveals an approximation of the source size. Hiding size accurately would require padding, which is outside v1.

## Encrypted payload

The payload below is serialized first and then encrypted as one authenticated unit.

| Field | Size | Value or limit |
| --- | ---: | --- |
| Filename length | 2 bytes | UTF-8 byte length; 1–1,024 bytes |
| Original filename | variable | UTF-8 filename only, never a path |
| Original file size | 8 bytes | Exact plaintext byte length |
| Original file bytes | declared size | Exact binary contents of the source file |

The original filename and exact size are therefore not readable without the correct key. No creation timestamp, generic metadata, key fingerprint, or optional flags are stored in v1.

On restore, the filename remains untrusted even though it was authenticated. KeyTide must reject `.` and `..`, NUL, directory components, and platform path separators, then write only inside the destination selected by the user. Existing files must not be overwritten silently.

Authentication failures use one public error regardless of whether the likely cause is a wrong key, modified prefix, modified ciphertext, or corruption.

## `.cofferkey` file

| Field | Size | Value |
| --- | ---: | --- |
| Magic | 8 bytes | ASCII `COFKEY\0\x01` |
| Format version | 1 byte | `1` |
| Algorithm | 1 byte | `1` = AES-256-GCM |
| Reserved | 2 bytes | Must be `0` in v1 |
| Key material | 32 bytes | Cryptographically random AES key |

The `.cofferkey` extension identifies the file in desktop interfaces; the extension is not a security boundary. The encoded key file is exactly 44 bytes. Other lengths, magic values, versions, algorithms, or nonzero reserved bytes are invalid. Writers request owner-only permissions on Unix-like systems (`0600`) and never replace an existing key file.

The key format marker identifies a valid KeyTide key; it does not reveal which container it opens. AES-GCM authentication determines whether the selected key matches.

## Parser requirements

- Validate every length before slicing or allocating.
- Limit filenames to 1,024 UTF-8 bytes.
- Reject unsupported versions and algorithms before decryption.
- Require the decrypted file byte count to equal the embedded original size.
- Never derive a filesystem path directly from an embedded name.
- Never write plaintext before authentication succeeds.
- Write through a temporary file in the destination directory and commit only after the complete write succeeds.
- Remove temporary output after failure or cancellation.
- Do not log keys, plaintext, ciphertext contents, or sensitive filenames.

## Compatibility

Version 1 readers must reject unknown versions rather than guessing. Future metadata, chunked large-file encryption, secret sharing, or algorithm changes require a deliberately specified new format version. Cross-platform test fixtures must guarantee byte-for-byte compatibility on macOS, Windows, and Linux.

The separate [v2 review draft](coffer-format-v2.md) follows this rule and does not reinterpret any v1 byte or reserved field.

### Deterministic compatibility fixture

This fixture is for compatibility testing only. Production keys and nonces must always come from the operating system's secure random source.

| Input | Value |
| --- | --- |
| Key | 32 bytes of `0x11` |
| Nonce | 12 bytes of `0x22` |
| Filename | UTF-8 `note.txt` |
| File bytes | UTF-8 `hello` |

Expected complete container, encoded as hexadecimal:

```text
434f4646455200010101222222222222222222222222000000000000002717ff6926b4aab12b9d4bde3c48a6e9d51f96a58706edc3edae22ac447457f57b1a54dfb50676d9
```

Expected decrypted payload, encoded as hexadecimal:

```text
00086e6f74652e747874000000000000000568656c6c6f
```

The automated Rust test locks the complete container bytes. The vector has also been independently decrypted with Node's AES-256-GCM implementation using bytes `0..30` as associated authenticated data, bytes `10..22` as the nonce, and the final 16 ciphertext bytes as the authentication tag.
