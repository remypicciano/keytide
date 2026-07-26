mod error;
mod format;
mod key;
mod ops;

pub use error::KeyTideError;
pub use ops::{
    ProtectRequest, ProtectResult, RestoreRequest, RestoreResult, protect_file, restore_file,
};
