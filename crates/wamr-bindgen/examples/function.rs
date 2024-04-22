use wamr_bindgen::function_bindgen;

#[function_bindgen]
fn test_function(_a: i32, _b: f32, _c: String, _d: &str, _e: i64, _f: u64, _g: i8) -> u8 {
    42
}

fn main() {}
