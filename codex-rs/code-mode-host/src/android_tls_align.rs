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
