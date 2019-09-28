//!

#[doc(hidden)]
#[macro_export(crate)]
macro_rules! new_wire {
    ($bits:expr) => {
        Wire::<{ $bits }, { $crate::wires::num_bytes($bits) }>::new()
    };
}

// For some reason this fails to compile:
// Update: it's because calls to `Wire::new_with_val` fail to compile (just like
// `Wire::get_bytes`). The below works so we'll use it.
// macro_rules! new_wire_with_val {
//     ($bits:expr, $val:expr) => {Wire::<{ $bits }, { num_bytes($bits) }>::new_with_val($val)};
// }

#[doc(hidden)]
#[macro_export(crate)]
macro_rules! new_wire_with_val {
    ($bits:expr, $val:expr) => {
        {
            let mut w = $crate::new_wire!($bits);
            w.set($val);

            w
        }
    };
}

/// The unchecked version of the [`W!`](TODO!) proc macro. If you're making
/// wires out of immediate values you should definitely use `W!` instead.
///
/// If you need to create wires without set values or wires whose values/number
/// of bits come from const expressions instead of literals, this is the macro
/// for you.
///
/// There are three styles:
///  - `[num bits] bits for { value } as type` and
///  - `[num bits]#{value}:type`
///  - `type:value => num bits`
///
/// The type of the expression can be omitted. If not explicitly specified, the
/// type will default to `usize`.
///
/// When `num bits` is an expression it must be surrounded by `[]`; otherwise
/// the square brackets can be omitted. The same goes for values: for literals
/// the braces (`{}`) aren't required. Additionally when the type is omitted,
/// braces on value expressions can be omitted.
///
/// There's also a shorthand for make new wires that aren't set to a value:
///  - `[expression for num bits] bits` or
///  - `<expression for num bits>`
///  - `<literal for num bits> bits`
///
///
/// Here are some examples:
///
/// Fully qualified `for` form:
/// ```rust
/// w!([4] bits for {2 + 3} as u128);
/// w!([8 * 8] bits for {core::u64::MAX} as u64);
/// ```
///
/// Fully qualified `#` form:
/// ```rust
/// w!([1]#{1}:u8);
/// w!([1] # {1} : u8);
/// ```
///
/// Fully qualified `=>` form:
/// ```rust
/// w!(u8:67 * 2 => )
/// ```
///
/// Types can be omitted:
/// ```rust
/// w!([13] bits for {13 * 2});
///
/// // When you leave out types you can ditch the braces on value expressions:
/// w!([13] bits for 13 * 2);
/// ```
/// ```rust
/// w!([0 + 2 + 4]#{6 + 8});
///
/// // Here too!
/// w!([0 + 2 + 4]#6 + 8);
/// ```
///
/// Brackets for literal bit numbers can be omitted:
/// ```rust
/// w!(13 bits for {13 * 2} as u32);
/// ```
///
///
/// Finally, if you just want a new wire:
/// ```rust
/// let w = w!([200 - 100] bits);
/// ```
/// ```rust
/// let w = w!(34 + 1);
/// ```
/// ```rust
/// let w = w!(9000 bits);
/// ```

// Matrix:
//  - form: { for, hash, arrow }
//  - bits: { expr quoted, expr unquoted, literal }
//  - val:  { expr quoted, expr unquoted, literal }
//  - type: { specified, omitted }
//
// ┌──────┬───────┬─────┬──────┰───┐
// │ Form │  Bits │ Val │ Type ┃ ? │
// ┝━━━━━━┿━━━━━━━┿━━━━━┿━━━━━━╋━━━┥
// │ for  │  "e"  │ "e" │ omit ┃ x │ // w!([3 * 3] bits for {3 * 2});
// │ for  │  "e"  │ "e" │  yes ┃ x │ // w!([3 * 3] bits for {3 * 2} as u32);
// │ for  │  "e"  │  e  │ omit ┃ x │ // w!([3 * 3] bits for 3 * 2);
// │ for  │  "e"  │  e  │  yes ┃ - │ // w!([3 * 3] bits for 3 * 2 as u32);
// │ for  │  "e"  │  l  │ omit ┃ x │ // w!([3 * 3] bits for 6);
// │ for  │  "e"  │  l  │  yes ┃ x │ // w!([3 * 3] bits for 6 as u32);
// │ for  │   e   │ "e" │ omit ┃ - │ // w!(3 * 3 bits for {3 * 2});
// │ for  │   e   │ "e" │  yes ┃ - │ // w!(3 * 3 bits for {3 * 2} as u32);
// │ for  │   e   │  e  │ omit ┃ - │ // w!(3 * 3 bits for 3 * 2);
// │ for  │   e   │  e  │  yes ┃ - │ // w!(3 * 3 bits for 3 * 2 as u32);
// │ for  │   e   │  l  │ omit ┃ - │ // w!(3 * 3 bits for 6);
// │ for  │   e   │  l  │  yes ┃ - │ // w!(3 * 3 bits for 6 as u32);
// │ for  │   l   │ "e" │ omit ┃ x │ // w!(9 bits for {3 * 2});
// │ for  │   l   │ "e" │  yes ┃ x │ // w!(9 bits for {3 * 2} as u32);
// │ for  │   l   │  e  │ omit ┃ x │ // w!(9 bits for 3 * 2);
// │ for  │   l   │  e  │  yes ┃ x │ // w!(9 bits for 3 * 2 as u32);
// │ for  │   l   │  l  │ omit ┃ - │ // w!(9 bits for 6);
// │ for  │   l   │  l  │  yes ┃ x │ // w!(9 bits for 6 as u32);
// │  #   │  "e"  │ "e" │ omit ┃ x │ // w!([9 - 2]#{30 - 2 * 10});
// │  #   │  "e"  │ "e" │  yes ┃ x │ // w!([9 - 2]#{30 - 2 * 10}:u128);
// │  #   │  "e"  │  e  │ omit ┃ x │ // w!([9 - 2]#30 - 2 * 10);
// │  #   │  "e"  │  e  │  yes ┃ - │ // w!([9 - 2]#30 - 2 * 10:u128);
// │  #   │  "e"  │  l  │ omit ┃ x │ // w!([9 - 2]#10);
// │  #   │  "e"  │  l  │  yes ┃ x │ // w!([9 - 2]#10:u128);
// │  #   │   e   │ "e" │ omit ┃ - │ // w!(9 - 2#{30 - 2 * 10});
// │  #   │   e   │ "e" │  yes ┃ - │ // w!(9 - 2#{30 - 2 * 10}:u128);
// │  #   │   e   │  e  │ omit ┃ - │ // w!(9 - 2#30 - 2 * 10);
// │  #   │   e   │  e  │  yes ┃ - │ // w!(9 - 2#30 - 2 * 10:u128);
// │  #   │   e   │  l  │ omit ┃ - │ // w!(9 - 2#10);
// │  #   │   e   │  l  │  yes ┃ - │ // w!(9 - 2#10:u128);
// │  #   │   l   │ "e" │ omit ┃ x │ // w!(7#{30 - 2 * 10});
// │  #   │   l   │ "e" │  yes ┃ x │ // w!(7#{30 - 2 * 10}:u128);
// │  #   │   l   │  e  │ omit ┃ x │ // w!(7#30 - 2 * 10);
// │  #   │   l   │  e  │  yes ┃ - │ // w!(7#30 - 2 * 10:u128);
// │  #   │   l   │  l  │ omit ┃ x │ // w!(7#10);
// │  #   │   l   │  l  │  yes ┃ x │ // w!(7#10:u128);
// │  =>  │  "e"  │ "e" │ omit ┃ x │ // w!({20 * 2 - 1} => { 2 * 2 + 4 });
// │  =>  │  "e"  │ "e" │  yes ┃ x │ // w!(u8:{20 * 2 - 1} => { 2 * 2 + 4 });
// │  =>  │  "e"  │  e  │ omit ┃ x │ // w!(20 * 2 - 1 => { 2 * 2 + 4 });
// │  =>  │  "e"  │  e  │  yes ┃ x │ // w!(u8: 20 * 2 - 1 => { 2 * 2 + 4 });
// │  =>  │  "e"  │  l  │ omit ┃ x │ // w!(39 => { 2 * 2 + 4 });
// │  =>  │  "e"  │  l  │  yes ┃ x │ // w!(u8: 39 => { 2 * 2 + 4 });
// │  =>  │   e   │ "e" │ omit ┃ x │ // w!({20 * 2 - 1} => 2 * 2 + 4);
// │  =>  │   e   │ "e" │  yes ┃ x │ // w!(u8:{20 * 2 - 1} => 2 * 2 + 4);
// │  =>  │   e   │  e  │ omit ┃ x │ // w!(20 * 2 - 1 => 2 * 2 + 4);
// │  =>  │   e   │  e  │  yes ┃ x │ // w!(u8: 20 * 2 - 1 => 2 * 2 + 4);
// │  =>  │   e   │  l  │ omit ┃ x │ // w!(39 => 2 * 2 + 4);
// │  =>  │   e   │  l  │  yes ┃ x │ // w!(u8: 39 => 2 * 2 + 4);
// │  =>  │   l   │ "e" │ omit ┃ x │ // w!({20 * 2 - 1} => 8);
// │  =>  │   l   │ "e" │  yes ┃ x │ // w!(u8:{20 * 2 - 1} => 8);
// │  =>  │   l   │  e  │ omit ┃ x │ // w!(20 * 2 - 1 => 8);
// │  =>  │   l   │  e  │  yes ┃ x │ // w!(u8: 20 * 2 - 1 => 8);
// │  =>  │   l   │  l  │ omit ┃ x │ // w!(39 => 8);
// │  =>  │   l   │  l  │  yes ┃ x │ // w!(u8: 39 => 8);
// └──────┴───────┴─────┴──────┸───┘

/// ```rust
/// w!(0 bits for 0);
/// ```
///
#[macro_export(crate)]
macro_rules! w {
    // val expr + bits expr + type
    ([$bits:expr] bits for { $val:expr } as $type:ty) => { $crate::new_wire_with_val!($bits, ($val) as $type) };
    ([$bits:expr]#{$val:expr}:$type:ty) => { $crate::w!([$bits] bits for { $val } as $type) };
    ($type:ty: $val:expr => $bits:expr) => { $crate::w!([$bits] bits for { $val } as $type) };

    // val expr + bits expr
    ([$bits:expr] bits for $val:expr) => { $crate::w!([$bits] bits for { $val } as usize) };
    ([$bits:expr]#{$val:expr}:$type:ty) => { $crate::w!([$bits] bits for $val) };
    ($val:expr => $bits:expr) => { $crate::w!(($bits) bits for { $val } as $type) };

    // val expr + type
    ($bits:literal bits for $val:expr) => { $crate::w!([$bits] bits for $val) };


    // bits expr + type

    // val expr

    // bits expr

    // nothing


    ($bits:literal bits for $val:expr) => { crate::w!(($bits) bits for $val) };

    ($val:expr => ($bits:expr) bits) => { crate::w!(($bits) bits for $val) };
    ($val:expr => $bits:literal bits) => { crate::w!($bits bits for $val) };
    (($bits:expr)#$val:expr) => { crate::w!(($bits) bits for $val) };
    ($bits:literal#$val:expr) => { crate::w!($bits bits for $val) };

    // new wire
    ([$bits:expr] bits) => { crate::new_wire!($bits) };
    ($bits:literal bits) => { crate::w!([$bits] bits) };
    ($bits:expr) => { crate::w!([$bits] bits) };
}
