extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned, ToTokens};
use std::collections::VecDeque;
use std::iter::FromIterator;
use syn::visit_mut::{self, VisitMut};
use syn::{parse2, parse_quote, Block, Expr, ExprPath, Item};

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
/// Based pretty heavily on [this](https://stackoverflow.com/a/54351072/3006245).
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macros;
/// fn no_tokens() {
///    println!("{}", repeat!());
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macros;
/// fn missing_repeated_tokens() {
///     println!("{}", repeat!(789,));
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macros;
/// fn missing_comma() {
///     println!("{}", repeat!(89 "yay" "go",));
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macros;
/// fn non_integer() {
///     println!("{}", repeat!("yay", "yay"));
/// }
/// ```
///
/// ```rust,compile_fail
/// # #[macro_use] extern crate repeat_macros;
/// fn negative_integer() {
///     println!("{}", repeat!(-789, "yay", ));
/// }
/// ```
///
/// ```ignore
/// # #[macro_use] extern crate repeat_macros;
/// println!("{}", repeat!(1, "yay"));
/// ```
#[proc_macro]
pub fn repeat(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = TokenStream::from(input);
    let mut tokens = input.into_iter().collect::<VecDeque<_>>();

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
        },
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

    let tokens: Vec<_> = tokens.iter().collect();

    core::iter::repeat(quote! {#(#tokens)*})
        .map(proc_macro::TokenStream::from)
        .take(num_times)
        .collect()
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

struct IdentifierReplace<'a> {
    identifier_name: &'a Ident,
    substitution: usize,
}

impl IdentifierReplace<'_> {
    fn modify_token_stream(&mut self, ts: &mut TokenStream) {
        *ts = ts
            .clone()
            .into_iter()
            .map(|tt| {
                use TokenTree::*;
                match tt {
                    Group(g) => {
                        let delim = g.delimiter();
                        let mut ts = g.stream();

                        self.modify_token_stream(&mut ts);

                        Group(proc_macro2::Group::new(delim, ts))
                    }
                    Ident(id) => {
                        if id == *self.identifier_name {
                            Literal(proc_macro2::Literal::usize_suffixed(self.substitution))
                        } else {
                            Ident(id)
                        }
                    }
                    a @ Punct(_) | a @ Literal(_) => a,
                }
            })
            .collect();
    }
}

impl VisitMut for IdentifierReplace<'_> {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Path(ExprPath { path, .. }) = expr {
            if path.is_ident(self.identifier_name) {
                let num = self.substitution;
                *expr = parse_quote!(#num);
                return;
            }
        } else if let Expr::Macro(expr_mac) = expr {
            // Can't make assumptions about what's valid for this macro, so we can't
            // just recurse and call `substitute_token`.

            // Instead we'll just recurse through the TokenStreams
            self.modify_token_stream(&mut expr_mac.mac.tokens);
        }

        visit_mut::visit_expr_mut(self, expr);
    }

    fn visit_item_mut(&mut self, item: &mut Item) {
        // We've got to check for macros here too:
        // (I'm fairly sure Item::Macro2 corresponds to macro definitions (with the
        // (`macro` keyword) -- all macro invocations in an item are mapped to
        // `Item::Macro`)
        if let Item::Macro(item_mac) = item {
            // Same deal as in `visit_expr_mut`:
            self.modify_token_stream(&mut item_mac.mac.tokens);
        }

        visit_mut::visit_item_mut(self, item);
    }
}

fn substitute_token(
    identifier_name: &Ident,
    substitution: usize,
    tokens: TokenStream,
) -> TokenStream {
    let mut id_replace = IdentifierReplace {
        identifier_name,
        substitution,
    };

    if let Ok(mut item) = parse2(tokens.clone()) {
        <IdentifierReplace as VisitMut>::visit_item_mut(&mut id_replace, &mut item);
        item.to_token_stream()
    } else if let Ok(mut expr) = parse2(tokens.clone()) {
        <IdentifierReplace as VisitMut>::visit_expr_mut(&mut id_replace, &mut expr);
        expr.to_token_stream()
    } else {
        // let mut ts = tokens;
        // id_replace.modify_token_stream(&mut ts);
        // ts

        let i = parse2::<Item>(tokens.clone());
        let b = parse2::<Block>(tokens.clone());

        spanned_err(
            Span::call_site(),
            format!(
                "Couldn't parse as an item or a block. \
                 Item: {:?} \
                 Block: {:?} ",
                i.err().unwrap(),
                b.err().unwrap()
            ),
        )
        .into()
    }
}

/// Use like: `repeat_with_n!{ num, var_name, { <tokens_to_repeat> } }`.
/// Or: `repeat_with_n!(num, var_name, { <tokens_to_repeat> });`.
/// Braces around the tokens to repeat _should_ be optional.
#[proc_macro]
pub fn repeat_with_n(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
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
        .map(|n| {
            substitute_token(
                &var,
                n,
                TokenStream::from_iter(tokens.iter().map(|tt| tt.clone().clone())),
            )
        })
        .fold(TokenStream::new(), |acc, ts| quote! { #acc #ts });

    proc_macro::TokenStream::from(ts)
}
