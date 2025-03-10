use std::collections::{HashMap, HashSet};

use ungrammar::{Grammar, Node, Rule, Token};

pub(super) struct FirstSet(HashMap<Node, HashSet<Token>>);

impl FirstSet {
    pub(super) fn new() -> Self {
        FirstSet(HashMap::new())
    }

    pub(super) fn contains(&self, node: &Node) -> bool {
        self.0.contains_key(node)
    }

    pub(super) fn insert(&mut self, node: Node, set: HashSet<Token>) -> Option<HashSet<Token>> {
        self.0.insert(node, set)
    }

    pub(super) fn get(&self, node: &Node) -> Option<&HashSet<Token>> {
        self.0.get(node)
    }

    pub(super) fn get_first_of_sorted(&self, rule: &Rule, grammar: &Grammar) -> Vec<Token> {
        let mut first_set = self
            .get_first_of(rule, grammar)
            .into_iter()
            .collect::<Vec<Token>>();
        first_set.sort();
        return first_set;
    }
    pub(super) fn get_first_of(&self, rule: &Rule, grammar: &Grammar) -> HashSet<Token> {
        match rule {
            Rule::Rep(other)
            | Rule::Opt(other)
            | Rule::Labeled {
                label: _,
                rule: other,
            } => self.get_first_of(other, grammar),
            Rule::Node(node) => self
                .get(node)
                .expect("Every node should have a first-set")
                .clone(),
            Rule::Seq(rules) => {
                let mut set: HashSet<Token> = HashSet::new();
                for rule in rules.iter() {
                    set.extend(&self.get_first_of(rule, grammar).clone());
                    if !is_nullable(rule, grammar) {
                        break;
                    }
                }
                set
            }
            Rule::Alt(rules) => {
                let set = rules.iter().fold(HashSet::new(), |mut accu, rule| {
                    accu.extend(self.get_first_of(rule, grammar));
                    accu
                });
                set
            }
            Rule::Token(token) => HashSet::from([*token]),
        }
    }
}

pub(super) fn compute_first(grammar: &Grammar) -> FirstSet {
    let mut first = FirstSet::new();
    for node in grammar.iter() {
        compute_first_helper(node, &grammar, &mut first);
    }
    return first;
}

fn compute_first_helper(node: Node, grammar: &Grammar, first_set: &mut FirstSet) {
    if !first_set.contains(&node) {
        let set = get_first(&grammar[node].rule, grammar, first_set);
        first_set.insert(node, set);
    }
}

pub(super) fn is_nullable(rule: &Rule, grammar: &Grammar) -> bool {
    match rule {
        Rule::Token(_) => false,
        Rule::Opt(_) => true,
        Rule::Rep(_) => true,
        Rule::Labeled {
            label: _,
            rule: other,
        } => is_nullable(other, grammar),
        Rule::Node(node) => is_nullable(&grammar[*node].rule, grammar),
        Rule::Seq(rules) => rules.iter().all(|rule| is_nullable(rule, grammar)),
        Rule::Alt(rules) => rules.iter().any(|rule| is_nullable(rule, grammar)),
    }
}

pub(super) fn get_first(
    rule: &Rule,
    grammar: &Grammar,
    first_set: &mut FirstSet,
) -> HashSet<Token> {
    match rule {
        Rule::Rep(other)
        | Rule::Opt(other)
        | Rule::Labeled {
            label: _,
            rule: other,
        } => get_first(other, grammar, first_set),
        Rule::Node(node) => {
            compute_first_helper(*node, grammar, first_set);
            first_set
                .get(node)
                .expect("FIRST name should have been connected")
                .clone()
        }
        Rule::Seq(rules) => {
            let mut set: HashSet<Token> = HashSet::new();
            for rule in rules.iter() {
                set.extend(get_first(rule, grammar, first_set));
                if !is_nullable(rule, grammar) {
                    break;
                }
            }
            set
        }
        Rule::Alt(rules) => rules.iter().fold(HashSet::new(), |mut accu, rule| {
            accu.extend(get_first(rule, grammar, first_set));
            accu
        }),
        Rule::Token(token) => HashSet::from([*token]),
    }
}

// fn compute_follow_set(
//     non_term: &String,
//     grammar: &Grammar,
//     first_set: &FirstSet,
//     follow_set: &mut FollowSet,
// ) {
// if follow_set.contains_key(non_term) {
//     return;
// }
// follow_set.insert(non_term.clone(), HashSet::new());
// if Term::NonTerminal(non_term.clone()) == grammar.start {
//     follow_set.entry(non_term.to_string()).and_modify(|e| {
//         e.insert("$".to_string());
//     });
// }
// grammar
//     .rules
//     .iter()
//     .filter_map(|rule| match rule {
//         Rule(_lhs, Term::Concatination(terms)) => {
//             if terms.iter().any(|term| match term {
//                 Term::NonTerminal(nt) if nt == non_term => true,
//                 _ => false,
//             }) {
//                 return Some(rule);
//             }
//             return None;
//         }
//         _ => None,
//     })
//     .for_each(|rule| {
//         match &rule.1 {
//             Term::Concatination(terms) => {
//                 terms
//                     .iter()
//                     .zip(terms.iter().skip(1))
//                     .filter_map(|(elem, next)| match elem {
//                         Term::NonTerminal(nt) if nt == non_term => Some(next),
//                         _ => None,
//                     })
//                     .for_each(|elem| match elem {
//                         Term::Terminal(t) => {
//                             follow_set
//                                 .entry(non_term.clone())
//                                 .and_modify(|e| {
//                                     e.insert(t.clone());
//                                 })
//                                 .or_insert(HashSet::from([t.clone()]));
//                         }
//                         Term::NonTerminal(nt) => {
//                             let mut first = first_set
//                                 .get(nt)
//                                 .expect(&format!(
//                                     "FIRST of {} should have beed calculated already",
//                                     nt
//                                 ))
//                                 .clone();
//                             if first.contains("''") {
//                                 first.remove("''");
//                                 compute_follow_set(nt, grammar, first_set, follow_set);
//                                 let follow = follow_set.get(nt).expect(&format!(
//                                     "FIRST of {} should have beed calculated already",
//                                     nt
//                                 ));
//                                 first.extend(follow.clone());
//                             }
//                             follow_set
//                                 .entry(non_term.clone())
//                                 .and_modify(|e| {
//                                     e.extend(first.clone());
//                                 })
//                                 .or_insert(first);
//                         }
//                         _ => {}
//                     });
//                 match terms.last() {
//                     Some(Term::NonTerminal(nt)) if nt == non_term => {
//                         compute_follow_set(&rule.0, grammar, first_set, follow_set);
//                         let set = follow_set
//                             .get(&rule.0)
//                             .expect(&format!(
//                                 "FOLLOW of {} should have beed calculated already",
//                                 nt
//                             ))
//                             .clone();
//                         follow_set
//                             .entry(non_term.clone())
//                             .and_modify(|e| {
//                                 e.extend(set.clone());
//                             })
//                             .or_insert(set);
//                     }
//                     _ => {}
//                 }
//             }
//             _ => panic!("All rules in the computation of FOLLOW should be concatinations"),
//         };
//     });
// }
