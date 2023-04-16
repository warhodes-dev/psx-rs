//! Allows easy conversions and type safety for the memory accessable types (u8, u16, and u32)

#[derive(Debug, Eq, PartialEq)]
pub enum AccessWidth {
    Byte,
    Short,
    Long,
}

pub trait Accessable {
    fn width() -> AccessWidth;
    fn from_u32(word: u32) -> Self;
    fn from_u16(word: u16) -> Self;
    fn from_u8(word: u8) -> Self;
    fn as_u32(&self) -> u32;
    fn as_u16(&self) -> u16;
    fn as_u8(&self) -> u8;
}

impl Accessable for u8 {
    fn width() -> AccessWidth { AccessWidth::Byte }

    fn from_u32(word: u32) -> u8 { word as u8 }
    fn from_u16(word: u16) -> u8 { word as u8 }
    fn from_u8(word: u8) -> u8 { log::error!("Casting memory accessable word from same type!"); word }

    fn as_u32(&self) -> u32 { *self as u32 }
    fn as_u16(&self) -> u16 { *self as u16 }
    fn as_u8(&self) -> u8 { log::error!("Casting memory accessable word as same type!"); *self }
}

impl Accessable for u16 {
    fn width() -> AccessWidth { AccessWidth::Short }

    fn from_u32(word: u32) -> u16 { word as u16 }
    fn from_u16(word: u16) -> u16 { log::error!("Casting memory accessable word from same type!"); word }
    fn from_u8(word: u8) -> u16 { word as u16 }

    fn as_u32(&self) -> u32 { *self as u32 }
    fn as_u16(&self) -> u16 { log::error!("Casting memory accessable word as same type!"); *self }
    fn as_u8(&self) -> u8 { *self as u8 }
}

impl Accessable for u32 {
    fn width() -> AccessWidth { AccessWidth::Long }

    fn from_u32(word: u32) -> u32 { log::error!("Casting memory accessable word from same type!"); word as u32 }
    fn from_u16(word: u16) -> u32 { word as u32 }
    fn from_u8(word: u8) -> u32 { word as u32 }

    fn as_u32(&self) -> u32 { log::error!("Casting memory accessable word to same type!"); *self }
    fn as_u16(&self) -> u16 { *self as u16 }
    fn as_u8(&self) -> u8 { *self as u8 }
}

