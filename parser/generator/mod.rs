mod utils;
use std::{
    fs::File,
    io::{Read, Write},
    str::FromStr,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use ungrammar::{Grammar, Rule, Token};
use utils::{compute_first, is_nullable, FirstSet};

pub fn generate() {
    let mut file = File::open("sparql.ungram").expect("File should exist");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("could not read file");
    let grammar = ungrammar::Grammar::from_str(&contents).unwrap();
    let first = compute_first(&grammar);

    generate_types(&grammar);
    generate_parser(&grammar, &first);
    generate_rules(&grammar);
}

fn generate_rule(grammar: &Grammar, rule: &Rule, first: &FirstSet) -> TokenStream {
    match rule {
        Rule::Labeled {
            label: _,
            rule: other,
        } => generate_rule(grammar, other, first),
        Rule::Node(node) => {
            let ident = format_ident!("parse_{}", grammar[*node].name);
            quote! {#ident (p);}
        }
        Rule::Token(token) => {
            let ident = generate_token_kind(&grammar[*token].name);
            quote! {p.expect(SyntaxKind::#ident);}
        }
        Rule::Seq(rules) => rules
            .iter()
            .map(|other| generate_rule(grammar, other, first))
            .collect(),
        Rule::Alt(rules) => {
            let match_arms: Vec<TokenStream> = rules
                .iter()
                .map(|other_rule| {
                    let tokens: Vec<TokenStream> = first
                        .get_first_of_sorted(other_rule, grammar)
                        .iter()
                        .map(|token| {
                            let kind = generate_token_kind(&grammar[*token].name);
                            quote! { SyntaxKind::#kind }
                        })
                        .collect();
                    let parse_rule = generate_rule(grammar, other_rule, first);
                    quote! {
                        #(#tokens )|* => {
                            #parse_rule
                        }
                    }
                })
                .collect();
            let catch_arm = match is_nullable(rule, grammar) {
                true => quote! {
                    _ => {}
                },
                false => quote! {
                    _ =>{
                        p.advance_with_error("Expected ....");
                    }
                },
            };
            quote! {
                match p.nth(0){
                  #(#match_arms)*,
                  SyntaxKind::Eof => {
                        eprintln!("Unexpected Eof");
                        p.close(marker, SyntaxKind::Error);
                        return
                  },
                  #catch_arm
                };
            }
        }
        Rule::Opt(other_rule) => {
            let first_set: Vec<TokenStream> = first
                .get_first_of_sorted(other_rule, grammar)
                .iter()
                .map(|token| generate_token_kind(&grammar[*token].name))
                .map(|ident| quote! {SyntaxKind::#ident})
                .collect();

            let parse_rule = generate_rule(grammar, other_rule, first);
            quote! {
                if p.at_any(&[#(#first_set),*]){
                #parse_rule
                }
            }
        }
        Rule::Rep(other_rule) => {
            let first_set: Vec<TokenStream> = generate_first_set(first, rule, grammar);
            let parse_rule = generate_rule(grammar, other_rule, first);
            quote! {
                while [#(#first_set),*].contains(&p.nth(0)) {
                    #parse_rule
                }
            }
        }
    }
}

fn generate_first_set(first: &FirstSet, rule: &Rule, grammar: &Grammar) -> Vec<TokenStream> {
    let mut first_set = first
        .get_first_of(rule, grammar)
        .into_iter()
        .collect::<Vec<Token>>();
    first_set.sort();
    first_set
        .iter()
        .map(|token| generate_token_kind(&grammar[*token].name))
        .map(|ident| quote! {SyntaxKind::#ident})
        .collect()
}

fn generate_token_kind(token: &str) -> proc_macro2::Ident {
    format_ident!(
        "{}",
        match token {
            "*" => "Star",
            "+" => "Plus",
            "-" => "Minus",
            "(" => "LParen",
            ")" => "RParen",
            "{" => "LCurly",
            "}" => "RCurly",
            "[" => "LBrack",
            "]" => "RBrack",
            "." => "Dot",
            "," => "Colon",
            ";" => "Semicolon",
            "|" => "Pipe",
            "||" => "DoublePipe",
            "/" => "Slash",
            "^" => "Zirkumflex",
            "^^" => "DoubleZirkumflex",
            "?" => "QuestionMark",
            "$" => "Dollar",
            "!" => "ExclamationMark",
            "&&" => "DoubleAnd",
            "=" => "Equals",
            "!=" => "ExclamationMarkEquals",
            "<" => "Less",
            "<=" => "LessEquals",
            ">" => "More",
            ">=" => "MoreEquals",
            "true" => "True",
            "false" => "False",
            other => other,
        }
    )
}

fn foo(grammar: &Grammar, rule: &Rule) -> TokenStream {
    match rule {
        Rule::Labeled {
            label: _label,
            rule,
        } => foo(grammar, rule),
        Rule::Node(node) => {
            let ident = format_ident!("{}", grammar[*node].name);
            quote! {
                Rule::Node(SyntaxKind::#ident)
            }
        }

        Rule::Token(token) => {
            let ident = format_ident!("{}", generate_token_kind(&grammar[*token].name));
            quote! {
                Rule::Token(SyntaxKind::#ident)
            }
        }
        Rule::Seq(rules) => {
            let sub_rules: Vec<_> = rules.iter().map(|rule| foo(grammar, rule)).collect();
            quote! {
                Rule::Seq(vec![
                    #(#sub_rules),*
                ])
            }
        }
        Rule::Alt(rules) => {
            let sub_rules: Vec<_> = rules.iter().map(|rule| foo(grammar, rule)).collect();
            quote! {
                Rule::Alt(vec![
                    #(#sub_rules),*
                ])
            }
        }
        Rule::Opt(rule) => {
            let sub_rule = foo(grammar, rule);
            quote! {
                Rule::Opt(Box::new(#sub_rule))
            }
        }
        Rule::Rep(rule) => {
            let sub_rule = foo(grammar, rule);
            quote! {
                Rule::Rep(Box::new(#sub_rule))
            }
        }
    }
}

fn generate_rules(grammar: &Grammar) {
    let arms = grammar.iter().map(|node| {
        let ident = format_ident!("{}", grammar[node].name);
        let rule = foo(grammar, &grammar[node].rule);
        quote! {
            SyntaxKind::#ident => {
                Some(#rule)
            }
        }
    });
    let tokens = quote! {
        use crate::syntax_kind::SyntaxKind;
        #[derive(Debug, Clone)]
        pub (super) enum Rule {
            Node(SyntaxKind),
            Token(SyntaxKind),
            Seq(Vec<Rule>),
            Alt(Vec<Rule>),
            Opt(Box<Rule>),
            Rep(Box<Rule>),
        }
        impl Rule {
            pub(super) fn first(&self) -> Vec<SyntaxKind> {
                match self {
                    Rule::Node(syntax_kind) | Rule::Token(syntax_kind) => vec![*syntax_kind],
                    Rule::Seq(rules) => rules.first().unwrap().first(),
                    Rule::Alt(rules) => rules.iter().flat_map(|rule| rule.first()).collect(),
                    Rule::Opt(rule) => rule.first(),
                    Rule::Rep(rule) => rule.first(),
                }
            }
            pub(super) fn is_nullable(&self) -> bool {
                match self {
                    Rule::Opt(_rule) | Rule::Rep(_rule) => true,
                    _ => false,
                }
            }
            pub(super) fn from_node_kind(kind: SyntaxKind) -> Option<Self> {
                match kind {
                    #(#arms)*
                    _ => None
                }
            }
        }

    };

    let syntax_tree = syn::parse2(tokens).unwrap();
    let formatted_code = prettyplease::unparse(&syntax_tree);

    let mut file = File::create("src/rules.rs").unwrap();
    file.write_all(formatted_code.as_bytes()).unwrap();
}

fn generate_types(grammar: &Grammar) {
    let token_kinds = grammar
        .tokens()
        .map(|token| generate_token_kind(grammar[token].name.as_str()))
        .map(|token| {
            let x = format!("{}", token);
            quote! {

                    #[token(#x, ignore(case))]
                    #token
            }
        });
    let parser_trees = grammar
        .iter()
        .map(|node| format_ident!("{}", grammar[node].name));

    let tokens = quote! {
        use logos::{Logos, Span};

        #[allow(non_camel_case_types)]
        #[derive(Logos, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize)]
        #[repr(u16)]
        pub enum SyntaxKind  {
            Eof = 0,
            Error,
            #[regex(r#"[ \t\n\f]+"#)]
            WHITESPACE,
            #(#token_kinds),*,
            #(#parser_trees),*
        }
    };

    let syntax_tree = syn::parse2(tokens).unwrap();
    let formatted_code = prettyplease::unparse(&syntax_tree);

    let mut file = File::create("gen_kinds.rs").unwrap();
    file.write_all(formatted_code.as_bytes()).unwrap();
}

fn format_rule(grammar: &Grammar, rule: &Rule) -> String {
    match rule {
        ungrammar::Rule::Labeled {
            label: _,
            rule: other,
        } => format_rule(grammar, other),
        ungrammar::Rule::Node(node) => grammar[*node].name.clone(),
        ungrammar::Rule::Token(token) => format!("'{}'", grammar[*token].name.clone()),
        ungrammar::Rule::Seq(rules) => rules
            .iter()
            .map(|other| {
                let formatted = format_rule(grammar, other);
                match other {
                    Rule::Alt(_) => format!("({})", formatted),
                    _ => formatted,
                }
            })
            .collect::<Vec<String>>()
            .join(" "),
        ungrammar::Rule::Alt(rules) => rules
            .iter()
            .map(|other| format_rule(grammar, other))
            .collect::<Vec<String>>()
            .join(" | "),
        ungrammar::Rule::Opt(other) => match **other {
            Rule::Seq(_) | Rule::Alt(_) => format!("({})?", format_rule(grammar, other)),
            _ => format!("{}?", format_rule(grammar, other)),
        },

        ungrammar::Rule::Rep(other) => match **other {
            Rule::Seq(_) | Rule::Alt(_) => format!("({})*", format_rule(grammar, other)),
            _ => format!("{}*", format_rule(grammar, other)),
        },
    }
}

fn generate_parser(grammar: &Grammar, first: &FirstSet) {
    let functions = grammar.iter().enumerate().map(|(idx, node)| {
        let name = &grammar[node].name;
        let rule = &grammar[node].rule;
        let comment = format!(" [{}] {} -> {}", idx, name, format_rule(grammar, rule));
        let tree_kind = format_ident!("{}", name);
        let function_name = format_ident!("parse_{}", name);
        let rules = generate_rule(grammar, rule, first);
        let nullable = is_nullable(rule, grammar);
        let first_set = generate_first_set(first, rule, grammar);
        let escape = match nullable {
            false => quote! {},
            true => quote! {
                if !p.at_any(&[#(#first_set),*]){
                    return;
                }
            },
        };
        quote! {
            #[doc = #comment]
            pub (super) fn #function_name (p: &mut Parser){
                #escape
                let marker = p.open();
                #rules
                p.close(marker, SyntaxKind::#tree_kind);
            }
        }
    });
    let parser = quote! {
        #![allow(non_snake_case)]
        use crate::SyntaxKind;
        use super::Parser;
         #(#functions)*
    };

    let syntax_tree = syn::parse2(parser).unwrap();
    let formatted_code = prettyplease::unparse(&syntax_tree);

    let mut file = File::create("src/parser/grammar.rs").unwrap();
    file.write_all(formatted_code.as_bytes()).unwrap();
}
