#[macro_use(repeat_with_n)]
extern crate repeat_macros;

#[cfg(test)]
mod tests {
    // A compile test:
    #![allow(unused)]
    struct Foo {}
    trait Bar {
        const BAZ: usize;
    }
    repeat_with_n! {0, n, impl Bar for Foo { const BAZ: usize = n; }}

    #[test]
    fn compiles() {
        assert!(true);
    }
}
