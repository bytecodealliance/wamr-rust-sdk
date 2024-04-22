#![allow(unused_variables)]
#![allow(dead_code)]

use wamr_bindgen::impl_bindgen;

struct Test {}

#[impl_bindgen]
impl Test {
    fn test_function(
        &self,
        a: i32,
        b: f32,
        c: String,
        d: &str,
        e: i64,
        f: &u64,
        g: i8,
        h: u8,
        i: i16,
        j: u16,
        k: f64,
    ) -> u8 {
        42
    }
}

fn main() {}
