#![feature(proc_macro_hygiene)]
#![feature(const_generics)]
#![allow(incomplete_features)]

#[macro_use(repeat_with_n)]
extern crate repeat_macros;


#[cfg(test)]
mod tests {
    trait Foo { }

    #[allow(unused)]
    struct Bar<const B: usize>;

    #[test]
    fn test() {
        repeat_with_n!(10, n, { impl Foo for Bar<{n as usize}> {} });
    }

    #[test]
    fn with_macro() {
        repeat_with_n!(10, longer_identifier, { println!("{}", longer_identifier); });
    }

    #[test]
    fn nested_macro() {
        repeat_with_n!(10, y, { println!("{}", format!("{}", format!("{}{}{}{}", y, y, y, y))); });
    }
}
