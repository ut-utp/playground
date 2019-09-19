#![recursion_limit = "128"]

extern crate proc_macro;

use syn::Block;
use syn::Item;
use std::iter::FromIterator;
use quote::ToTokens;
use syn::{Expr, ExprPath};
use proc_macro2::{Literal, Ident, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use std::collections::VecDeque;
use syn::visit_mut::{self, VisitMut};
use syn::parse2;
use syn::{parse, parse_quote};
// use proc_macro::TokenStream;

/// Report an error with the given `span` and message.
fn spanned_err(span: Span, msg: impl Into<String>) -> proc_macro::TokenStream {
    let msg = msg.into();
    quote_spanned!(span.into() => {
        compile_error!(#msg);
    })
    .into()
}

/// Use like: `repeat!(num, <tokens to repeat>)` where num is a positive integer literal
/// and is followed by a comma and a non-zero number of tokens to repeat.
///
/// Based pretty heavily on this: https://stackoverflow.com/a/54351072/3006245
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macro;
/// fn no_tokens() {
///    println!("{}", repeat!());
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macro;
/// fn missing_repeated_tokens() {
///     println!("{}", repeat!(789,));
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macro;
/// fn missing_comma() {
///     println!("{}", repeat!(89 "yay" "go",));
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macro;
/// fn non_integer() {
///     println!("{}", repeat!("yay", "yay"));
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macro;
/// fn negative_integer() {
///     println!("{}", repeat!(-789, "yay", ));
/// }
/// ```
///
/// ```ignore
/// # #[macro_use] extern crate repeat_macro;
/// println!("{}", repeat!(1, "yay"));
/// ```
#[proc_macro]
pub fn repeat(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let mut tokens = input.into_iter().collect::<VecDeque<_>>();

    // if let Some(err) = match tokens.len() {
    //     0 => Some(spanned_err(
    //         Span::call_site(),
    //         "We expected a number and some tokens to repeat, but got nothing.",
    //     )),
    //     1 | 2 => Some(spanned_err(
    //         tokens.last().unwrap().span(),
    //         "We expected a number *and* some tokens to repeat, but didn't get any \
    //          tokens to repeat.",
    //     )),
    //     _ => None,
    // } {
    //     return err;
    // }

    let num_times: usize = match tokens.pop_front() {
        None => {
            return spanned_err(
                Span::call_site(),
                "We expected a number and some tokens to repeat, but got nothing.",
            )
        }
        Some(TokenTree::Literal(l)) => match l.to_string().parse() {
            Ok(unsigned) => unsigned,
            Err(err) => {
                return spanned_err(
                    l.span(),
                    format!(
                        "We expected an unsigned integer literal but got {} \
                         resulting in the following error when parsing as a usize: {}",
                        l, err
                    ),
                );
            }
        },
        Some(other) => {
            return spanned_err(
                other.span(),
                format!("We expected an unsigned integer literal but got {}.", other),
            )
        }
    };

    match match tokens.pop_front() {
        None => {
            return spanned_err(
                tokens.back().unwrap().span(),
                "We expected a number, a comma, *and* some tokens to repeat, but we didn't \
                 find any tokens after the number.",
            )
        }
        Some(tok) => match tok {
            TokenTree::Punct(ref p) => {
                if let ',' = p.as_char() {
                    Ok(())
                } else {
                    Err(tok)
                }
            }
            _ => Err(tok),
        }, // Some(TokenTree::Punct(ref p)) => {
           //     if let ',' = p.as_char() { Ok(()) }
           //     else { Err(p) }
           // }
           // Some(other) => { Err(other) }
    } {
        Err(tok) => {
            return spanned_err(
                tok.span(),
                format!(
                    "We expected a comma after the number of times to repeat but got {}.",
                    tok
                ),
            )
        }
        Ok(_) => {}
    }

    if tokens.len() == 0 {
        return spanned_err(
            Span::call_site(),
            "We expected a number *and* some tokens to repeat, but didn't get any \
             tokens to repeat.",
        );
    }

    // let next_tok = tokens.pop_front().unwrap();
    // match if let TokenTree::Punct(ref p) = next_tok {
    //     if let ',' = p.as_char() {
    //         Some(())
    //     } else {
    //         None
    //     }
    // } else {
    //     None
    // } {
    //     None => {
    //         return spanned_err(
    //             next_tok.span(),
    //             format!(
    //                 "We expected a comma after the number of times to repeat but got {}.",
    //                 next_tok
    //             ),
    //         );
    //     }
    //     _ => {}
    // }

    let tokens: Vec<_> = tokens.iter().collect();

    core::iter::repeat(quote! {#(#tokens)*})
        .map(proc_macro::TokenStream::from)
        .take(num_times)
        .collect()

    // proc_macro::TokenStream::from(output)
}

fn parse_comma(tokens: &mut VecDeque<TokenTree>) -> Result<(), proc_macro::TokenStream> {
    tokens
        .pop_front()
        .ok_or_else(|| spanned_err(Span::call_site(), "Expected a comma; ran out of tokens."))
        .and_then(|tok| {
            if let TokenTree::Punct(ref p) = tok {
                if let ',' = p.as_char() {
                    Ok(())
                } else {
                    Err(tok)
                }
            } else {
                Err(tok)
            }
            .map_err(|tok| spanned_err(tok.span(), "Expected a comma."))
        })
}

// fn substitute_token(
//     identifier_name: &String,
//     substitution: TokenTree,
//     tokens: &Vec<&TokenTree>,
// ) -> Vec<TokenTree> {
//     tokens
//         .iter()
//         .map(|tok| {
//             println!("TOK! {} vs. {}", tok.to_string(), *identifier_name);

//             if tok.to_string() == *identifier_name {
//                 substitution.clone()
//             } else {
//                 (*tok).clone()
//             }
//         })
//         .collect()
// }

struct IdentifierReplace<'a> {
    identifier_name: &'a Ident,
    substitution: &'a Expr
}

impl VisitMut for IdentifierReplace<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // println!("hit: {:?}", expr);
        if let Expr::Path(ExprPath { path, ..}) = expr {
            println!("yo! we got: {:?}", path.get_ident());
            if path.is_ident(self.identifier_name) {
                println!("match!!");
                *expr = self.substitution.clone();
                return;
            }
            println!("no match!!!!");
        }

        visit_mut::visit_expr_mut(self, expr);
    }
}

fn substitute_token(
    identifier_name: &Ident,
    substitution: &Expr,
    tokens: TokenStream,
) -> TokenStream {
    let mut id_replace = IdentifierReplace { identifier_name, substitution };

    // parse2(tokens.clone()) // Try as an item
    //     .as_mut()
    //     .map(|mut t| {
    //         <IdentifierReplace as VisitMut>::visit_item_mut(&mut id_replace, &mut t);
    //         t.to_token_stream()
    //     })
    //     .or_else(|_| {
    //         parse2(tokens.clone())
    //             .as_mut()
    //             .map(|mut t| {
    //                 <IdentifierReplace as VisitMut>::visit_expr_mut(&mut id_replace, &mut t);
    //                 t.to_token_stream()
    //             })
    //     })
    //     .unwrap()

    if let Ok(mut item) = parse2(tokens.clone()) {
        <IdentifierReplace as VisitMut>::visit_item_mut(&mut id_replace, &mut item);
        item.to_token_stream()
    } else if let Ok(mut expr) = parse2(tokens.clone()) {
        <IdentifierReplace as VisitMut>::visit_expr_mut(&mut id_replace, &mut expr);
        expr.to_token_stream()
    } else {
        let i = parse2::<Item>(tokens.clone());
        let b = parse2::<Block>(tokens.clone());

        spanned_err(Span::call_site(),
            format!("Couldn't parse as an item or a block. \
                Item: {:?} \
                Block: {:?} ",
                    i.err().unwrap(), b.err().unwrap())).into()
    }

    // <IdentifierReplace as VisitMut>::visit_item_mut(&mut id_replace, &mut tokens);

    // tokens.to_token_stream()
}

/// Use like: `repeat_with_n!{ num, var_name, <tokens_to_repeat> }`.
#[proc_macro]
pub fn repeat_with_n(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    println!("{:#?}", input);

    let input = TokenStream::from(input);
    let mut tokens = input.into_iter().collect::<VecDeque<_>>();

    // Number of times to repeat:
    let num = if let Some(TokenTree::Literal(l)) = tokens.pop_front() {
        l.to_string().parse().unwrap()
    } else {
        return spanned_err(
            Span::call_site(),
            "Expected unsigned number as first argument.",
        );
    };

    // Comma:
    if let Err(ts) = parse_comma(&mut tokens) {
        return ts;
    }

    // Var:
    let var: Ident = if let Some(tok) = tokens.pop_front() {
        if let TokenTree::Ident(id) = tok {
            id
        } else {
            return spanned_err(tok.span(), "Expected an identifier.");
        }
    } else {
        return spanned_err(
            Span::call_site(),
            "Expected identifier as the second argument.",
        );
    };

    // Another Comma:
    if let Err(ts) = parse_comma(&mut tokens) {
        return ts;
    }

    // The rest of the tokens:
    // (we don't care if this is empty!)
    let tokens: Vec<_> = tokens.iter().collect();
    let ts = (0..=num)
        .map(|n| parse_quote!(#n))
        .map(|e: Expr| substitute_token(&var, &e, TokenStream::from_iter(tokens.iter().map(|tt| tt.clone().clone()))))
        .fold(TokenStream::new(), |acc, ts| quote!{ #acc #ts });

    // println!("{:#?}", ts);
    // println!("ret: {}", ts);
    proc_macro::TokenStream::from(ts)

    // let ts = (0..=num)
    //     .map(|n| Literal::usize_suffixed(n))
    //     .map(TokenTree::Literal)
    //     .map(|tt| substitute_token(&var, tt, &tokens))
    //     .map(|tt| quote! {#(#tt)*})
    //     .fold(TokenStream::new(), |acc, ts| quote!{ #acc #ts });

    // println!("{}", ts);

    // proc_macro::TokenStream::from(ts)


    // if let Some(tok) = tokens.pop_front() {
    //     match if let TokenTree::Punct(ref p) = tok {
    //         if let ',' = p.as_char() { Ok(()) }
    //         else { Err(tok) }
    //     }
    //     else { Err(tok) }
    //     {
    //         Err(tok) => return spanned_err(tok.span(), "Expected a comma."),
    //         _ => {}
    //     }
    // } else {
    //     return spanned_err(Span::call_site(), "Expected a comma; ran out of tokens.");
    // }

    // let num = tokens.pop_front().unwrap()
}
