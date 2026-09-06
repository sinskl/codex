//! Cross-platform advisory file locks.
//!
//! On most platforms this is a thin wrapper over [`std::fs::File`] locks.
//! Rust's standard library does not support `File::lock` on Android, so this
//! crate falls back to `flock(2)`, which the Android kernel does support.
//!
//! [`try_lock`] reports contention as [`io::ErrorKind::WouldBlock`] instead of
//! std's `TryLockError` so that match sites stay simple:
//!
//! ```ignore
//! match codex_file_lock::try_lock(&file) {
//!     Ok(()) => { /* acquired */ }
//!     Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => { /* held elsewhere */ }
//!     Err(err) => return Err(err),
//! }
//! ```

use std::fs::File;
use std::io;
use std::io::Error;
use std::io::ErrorKind;

/// Acquires an exclusive advisory lock, blocking until the lock is available.
pub fn lock(file: &File) -> io::Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        file.lock()
    }
    #[cfg(target_os = "android")]
    {
        flock(file, libc::LOCK_EX)
    }
}

/// Tries to acquire an exclusive advisory lock without blocking.
///
/// Returns [`Err`] with kind [`ErrorKind::WouldBlock`] when the lock is held
/// elsewhere.
pub fn try_lock(file: &File) -> io::Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        file.try_lock().map_err(try_lock_error_into_io)
    }
    #[cfg(target_os = "android")]
    {
        flock(file, libc::LOCK_EX | libc::LOCK_NB)
    }
}

/// Acquires a shared advisory lock, blocking until the lock is available.
pub fn lock_shared(file: &File) -> io::Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        file.lock_shared()
    }
    #[cfg(target_os = "android")]
    {
        flock(file, libc::LOCK_SH)
    }
}

/// Tries to acquire a shared advisory lock without blocking.
///
/// Returns [`Err`] with kind [`ErrorKind::WouldBlock`] when the lock is held
/// elsewhere.
pub fn try_lock_shared(file: &File) -> io::Result<()> {
    #[cfg(not(target_os = "android"))]
    {
        file.try_lock_shared().map_err(try_lock_error_into_io)
    }
    #[cfg(target_os = "android")]
    {
        flock(file, libc::LOCK_SH | libc::LOCK_NB)
    }
}

#[cfg(not(target_os = "android"))]
fn try_lock_error_into_io(err: std::fs::TryLockError) -> io::Error {
    match err {
        std::fs::TryLockError::WouldBlock => Error::new(ErrorKind::WouldBlock, "lock is held elsewhere"),
        std::fs::TryLockError::Error(error) => error,
    }
}

#[cfg(target_os = "android")]
fn flock(file: &File, operation: i32) -> io::Result<()> {
    // SAFETY: `flock` only inspects the file descriptor, which is borrowed
    // from `file` and guaranteed to be open for the duration of the call.
    let result = unsafe { libc::flock(file.as_fd().as_raw_fd(), operation) };
    if result == 0 {
        return Ok(());
    }
    let err = io::Error::last_os_error();
    if matches!(
        err.raw_os_error(),
        Some(libc::EWOULDBLOCK) | Some(libc::EINTR)
    ) && operation & libc::LOCK_NB != 0
    {
        // `EINTR` is grouped with contention here because all callers treat a
        // failed non-blocking acquisition as "try again later".
        return Err(Error::new(ErrorKind::WouldBlock, "lock is held elsewhere"));
    }
    Err(err)
}

#[cfg(target_os = "android")]
use std::os::fd::AsFd;
#[cfg(target_os = "android")]
use std::os::fd::AsRawFd;
