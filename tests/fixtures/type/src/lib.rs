
#[export_name = "return_i32"]
pub fn return_i32() -> i32 {
    i32::MAX
}

#[export_name = "param_i32"]
pub fn param_i32(v: i32) -> i32 {
    v
}

#[export_name = "return_u32"]
pub fn return_u32() -> u32 {
    u32::MAX
}

#[export_name = "param_u32"]
pub fn param_u32(v: u32) -> u32 {
    v
}

#[export_name = "return_i64"]
pub fn return_i64() -> i64 {
    i64::MAX
}

#[export_name = "param_i64"]
pub fn param_i64(v: i64) -> i64 {
    v
}

#[export_name = "return_u64"]
pub fn return_u64() -> u64 {
    u64::MAX
}

#[export_name = "param_u64"]
pub fn param_u64(v: u64) -> u64 {
    v
}

