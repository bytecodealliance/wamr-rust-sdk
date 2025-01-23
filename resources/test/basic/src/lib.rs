#![no_std]

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "host")]
extern "C" {
    fn now() -> i64;
    // fn calculate_native(n: f32, func1: usize, func2: usize) -> f32;
}

static mut START_TIME: i64 = 0;
static mut TIMER_ACTIVE: bool = false;

#[export_name = "start"]
pub fn start() {
    unsafe {
        START_TIME = now();
        TIMER_ACTIVE = true;
    }
}

#[export_name = "stop"]
pub fn stop() -> i64 {
    unsafe {
        if TIMER_ACTIVE {
            TIMER_ACTIVE = false;
            return now() - START_TIME;
        }
        0
    }
}

// fn mul7(n: f32) -> f32 {
//     n * 7.0
// }

// fn mul5(n: f32) -> f32 {
//     n * 5.0
// }

// #[export_name = "calculate"]
// pub fn calculate(n: f32) -> f32 {
//     unsafe {
//         calculate_native(n, mul5 as usize, mul7 as usize)
//     }
// }