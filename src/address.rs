use std::ffi::c_void;

pub type Addr = *const c_void;

/// 基地址以及偏移1,和偏移2
#[derive(Debug)]
pub struct Address(pub(crate) Addr, pub(crate) u32, pub(crate) u32);

pub const SUN: Address = Address(0x006A9F38 as _, 0x768, 0x5560);
pub const SLIVER_COIN: Address = Address(0x006A9EC0 as _, 0x82C, 0x208);
pub const CD: Address = Address(0x006A9EC0 as _, 0x768, 0x144);
