#![feature(proc_macro_hygiene)]

#[macro_use(repeat_with_n)]
extern crate repeat_macro;


#[cfg(test)]
mod tests {
    use super::*;

    fn test() {
        repeat_with_n!(10, n, (println!("{}", n)));
    }

    #[test]
    fn name() {
        unimplemented!();
    }
}
