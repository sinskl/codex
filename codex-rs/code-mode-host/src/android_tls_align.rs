//! Android Bionic requires PT_TLS p_align >= 64 bytes on ARM64.
//!
//! Bionic's Thread Control Block uses slots at TPIDR+0..TPIDR+63; the
//! executable TLS segment starts at round_up(sizeof(tcb_head), p_align).
//! With p_align=8 the TLS segment starts at TPIDR+16, stomping on
//! Bionic's TCB slots. With p_align=64 the TLS data starts at TPIDR+64,
//! leaving all TCB slots intact.
//!
//! The V8 static library contains thread-local variables with 8-byte
//! alignment, which lld propagates as PT_TLS p_align=8. This module
//! creates a `.tbss` section with explicit 64-byte alignment so lld
//! computes max(all TLS section alignments) = 64 and emits p_align=64.
//!
//! The symbol is referenced from `force_tls_alignment()` so the linker
//! does not garbage-collect the section.

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
core::arch::global_asm!(
    r#"
.section .tbss, "awT", @nobits
.balign 64
.global _android_tls_align_force
.hidden _android_tls_align_force
.type _android_tls_align_force, @tls_object
.size _android_tls_align_force, 64
_android_tls_align_force:
.zero 64
"#
);

/// Forces the 64-byte-aligned TLS object above to be retained by the
/// linker. The value read is discarded; only the section reference matters.
///
/// # Errors
///
/// Never returns an error; the TLS object is always readable.
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
pub fn force_tls_alignment() {
    extern "C" {
        #[thread_local]
        static _android_tls_align_force: [u8; 64];
    }
    // Volatile-ish read to keep the reference alive.
    let _ = unsafe { core::ptr::read_volatile(&_android_tls_align_force[0]) };
}

#[cfg(any(not(target_os = "android"), not(target_arch = "aarch64")))]
pub fn force_tls_alignment() {}
