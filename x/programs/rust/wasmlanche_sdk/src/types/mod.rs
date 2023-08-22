use crate::program::Value;
use crate::store::ProgramContext;

/// A struct that enforces a fixed length of 32 bytes which represents an address.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Address {
    bytes: [u8; Self::LEN],
}

impl Address {
    pub const LEN: usize = 32;
    // Constructor function for Address
    #[must_use]
    pub fn new(bytes: [u8; Self::LEN]) -> Self {
        Self { bytes }
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::StringObject(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::StringObject(String::from(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::IntObject(value)
    }
}

impl From<Address> for Value {
    fn from(value: Address) -> Self {
        Value::AddressObject(value)
    }
}

impl From<i64> for Address {
    fn from(value: i64) -> Self {
        let bytes: [u8; Self::LEN] = unsafe {
            // We want to copy the bytes here, since [value] represents a ptr created by the host
            std::slice::from_raw_parts(value as *const u8, Self::LEN)
                .try_into()
                .unwrap()
        };
        Self { bytes }
    }
}

impl From<Value> for i64 {
    fn from(value: Value) -> Self {
        match value {
            Value::IntObject(i) => i,
            _ => panic!("Cannot conver to i64"),
        }
    }
}

impl From<Value> for ProgramContext {
    fn from(value: Value) -> Self {
        match value {
            Value::ProgramObject(i) => i,
            _ => panic!("Cannot conver to ProgramContext"),
        }
    }
}
