use std::{
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use zeroize::Zeroizing;

use super::{
    error::KeyTideError,
    format,
    key::{self, SecretKey},
};

pub struct ProtectRequest<'a> {
    pub source: &'a Path,
    pub container_output: &'a Path,
    pub key_output: &'a Path,
    pub cancelled: Option<&'a AtomicBool>,
}

pub struct ProtectResult {
    pub container: PathBuf,
    pub key: PathBuf,
}

pub struct RestoreRequest<'a> {
    pub container: &'a Path,
    pub key: &'a Path,
    pub output: &'a Path,
    pub cancelled: Option<&'a AtomicBool>,
}

pub struct RestoreResult {
    pub output: PathBuf,
    pub original_filename: String,
}

pub fn protect_file(request: ProtectRequest<'_>) -> Result<ProtectResult, KeyTideError> {
    tracing::info!(operation = "protect", "operation started");
    let result = protect_file_inner(request);
    match &result {
        Ok(_) => tracing::info!(operation = "protect", "operation completed"),
        Err(error) => tracing::warn!(
            operation = "protect",
            code = error.code(),
            "operation failed"
        ),
    }
    result
}

fn protect_file_inner(request: ProtectRequest<'_>) -> Result<ProtectResult, KeyTideError> {
    if request.key_output == request.container_output {
        return Err(KeyTideError::OutputExists(
            request.container_output.to_path_buf(),
        ));
    }
    ensure_absent(request.container_output)?;
    ensure_absent(request.key_output)?;
    let source_name = request
        .source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or(KeyTideError::InvalidFilename)?;
    format::validate_filename(source_name)?;
    let plaintext = Zeroizing::new(fs::read(request.source).map_err(KeyTideError::ReadFailed)?);
    check_cancelled(request.cancelled)?;

    let key = SecretKey::generate()?;
    let container = format::encrypt(source_name, &plaintext, &key)?;
    check_cancelled(request.cancelled)?;
    let mut container_temp = TemporaryOutput::create(request.container_output, false)?;
    container_temp.write_all(&container)?;

    let mut key_temp = TemporaryOutput::create(request.key_output, true)?;
    let encoded = key::encode(&key);
    key_temp.write_all(&encoded)?;

    check_cancelled(request.cancelled)?;

    let committed_key = key_temp.commit()?;
    if let Err(error) = check_cancelled(request.cancelled) {
        let _ = fs::remove_file(&committed_key);
        return Err(error);
    }
    let committed_container = match container_temp.commit() {
        Ok(path) => path,
        Err(error) => {
            let _ = fs::remove_file(&committed_key);
            return Err(error);
        }
    };
    Ok(ProtectResult {
        container: committed_container,
        key: committed_key,
    })
}

pub fn restore_file(request: RestoreRequest<'_>) -> Result<RestoreResult, KeyTideError> {
    tracing::info!(operation = "restore", "operation started");
    let result = restore_file_inner(request);
    match &result {
        Ok(_) => tracing::info!(operation = "restore", "operation completed"),
        Err(error) => tracing::warn!(
            operation = "restore",
            code = error.code(),
            "operation failed"
        ),
    }
    result
}

fn restore_file_inner(request: RestoreRequest<'_>) -> Result<RestoreResult, KeyTideError> {
    ensure_absent(request.output)?;
    let container = fs::read(request.container).map_err(KeyTideError::ReadFailed)?;
    let key_bytes = Zeroizing::new(fs::read(request.key).map_err(KeyTideError::ReadFailed)?);
    let key = key::parse(&key_bytes)?;
    let payload = format::decrypt(&container, &key)?;
    check_cancelled(request.cancelled)?;

    // No destination file exists until authentication and complete payload validation succeed.
    let mut output = TemporaryOutput::create(request.output, false)?;
    output.write_all(&payload.bytes)?;
    check_cancelled(request.cancelled)?;
    let committed = output.commit()?;
    Ok(RestoreResult {
        output: committed,
        original_filename: payload.filename,
    })
}

fn check_cancelled(cancelled: Option<&AtomicBool>) -> Result<(), KeyTideError> {
    if cancelled.is_some_and(|value| value.load(Ordering::Relaxed)) {
        Err(KeyTideError::Cancelled)
    } else {
        Ok(())
    }
}

fn ensure_absent(path: &Path) -> Result<(), KeyTideError> {
    if path.exists() {
        Err(KeyTideError::OutputExists(path.to_path_buf()))
    } else {
        Ok(())
    }
}

struct TemporaryOutput {
    final_path: PathBuf,
    temporary_path: PathBuf,
    file: Option<File>,
}

impl TemporaryOutput {
    fn create(final_path: &Path, owner_only: bool) -> Result<Self, KeyTideError> {
        #[cfg(not(unix))]
        let _ = owner_only;

        ensure_absent(final_path)?;
        let parent = final_path.parent().ok_or_else(|| {
            KeyTideError::WriteFailed(io::Error::new(
                io::ErrorKind::InvalidInput,
                "missing parent",
            ))
        })?;
        for _ in 0..16 {
            let mut random = [0_u8; 8];
            getrandom::fill(&mut random).map_err(|_| KeyTideError::RandomFailed)?;
            let suffix = u64::from_ne_bytes(random);
            let temporary_path = parent.join(format!(".coffer-{suffix:016x}.tmp"));
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            if owner_only {
                use std::os::unix::fs::OpenOptionsExt as _;
                options.mode(0o600);
            }
            match options.open(&temporary_path) {
                Ok(file) => {
                    return Ok(Self {
                        final_path: final_path.to_path_buf(),
                        temporary_path,
                        file: Some(file),
                    });
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(KeyTideError::WriteFailed(error)),
            }
        }
        Err(KeyTideError::WriteFailed(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "could not reserve temporary output",
        )))
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), KeyTideError> {
        let file = self.file.as_mut().ok_or_else(|| {
            KeyTideError::WriteFailed(io::Error::other("temporary output is closed"))
        })?;
        file.write_all(bytes).map_err(KeyTideError::WriteFailed)?;
        file.sync_all().map_err(KeyTideError::WriteFailed)
    }

    fn commit(mut self) -> Result<PathBuf, KeyTideError> {
        self.file.take();
        match fs::hard_link(&self.temporary_path, &self.final_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                return Err(KeyTideError::OutputExists(self.final_path.clone()));
            }
            Err(error) => return Err(KeyTideError::WriteFailed(error)),
        }
        if let Err(error) = fs::remove_file(&self.temporary_path) {
            let _ = fs::remove_file(&self.final_path);
            return Err(KeyTideError::WriteFailed(error));
        }
        Ok(self.final_path.clone())
    }
}

impl Drop for TemporaryOutput {
    fn drop(&mut self) {
        self.file.take();
        let _ = fs::remove_file(&self.temporary_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_operations_round_trip_and_never_replace_outputs() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.bin");
        let container = directory.path().join("source.bin.coffer");
        let key = directory.path().join("source.bin.cofferkey");
        let restored = directory.path().join("restored.bin");
        fs::write(&source, [0, 1, 2, 0xff]).unwrap();

        protect_file(ProtectRequest {
            source: &source,
            container_output: &container,
            key_output: &key,
            cancelled: None,
        })
        .unwrap();
        assert!(container.exists());
        assert!(key.exists());

        let result = restore_file(RestoreRequest {
            container: &container,
            key: &key,
            output: &restored,
            cancelled: None,
        })
        .unwrap();
        assert_eq!(result.original_filename, "source.bin");
        assert_eq!(fs::read(&restored).unwrap(), [0, 1, 2, 0xff]);
        assert!(matches!(
            restore_file(RestoreRequest {
                container: &container,
                key: &key,
                output: &restored,
                cancelled: None,
            }),
            Err(KeyTideError::OutputExists(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn generated_key_is_owner_only_on_unix() {
        use std::os::unix::fs::PermissionsExt as _;
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.txt");
        let container = directory.path().join("source.coffer");
        let key = directory.path().join("source.cofferkey");
        fs::write(&source, b"secret").unwrap();
        protect_file(ProtectRequest {
            source: &source,
            container_output: &container,
            key_output: &key,
            cancelled: None,
        })
        .unwrap();
        assert_eq!(
            fs::metadata(key).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn authentication_failure_creates_no_plaintext_or_temp_file() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.txt");
        let container = directory.path().join("source.coffer");
        let key = directory.path().join("source.cofferkey");
        let wrong_key = directory.path().join("wrong.cofferkey");
        let wrong_container = directory.path().join("wrong.coffer");
        let output = directory.path().join("restored.txt");
        fs::write(&source, b"secret").unwrap();
        protect_file(ProtectRequest {
            source: &source,
            container_output: &container,
            key_output: &key,
            cancelled: None,
        })
        .unwrap();
        protect_file(ProtectRequest {
            source: &source,
            container_output: &wrong_container,
            key_output: &wrong_key,
            cancelled: None,
        })
        .unwrap();
        assert!(matches!(
            restore_file(RestoreRequest {
                container: &container,
                key: &wrong_key,
                output: &output,
                cancelled: None,
            }),
            Err(KeyTideError::AuthenticationFailed)
        ));
        assert!(!output.exists());
        assert_eq!(
            fs::read_dir(directory.path())
                .unwrap()
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
                .count(),
            0
        );
    }

    #[test]
    fn pre_cancelled_operation_creates_no_output() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.txt");
        let container = directory.path().join("source.coffer");
        let key = directory.path().join("source.cofferkey");
        fs::write(&source, b"secret").unwrap();
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            protect_file(ProtectRequest {
                source: &source,
                container_output: &container,
                key_output: &key,
                cancelled: Some(&cancelled),
            }),
            Err(KeyTideError::Cancelled)
        ));
        assert!(!container.exists());
        assert!(!key.exists());
    }

    #[test]
    fn temporary_output_drop_and_commit_race_leave_no_partial_data() {
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("output.bin");
        let temporary_path = {
            let mut temporary = TemporaryOutput::create(&output, false).unwrap();
            temporary.write_all(b"incomplete").unwrap();
            temporary.temporary_path.clone()
        };
        assert!(!temporary_path.exists());
        assert!(!output.exists());

        let mut temporary = TemporaryOutput::create(&output, false).unwrap();
        temporary.write_all(b"new data").unwrap();
        let reserved_temp = temporary.temporary_path.clone();
        fs::write(&output, b"existing data").unwrap();
        assert!(matches!(
            temporary.commit(),
            Err(KeyTideError::OutputExists(_))
        ));
        assert_eq!(fs::read(&output).unwrap(), b"existing data");
        assert!(!reserved_temp.exists());
    }

    #[test]
    fn every_protection_uses_a_distinct_random_key() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("source.txt");
        let first_container = directory.path().join("first.coffer");
        let first_key = directory.path().join("first.cofferkey");
        let second_container = directory.path().join("second.coffer");
        let second_key = directory.path().join("second.cofferkey");
        let invalid_output = directory.path().join("must-not-exist.txt");
        fs::write(&source, b"same source").unwrap();

        for (container, key) in [
            (&first_container, &first_key),
            (&second_container, &second_key),
        ] {
            protect_file(ProtectRequest {
                source: &source,
                container_output: container,
                key_output: key,
                cancelled: None,
            })
            .unwrap();
        }

        assert_ne!(
            fs::read(&first_key).unwrap(),
            fs::read(&second_key).unwrap()
        );
        assert!(matches!(
            restore_file(RestoreRequest {
                container: &second_container,
                key: &first_key,
                output: &invalid_output,
                cancelled: None,
            }),
            Err(KeyTideError::AuthenticationFailed)
        ));
        assert!(!invalid_output.exists());
    }
}
