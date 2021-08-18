use crate::gen::{Gens, Gen};

pub trait Choose where Self: Sized {
    fn choose(min: Self, max: Self) -> Gen<Self>;
}

impl Choose for i64 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_i64(min, max)
    }
}

impl Choose for u64 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_u64(min, max)
    }
}

impl Choose for i32 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_i32(min, max)
    }
}

impl Choose for u32 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_u32(min, max)
    }
}

impl Choose for i16 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_i16(min, max)
    }
}

impl Choose for u16 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_u16(min, max)
    }
}

impl Choose for i8 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_i8(min, max)
    }
}

impl Choose for u8 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_u8(min, max)
    }
}

impl Choose for char {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_char(min, max)
    }
}

impl Choose for f64 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_f64(min, max)
    }
}

impl Choose for f32 {
    fn choose(min: Self, max: Self) -> Gen<Self> {
        Gens::choose_f32(min, max)
    }
}