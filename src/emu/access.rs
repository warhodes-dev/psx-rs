//! Allows easy conversions and type safety for the memory Access types (u8, u16, and u32)

#[derive(Debug, Eq, PartialEq)]
pub enum AccessWidth {
    Byte,
    Half,
    Word,
}

pub trait Access {
    fn width() -> AccessWidth;
    fn from_u32(word: u32) -> Self;
    fn from_u16(word: u16) -> Self;
    fn from_u8(word: u8) -> Self;
    fn as_u32(&self) -> u32;
    fn as_u16(&self) -> u16;
    fn as_u8(&self) -> u8;
}

impl Access for u8 {
    fn width() -> AccessWidth { AccessWidth::Byte }

    fn from_u32(word: u32) -> u8 { word as u8 }
    fn from_u16(word: u16) -> u8 { word as u8 }
    fn from_u8(word: u8) -> u8 { word }

    fn as_u32(&self) -> u32 { *self as u32 }
    fn as_u16(&self) -> u16 { *self as u16 }
    fn as_u8(&self) -> u8 { *self }
}

impl Access for u16 {
    fn width() -> AccessWidth { AccessWidth::Half }

    fn from_u32(word: u32) -> u16 { word as u16 }
    fn from_u16(word: u16) -> u16 { word }
    fn from_u8(word: u8) -> u16 { word as u16 }

    fn as_u32(&self) -> u32 { *self as u32 }
    fn as_u16(&self) -> u16 { *self }
    fn as_u8(&self) -> u8 { *self as u8 }
}

impl Access for u32 {
    fn width() -> AccessWidth { AccessWidth::Word }

    fn from_u32(word: u32) -> u32 { word }
    fn from_u16(word: u16) -> u32 { word as u32 }
    fn from_u8(word: u8) -> u32 { word as u32 }

    fn as_u32(&self) -> u32 { *self }
    fn as_u16(&self) -> u16 { *self as u16 }
    fn as_u8(&self) -> u8 { *self as u8 }
}

