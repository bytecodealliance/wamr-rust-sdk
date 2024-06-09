use std::string::String;
use std::alloc::{alloc, dealloc, Layout};

#[no_mangle]
#[export_name = "my_malloc"]
pub unsafe fn my_alloc(size: usize) -> *mut u8 {
    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, align);
    alloc(layout)
}

#[no_mangle]
#[export_name = "my_free"]
pub unsafe fn my_dealloc(ptr: *mut u8, size: usize) {
    let align = std::mem::align_of::<usize>();
    let layout = Layout::from_size_align_unchecked(size, align);
    dealloc(ptr, layout);
}

#[no_mangle]
#[export_name = "my_strcat"]
pub fn my_strcat(s1: &str, s2: &str) -> String {
    println!("-=-|> s1: {}, s2: {}", s1, s2);

    let mut result = String::with_capacity(s1.len() + s2.len());
    let ret_ref = &result;
    println!("  --> the address of result: {:p}", ret_ref as *const String);

    result.push_str(s1);
    result.push_str(s2);
    println!("  --> result: {result}");

    result
}

fn main() {
    let result = my_strcat("hello", "world");
    println!("-> {}", result);
    let result = my_strcat(&result, "from, xxx,");
    println!("-> {}", result);
    let result = my_strcat(&result, "main function");
    println!("-> {}", result);
}
