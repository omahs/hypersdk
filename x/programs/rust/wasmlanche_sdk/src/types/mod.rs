use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::store::ProgramContext;
/// A struct that enforces a fixed length of 32 bytes which represents an address.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Address(Bytes32);

impl Address {
    pub const LEN: usize = 32;
    // Constructor function for Address
    pub fn new(bytes: [u8; Self::LEN]) -> Self {
        Self(Bytes32::new(bytes))
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0.as_bytes()
    }
}

impl From<i64> for Address {
    fn from(value: i64) -> Self {
        Self(Bytes32::from(value))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub struct Bytes32([u8; Self::LEN]);
impl Bytes32 {
    pub const LEN: usize = 32;
    pub fn new(bytes: [u8; Self::LEN]) -> Self {
        Self(bytes)
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    pub fn to_string(&self) -> String {
        // Find the first null byte, or use the full length.
        let null_pos = self.0.iter().position(|&b| b == b'\0').unwrap_or(Self::LEN);
        String::from_utf8_lossy(&self.0[..null_pos]).to_string()
    }
}

impl From<i64> for Bytes32 {
    fn from(value: i64) -> Self {
        let bytes: [u8; Self::LEN] = unsafe {
            // We want to copy the bytes here, since [value] represents a ptr created by the host
            std::slice::from_raw_parts(value as *const u8, Self::LEN)
                .try_into()
                .unwrap()
        };
        Self(bytes)
    }
}

pub trait Argument {
    fn as_bytes(&self) -> Cow<'_, [u8]>;
    fn is_primitive(&self) -> bool;
    fn len(&self) -> usize {
        self.as_bytes().len()
    }
}

impl Argument for Bytes32 {
    fn as_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0)
    }
    fn is_primitive(&self) -> bool {
        false
    }
}

impl Argument for Address {
    fn as_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.0.as_bytes())
    }
    fn is_primitive(&self) -> bool {
        false
    }
}

impl Argument for i64 {
    fn as_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(self.to_be_bytes().to_vec())
    }
    fn is_primitive(&self) -> bool {
        true
    }
}

impl Argument for ProgramContext {
    fn as_bytes(&self) -> Cow<'_, [u8]> {
        self.program_id.as_bytes()
    }
    fn is_primitive(&self) -> bool {
        true
    }
}

impl From<i64> for Box<dyn Argument> {
    fn from(value: i64) -> Self {
        Box::new(value)
    }
}

impl From<Bytes32> for Box<dyn Argument> {
    fn from(value: Bytes32) -> Self {
        Box::new(value)
    }
}

impl From<Address> for Box<dyn Argument> {
    fn from(value: Address) -> Self {
        Box::new(value)
    }
}

impl From<ProgramContext> for Box<dyn Argument> {
    fn from(value: ProgramContext) -> Self {
        Box::new(value)
    }
}

impl From<String> for Bytes32 {
    fn from(value: String) -> Self {
        let mut bytes: [u8; Self::LEN] = [0; Self::LEN];
        bytes[..value.len()].copy_from_slice(value.as_bytes());
        Self(bytes)
    }
}
