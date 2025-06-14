(module
    (func (export "multi") (result f64 f32 i32 i64 f64 f32 i32 i64 v128 v128 v128 v128)
        f64.const 22.2222
        f32.const 1.57
        i32.const 42
        i64.const 3523
        f64.const 22.2222
        f32.const 1.57
        i32.const 42
        i64.const 3523
        v128.const i32x4 1 2 3 4
        v128.const f32x4 1 2 3 4
        v128.const i64x2 1 2
        v128.const f64x2 1 2
    )
)