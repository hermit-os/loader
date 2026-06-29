//! PC-relative addressing on i386.

/// Performs PC-relative addressing on i386.
///
/// This macro puts the address of `sym` into `reg` using the GOT.
///
/// On x86-64, this would just be the following line:
///
/// ```asm
/// mov \reg, qword ptr [rip + \sym@GOTPCREL]
/// ```
///
/// On i386, though, there is no direct way to access EIP.
/// Thus, we need to push the current EIP to the stack via `call`.
/// Then, we to adjust it's value and resolve the symbol via the GOT.
.macro movgot reg, sym
    call 2f
2:
    pop \reg
3:
// LLVM currently rejects the equivalent Intel syntax:
// https://github.com/llvm/llvm-project/issues/161550
.att_syntax prefix
    addl $_GLOBAL_OFFSET_TABLE_ + (3b - 2b), %\reg
.intel_syntax noprefix
    mov \reg, dword ptr [\reg + \sym@GOT]
.endm
