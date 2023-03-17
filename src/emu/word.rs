

pub enum AccessWidth {
    Byte,
    Half,
    Word,
}

pub trait Word {
    fn width() -> AccessWidth;
}

impl Word for u8 {
    fn width() -> AccessWidth {
        AccessWidth::Byte
    }
}

impl Word for u16 {
    fn width() -> AccessWidth {
        AccessWidth::Byte
    }
}

impl Word for u32 {
    fn width() -> AccessWidth {
        AccessWidth::Byte
    }
}