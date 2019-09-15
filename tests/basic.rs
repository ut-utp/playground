#![feature(proc_macro_hygiene)]

#[macro_use(repeat)]
extern crate repeat_macro;

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // #[should_panic]
    // fn no_tokens() {
    //    println!("{}", repeat!());
    // }

    // #[test]
    // #[should_panic]
    // fn missing_repeated_tokens() {
    //     println!("{}", repeat!(789,));
    // }

    // #[test]
    // #[should_panic]
    // fn missing_comma() {
    //     println!("{}", repeat!(89 "yay" "go",));
    // }

    // #[test]
    // #[should_panic]
    // fn non_integer() {
    //     println!("{}", repeat!("yay", "yay"));
    // }

    // #[test]
    // #[should_panic]
    // fn negative_integer() {
    //     println!("{}", repeat!(-789, "yay", ));
    // }

    #[test]
    fn simple() {
        assert_eq!("AH", format!("A{}", repeat!(1, "H")));
    }

    macro_rules! append_to_string {
        [$str_var:ident, $str:literal, $t:ident $($r:ident)*] => {
            $str_var.push_str($str);

            append_to_string![$str_var, $str, $($r)*]
        };
        [$str_var:ident, $str:literal,] => {
            $str_var.push_str($str);
        }

    }

    // macro_rules! ats {
    //     ($str_var:ident, $str:literal, $num:literal) => {
    //         append_to_string![$str_var, $str, repeat!($num, T)]
    //     };
    // }

    #[test]
    fn test_append_to_string_macro() {
        let mut s = String::from("This string is very");

        append_to_string![s, ", very", T T];
        s.push_str(" long.");

        assert_eq!(s, "This string is very, very, very, very long.");
    }

    // #[test]
    // fn test_repeat_expansion_in_macro() {
    //     let mut s = String::from("This string is very");

    //     ats!(s, ", very", 2);
    // }
}
