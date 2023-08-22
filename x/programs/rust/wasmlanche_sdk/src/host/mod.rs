use crate::store::ProgramContext;

// The map module contains functionality for storing and retrieving key-value pairs.
#[link(wasm_import_module = "map")]
extern "C" {
    #[link_name = "init_program"]
    fn _init_program() -> i64;

    #[link_name = "store_bytes"]
    fn _store_bytes(
        contractId: u64,
        key_ptr: *const u8,
        key_len: usize,
        value_ptr: *const u8,
        value_len: usize,
    ) -> i32;

    #[link_name = "get_bytes_len"]
    fn _get_bytes_len(contract_id: u64, key_ptr: *const u8, key_len: usize) -> i32;

    #[link_name = "get_bytes"]
    fn _get_bytes(contract_id: u64, key_ptr: *const u8, key_len: usize, val_len: i32) -> i32;
}

// The program module contains functionality for invoking external programs.
#[link(wasm_import_module = "program")]
extern "C" {
    #[link_name = "invoke_program"]
    fn _invoke_program(
        contract_id: u64,
        call_contract_id: u64,
        method_name_ptr: *const u8,
        method_name_len: usize,
        args_ptr: *const u8,
        args_len: usize,
    ) -> i64;
}

/* wrappers for unsafe imported functions ----- */
/// Returns the `map_id` or None if there was an error
#[must_use]
pub fn init_program_storage() -> ProgramContext {
    unsafe { ProgramContext::from(_init_program()) }
}

/// Stores the bytes at `value_ptr` to the bytes at key ptr on the host.
///
/// # Safety
/// The caller must ensure that `key_ptr` + `key_len` and
/// `value_ptr` + `value_len` point to valid memory locations.
#[must_use]
pub unsafe fn store_bytes(
    ctx: &ProgramContext,
    key_ptr: *const u8,
    key_len: usize,
    value_ptr: *const u8,
    value_len: usize,
) -> i32 {
    unsafe { _store_bytes(ctx.program_id, key_ptr, key_len, value_ptr, value_len) }
}

/// Gets the length of the bytes associated with the key from the host.
///
/// # Safety
/// The caller must ensure that `key_ptr` + `key_len` points to valid memory locations.
#[must_use]
pub unsafe fn get_bytes_len(ctx: &ProgramContext, key_ptr: *const u8, key_len: usize) -> i32 {
    unsafe { _get_bytes_len(ctx.program_id, key_ptr, key_len) }
}

/// Gets the bytes associated with the key from the host.
///
/// # Safety
/// The caller must ensure that `key_ptr` + `key_len` points to valid memory locations.
#[must_use]
pub unsafe fn get_bytes(
    ctx: &ProgramContext,
    key_ptr: *const u8,
    key_len: usize,
    val_len: i32,
) -> i32 {
    unsafe { _get_bytes(ctx.program_id, key_ptr, key_len, val_len) }
}

/// Invokes another program and returns the result.
#[must_use]
pub fn invoke_program_method(
    ctx: &ProgramContext,
    call_ctx: &ProgramContext,
    method_name: &str,
    args: &[u8],
) -> i64 {
    let method_name_bytes = method_name.as_bytes();
    unsafe {
        _invoke_program(
            ctx.program_id,
            call_ctx.program_id,
            method_name_bytes.as_ptr(),
            method_name_bytes.len(),
            args.as_ptr(),
            args.len(),
        )
    }
}
