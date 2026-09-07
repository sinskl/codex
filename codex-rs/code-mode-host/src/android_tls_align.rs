//! Android Bionic requires PT_TLS p_align >= 64 bytes on ARM64.
//!
//! Bionic's Thread Control Block uses slots at TPIDR+0..TPIDR+63; the
//! executable TLS segment starts at round_up(sizeof(tcb_head), p_align).
//! With p_align=8 the TLS segment starts at TPIDR+16, stomping on
//! Bionic's TCB slots. With p_align=64 the TLS data starts at TPIDR+64,
//! leaving all TCB slots intact.
//!
//! The V8 static library contains thread-local variables with 8-byte
//! alignment, which lld propagates as PT_TLS p_align=8. The assembly
//! below creates a `.tbss` section with explicit 64-byte alignment so
//! lld computes max(all TLS section alignments) = 64 and emits p_align=64.
//!
//! `force_tls_alignment()` references the TLS object through mrs tpidr_el0
//! so the section is never garbage-collected by the linker.

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

.section .text
.global _android_tls_align_touch
.hidden _android_tls_align_touch
.type _android_tls_align_touch, @function
_android_tls_align_touch:
    // Reference the TLS symbol so the linker keeps its section alive.
    mrs x0, tpidr_el0
    add x0, x0, :tprel_hi12:_android_tls_align_force
    add x0, x0, :tprel_lo12_nc:_android_tls_align_force
    // Load one byte to complete the reference.
    ldrb w0, [x0]
    ret
"#
);

#[cfg(all(target_os = "android", target_arch = "aarch64"))]
extern "C" {
    fn _android_tls_align_touch();
}

/// Touches the 64-byte-aligned TLS anchor so the linker retains it,
/// forcing PT_TLS p_align = 64.
///
/// # Errors
///
/// Never fails.
#[cfg(all(target_os = "android", target_arch = "aarch64"))]
pub fn force_tls_alignment() {
    // SAFETY: trivial leaf function reading one TLS byte.
    unsafe { _android_tls_align_touch() };
}

#[cfg(any(not(target_os = "android"), not(target_arch = "aarch64")))]
pub fn force_tls_alignment() {}
