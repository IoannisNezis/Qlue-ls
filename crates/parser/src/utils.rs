use rowan::TextSize;

use crate::{rules::Rule, syntax_kind::SyntaxKind, SyntaxNode};

pub fn continuations_at(root: &SyntaxNode, mut offset: TextSize) -> Option<Vec<SyntaxKind>> {
    let token = if offset < root.text_range().start() {
        return None;
    } else if offset >= root.text_range().end() {
        let last_token = root.last_token()?;
        offset = last_token.text_range().end();
        last_token
    } else {
        match root.token_at_offset(offset) {
            rowan::TokenAtOffset::Single(token) => Some(token),
            rowan::TokenAtOffset::Between(token1, _) => Some(token1),
            rowan::TokenAtOffset::None => None,
        }?
    };
    let mut parent = token.parent()?;

    let mut children_stack: Vec<_> = parent.children_with_tokens().collect();
    children_stack.reverse();
    let mut rule_stack = vec![Rule::from_node_kind(parent.kind())?];
    let mut result = vec![];
    loop {
        if let Some(child) = children_stack.last() {
            if let Some(rule) = rule_stack.last() {
                if child.text_range().start() >= offset {
                    // NOTE: This child is behind the "cursor"
                    // -> Add remaining stack to solution
                    // -> if the stack is nullable, move up in the tree
                    while let Some(rule) = rule_stack.pop() {
                        result.append(&mut rule.first());
                        if !rule.is_nullable() {
                            return Some(result);
                        }
                    }
                    // NOTE: The stack was nullable, move up the tree
                    if let Some(grand_parent) = parent.parent() {
                        parent = grand_parent;
                        children_stack = parent.children_with_tokens().collect();
                        children_stack.reverse();
                        rule_stack.push(Rule::from_node_kind(parent.kind())?);
                        continue;
                    } else {
                        return Some(result);
                    }
                }
                if child.kind() == SyntaxKind::WHITESPACE {
                    children_stack.pop();
                } else if child.kind() == SyntaxKind::Error {
                    return Some(result);
                } else {
                    match &rule {
                        Rule::Node(syntax_kind) | Rule::Token(syntax_kind) => {
                            if *syntax_kind == child.kind() {
                                rule_stack.pop();
                                children_stack.pop();
                            } else if rule.is_nullable() {
                                rule_stack.pop();
                            } else {
                                return Some(result);
                            }
                            if children_stack.is_empty() && rule_stack.is_empty() {
                                // NOTE: Position could not be found in this rule, move up the tree
                                if let Some(grand_parent) = parent.parent() {
                                    parent = grand_parent;
                                    children_stack.extend(parent.children_with_tokens());
                                    children_stack.reverse();
                                    rule_stack.push(Rule::from_node_kind(parent.kind())?);
                                } else {
                                    return Some(result);
                                }
                            }
                        }
                        Rule::Seq(rules) => {
                            let x = rules.clone();
                            rule_stack.pop();
                            rule_stack.extend(x.into_iter().rev());
                        }
                        Rule::Alt(rules) => {
                            if let Some(rule) = rules
                                .iter()
                                .find(|rule| rule.first().iter().any(|kind| *kind == child.kind()))
                            {
                                let x = rule.clone();
                                rule_stack.pop();
                                rule_stack.push(x);
                            } else if rules.iter().any(|rule| rule.is_nullable()) {
                                rule_stack.pop();
                            } else {
                                return None;
                            }
                        }
                        Rule::Opt(rule) => {
                            let x = *rule.clone();
                            rule_stack.pop();
                            if x.first().contains(&child.kind()) {
                                rule_stack.push(x);
                            }
                        }
                        Rule::Rep(rule) => {
                            if rule.first().iter().any(|kind| *kind == child.kind()) {
                                rule_stack.push(*rule.clone());
                            } else {
                                rule_stack.pop();
                            }
                        }
                    }
                }
            } else {
                // NOTE: The rule stack is empty ->
                for rule in rule_stack.iter().rev() {
                    result.append(&mut rule.first());
                    if !rule.is_nullable() {
                        break;
                    }
                }
                return Some(result);
            }
        } else {
            // NOTE: The childrens stack is emtpy
            // -> Add remaining stack to solution
            // -> if the stack is nullable, move up in the tree
            while let Some(rule) = rule_stack.pop() {
                result.append(&mut rule.first());
                if !rule.is_nullable() {
                    return Some(result);
                }
            }
            if let Some(grand_parent) = parent.parent() {
                parent = grand_parent;
                children_stack.extend(parent.children_with_tokens());
                children_stack.reverse();
                rule_stack.push(Rule::from_node_kind(parent.kind())?);
            } else {
                return Some(result);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{parse_query, rules::Rule, syntax_kind::SyntaxKind};

    use super::continuations_at;

    #[test]
    fn nullablity() {
        assert!(Rule::from_node_kind(SyntaxKind::GroupGraphPatternSub)
            .unwrap()
            .is_nullable());
    }

    #[test]
    fn first() {
        assert_eq!(
            Rule::from_node_kind(SyntaxKind::GroupGraphPatternSub)
                .unwrap()
                .first(),
            vec![SyntaxKind::TriplesBlock, SyntaxKind::GraphPatternNotTriples]
        );
    }

    #[test]
    fn continueations_failure() {
        //           0123456789012345678
        let input = "SELECT WHERE { }";
        let root = parse_query(input);
        assert_eq!(continuations_at(&root, 12.into()), None);
    }

    #[test]
    fn continueations_triplesblock() {
        //           012345678901234567890123456
        let input = "SELECT WHERE { ?a ?b ?c . }";
        let root = parse_query(input);
        assert_eq!(
            continuations_at(&root, 25.into()),
            vec![
                SyntaxKind::TriplesBlock,
                SyntaxKind::GraphPatternNotTriples,
                SyntaxKind::RCurly
            ]
            .into()
        );
    }

    #[test]
    fn continueations_where_clause() {
        //           0123456789012345678
        let input = "SELECT * WHERE { }";
        let root = parse_query(input);
        assert_eq!(
            continuations_at(&root, 14.into()),
            vec![SyntaxKind::GroupGraphPattern].into()
        );
    }

    #[test]
    fn continueations_triple() {
        //           01234567890123456789012345678901234
        let input = "SELECT * FROM <> WHERE { ?a ?s ?c }  ";
        let root = parse_query(input);
        assert_eq!(
            continuations_at(&root, 24.into()),
            vec![
                SyntaxKind::SubSelect,
                SyntaxKind::GroupGraphPatternSub,
                SyntaxKind::RCurly
            ]
            .into()
        );
        // assert_eq!(
        //     continuations_at(&root, 27.into()),
        //     vec![SyntaxKind::PropertyListPathNotEmpty].into()
        // );
        // assert_eq!(
        //     continuations_at(&root, 30.into()),
        //     vec![SyntaxKind::ObjectListPath].into()
        // );
    }

    #[test]
    fn continueations_select_query() {
        //           012345678901234567890123456789
        let input = "SELECT * FROM <> WHERE {}\n";
        let root = parse_query(input);
        assert_eq!(
            continuations_at(&root, 8.into()),
            vec![SyntaxKind::DatasetClause, SyntaxKind::WhereClause].into()
        );
        assert_eq!(
            continuations_at(&root, 16.into()),
            vec![SyntaxKind::DatasetClause, SyntaxKind::WhereClause].into()
        );
        assert_eq!(
            continuations_at(&root, 25.into()),
            vec![SyntaxKind::SolutionModifier, SyntaxKind::ValuesClause].into()
        );
        // Maybe this would be reasonable?:
        // assert_eq!(
        //     continuations_at(&root, 27.into()),
        //     vec![SyntaxKind::SolutionModifier, SyntaxKind::ValuesClause].into()
        // );
    }

    #[test]
    fn continueations_values_clause() {
        //           0123456789012345678901234567890
        let input = "SELECT * WHERE { VALUES ?a {}}";
        let root = parse_query(input);
        assert_eq!(
            continuations_at(&root, 27.into()),
            vec![SyntaxKind::LCurly].into()
        );
        assert_eq!(
            continuations_at(&root, 28.into()),
            vec![SyntaxKind::DataBlockValue, SyntaxKind::RCurly].into()
        );
    }
}
