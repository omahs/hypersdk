use crate::errors::StorageError;
use crate::host::{get_bytes, get_bytes_len, invoke_program_method, store_bytes};
use crate::program::Value;
use std::borrow::Cow;
use std::str;

pub struct Tag(pub u8);

impl Tag {
    #[must_use]
    pub fn as_u8(&self) -> u8 {
        self.0
    }
}

impl From<u8> for Tag {
    fn from(tag: u8) -> Self {
        Tag(tag)
    }
}

/// Store represents any type that can be stored in the host.
pub trait Store: Sized {
    fn as_bytes(&self) -> Cow<'_, [u8]>;
    /// Returns the tag of the type.
    fn as_tag(&self) -> Tag;
    /// # Errors
    /// Fails when the bytes cannot be converted to the type to be stored.
    fn from_bytes(bytes: &[u8]) -> Result<Self, StorageError>;
}

/// `ProgramContext` defines helper methods for the program builder
/// to interact with the host.
#[derive(Clone)]
pub struct ProgramContext {
    pub program_id: u64,
}

impl ProgramContext {
    /// # Errors
    /// Returns a `StorageError` if the value cannot be serialized or stored.
    pub fn store_value<T: Store>(&self, key: &str, value: &T) -> Result<(), StorageError> {
        let key_bytes = key.as_bytes();
        // Add the tag(u8) to the start of val_bytes
        store_key_value(self, key_bytes, value)
    }

    /// # Errors
    /// Returns a `StorageError` if the value cannot be serialized or stored.
    pub fn store_map_value<T: Store>(
        &self,
        map_name: &str,
        key: &Value,
        value: &T,
    ) -> Result<(), StorageError> {
        let key_bytes = get_map_key(map_name, key);
        // Add a tag(u8) to the start of val_bytes
        store_key_value(self, &key_bytes, value)
    }

    /// # Errors
    /// Returns a `StorageError` if there is no value at the given `name`.
    pub fn get_value(&self, name: &str) -> Result<Value, StorageError> {
        get_field(self, name)
    }

    /// # Errors
    /// Returns a `StorageError` if there is no value at the given `map_name` and `key`.
    pub fn get_map_value(&self, map_name: &str, key: &Value) -> Result<Value, StorageError> {
        get_map_field(self, map_name, key)
    }
}

impl From<ProgramContext> for i64 {
    fn from(ctx: ProgramContext) -> Self {
        ctx.program_id.try_into().unwrap()
    }
}

impl From<i64> for ProgramContext {
    fn from(value: i64) -> Self {
        ProgramContext {
            program_id: value.try_into().unwrap(),
        }
    }
}

fn store_key_value<T: Store>(
    ctx: &ProgramContext,
    key_bytes: &[u8],
    value: &T,
) -> Result<(), StorageError> {
    let val_bytes = std::iter::once(value.as_tag().as_u8())
        .chain(value.as_bytes().iter().copied())
        .collect::<Vec<u8>>();
    match unsafe {
        store_bytes(
            ctx,
            key_bytes.as_ptr(),
            key_bytes.len(),
            val_bytes.as_ptr(),
            val_bytes.len(),
        )
    } {
        0 => Ok(()),
        _ => Err(StorageError::HostStoreError),
    }
}

fn get_field_as_bytes(ctx: &ProgramContext, name: &[u8]) -> Result<Vec<u8>, StorageError> {
    let name_ptr = name.as_ptr();
    let name_len = name.len();
    // First get the length of the bytes from the host.
    let bytes_len = unsafe { get_bytes_len(ctx, name_ptr, name_len) };

    // Speculation that compiler might be optimizing out this if statement.
    if bytes_len < 0 {
        return Err(StorageError::InvalidByteLength(
            bytes_len.try_into().unwrap(),
        ));
    }

    // Get_bytes allocates bytes_len memory in the WASM module.
    let bytes_ptr = unsafe { get_bytes(ctx, name_ptr, name_len, bytes_len) };

    // Defensive check here to unsure we don't grab out of bounds memory.
    let Ok(bytes_len) = bytes_len.try_into() else {
        return Err(StorageError::HostRetrieveError);
    };

    let bytes_ptr = bytes_ptr as *mut u8;

    // Take ownership of those bytes grabbed from the host. We want Rust to manage the memory.
    let bytes = unsafe { Vec::from_raw_parts(bytes_ptr, bytes_len, bytes_len) };

    Ok(bytes)
}

/// Converts a byte vector to a string
/// # Errors
/// Returns a `FromUtf8Error` if the bytes are not valid UTF-8.
pub fn to_string(bytes: Vec<u8>) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(bytes)
}

/// Gets the field [name] from the host and returns it as a `ProgramValue`.
/// # Errors
/// Returns a `StorageError` if the field cannot be found.
fn get_field<T: Store>(ctx: &ProgramContext, name: &str) -> Result<T, StorageError> {
    let bytes = get_field_as_bytes(ctx, name.as_bytes())?;
    T::from_bytes(&bytes)
}

/// Gets the correct key to in the host storage for a [`map_name`] and [key] within that map  
fn get_map_key(map_name: &str, key: &Value) -> Vec<u8> {
    [map_name.as_bytes(), &key.as_bytes()].concat()
}

// Gets the value from the map [name] with key [key] from the host and returns it as a ProgramValue.
fn get_map_field<T: Store>(
    ctx: &ProgramContext,
    name: &str,
    key: &Value,
) -> Result<T, StorageError> {
    let map_key = get_map_key(name, key);
    let map_value = get_field_as_bytes(ctx, &map_key)?;
    T::from_bytes(&map_value)
}

/// Implement the `program_invoke` function for the `ProgramContext` which allows a program to
/// call another program.
impl ProgramContext {
    #[must_use]
    pub fn invoke_program_method(
        &self,
        call_ctx: &ProgramContext,
        fn_name: &str,
        call_args: &[Value],
    ) -> Value {
        // hardcode first arg for now
        let result = invoke_program_method(self, call_ctx, fn_name, &Self::marshal_args(call_args));
        // Hardcode int for now
        Value::IntObject(result)
    }

    fn marshal_args(args: &[Value]) -> Vec<u8> {
        use std::mem::size_of;
        // Size of meta data for each argument
        let meta_size = size_of::<i64>() + 1;

        // Calculate the total size of the combined byte slices
        let total_size: usize = args
            .iter()
            .map(|cow| cow.as_bytes().len() + meta_size)
            .sum();

        // Create a mutable Vec<u8> to hold the combined bytes
        let mut bytes = Vec::with_capacity(total_size);

        for arg in args {
            // if we want to be efficient we dont need to add length of bytes if its an int
            let len = Value::IntObject(
                i64::try_from(arg.as_bytes().len()).expect("Not handling errors yet."),
            );
            bytes.extend_from_slice(&len.as_bytes());

            match arg {
                Value::IntObject(_) => {
                    bytes.extend_from_slice(&[1]);
                }
                _ => {
                    bytes.extend_from_slice(&[0]);
                }
            }
            bytes.extend_from_slice(&arg.as_bytes());
        }
        bytes
    }
}
