#![allow(non_snake_case)]
use super::Parser;
use crate::SyntaxKind;
/// [0] QueryUnit -> Query
pub(super) fn parse_QueryUnit(p: &mut Parser) {
    let marker = p.open();
    parse_Query(p);
    if !p.at(SyntaxKind::Eof) {
        let error_marker = p.open();
        while !p.at(SyntaxKind::Eof) {
            p.advance();
        }
        p.close(error_marker, SyntaxKind::Error);
    }
    p.close(marker, SyntaxKind::QueryUnit);
}
/// [1] Query -> Prologue (SelectQuery | ConstructQuery | DescribeQuery | AskQuery) ValuesClause
pub(super) fn parse_Query(p: &mut Parser) {
    let marker = p.open();
    parse_Prologue(p);
    match p.nth(0) {
        SyntaxKind::SELECT => {
            parse_SelectQuery(p);
        }
        SyntaxKind::CONSTRUCT => {
            parse_ConstructQuery(p);
        }
        SyntaxKind::DESCRIBE => {
            parse_DescribeQuery(p);
        }
        SyntaxKind::ASK => {
            parse_AskQuery(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::Query);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::SELECT,
                SyntaxKind::CONSTRUCT,
                SyntaxKind::DESCRIBE,
                SyntaxKind::ASK,
            ]);
        }
    };
    parse_ValuesClause(p);
    p.close(marker, SyntaxKind::Query);
}
/// [2] Prologue -> (BaseDecl | PrefixDecl | VersionDecl)*
pub(super) fn parse_Prologue(p: &mut Parser) {
    if !p.at_any(&[SyntaxKind::BASE, SyntaxKind::PREFIX, SyntaxKind::VERSION]) {
        return;
    }
    let marker = p.open();
    while [SyntaxKind::BASE, SyntaxKind::PREFIX, SyntaxKind::VERSION].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        match p.nth(0) {
            SyntaxKind::BASE => {
                parse_BaseDecl(p);
            }
            SyntaxKind::PREFIX => {
                parse_PrefixDecl(p);
            }
            SyntaxKind::VERSION => {
                parse_VersionDecl(p);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::Prologue);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![
                    SyntaxKind::BASE,
                    SyntaxKind::PREFIX,
                    SyntaxKind::VERSION,
                ]);
            }
        };
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::Prologue);
}
/// [3] SelectQuery -> SelectClause DatasetClause* WhereClause SolutionModifier
pub(super) fn parse_SelectQuery(p: &mut Parser) {
    let marker = p.open();
    parse_SelectClause(p);
    while [SyntaxKind::FROM].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        parse_DatasetClause(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    if !p.at_any(&[SyntaxKind::WHERE, SyntaxKind::LCurly]) && !p.eof() {
        p.error_until(
            vec![SyntaxKind::WHERE, SyntaxKind::LCurly],
            &[SyntaxKind::WHERE, SyntaxKind::LCurly],
        );
    }
    parse_WhereClause(p);
    parse_SolutionModifier(p);
    p.close(marker, SyntaxKind::SelectQuery);
}
/// [4] ConstructQuery -> 'CONSTRUCT' (ConstructTemplate DatasetClause* WhereClause SolutionModifier | DatasetClause* 'WHERE' '{' TriplesTemplate? '}' SolutionModifier)
pub(super) fn parse_ConstructQuery(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::CONSTRUCT);
    if !p.at_any(&[SyntaxKind::WHERE, SyntaxKind::LCurly, SyntaxKind::FROM]) && !p.eof() {
        p.error_until(
            vec![SyntaxKind::WHERE, SyntaxKind::LCurly, SyntaxKind::FROM],
            &[SyntaxKind::WHERE, SyntaxKind::LCurly, SyntaxKind::FROM],
        );
    }
    match p.nth(0) {
        SyntaxKind::LCurly => {
            parse_ConstructTemplate(p);
            while [SyntaxKind::FROM].contains(&p.nth(0)) {
                let checkpoint = p.pos();
                parse_DatasetClause(p);
                if p.pos() == checkpoint {
                    break;
                }
            }
            if !p.at_any(&[SyntaxKind::WHERE, SyntaxKind::LCurly]) && !p.eof() {
                p.error_until(
                    vec![SyntaxKind::WHERE, SyntaxKind::LCurly],
                    &[SyntaxKind::WHERE, SyntaxKind::LCurly],
                );
            }
            parse_WhereClause(p);
            parse_SolutionModifier(p);
        }
        SyntaxKind::WHERE | SyntaxKind::FROM => {
            while [SyntaxKind::FROM].contains(&p.nth(0)) {
                let checkpoint = p.pos();
                parse_DatasetClause(p);
                if p.pos() == checkpoint {
                    break;
                }
            }
            if !p.at_any(&[SyntaxKind::WHERE]) && !p.eof() {
                p.error_until(vec![SyntaxKind::WHERE], &[SyntaxKind::WHERE]);
            }
            p.expect(SyntaxKind::WHERE);
            if !p.at_any(&[SyntaxKind::LCurly]) && !p.eof() {
                p.error_until(vec![SyntaxKind::LCurly], &[SyntaxKind::LCurly]);
            }
            p.expect(SyntaxKind::LCurly);
            if p.at_any(&[
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::LParen,
                SyntaxKind::INTEGER,
                SyntaxKind::NIL,
                SyntaxKind::LBrack,
                SyntaxKind::DoubleLess,
                SyntaxKind::DoubleLessLParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
            ]) {
                parse_TriplesTemplate(p);
            }
            if !p.at_any(&[SyntaxKind::RCurly]) && !p.eof() {
                p.error_until(vec![SyntaxKind::RCurly], &[SyntaxKind::RCurly]);
            }
            p.expect(SyntaxKind::RCurly);
            parse_SolutionModifier(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ConstructQuery);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::LCurly,
                SyntaxKind::WHERE,
                SyntaxKind::FROM,
            ]);
        }
    };
    p.close(marker, SyntaxKind::ConstructQuery);
}
/// [5] DescribeQuery -> 'DESCRIBE' (VarOrIri VarOrIri* | '*') DatasetClause* WhereClause? SolutionModifier
pub(super) fn parse_DescribeQuery(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DESCRIBE);
    if !p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::Star,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::PNAME_LN,
    ]) && !p.eof()
    {
        p.error_until(
            vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::Star,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::PNAME_LN,
            ],
            &[
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::Star,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::PNAME_LN,
            ],
        );
    }
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::PNAME_LN => {
            parse_VarOrIri(p);
            while [
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::PNAME_LN,
            ]
            .contains(&p.nth(0))
            {
                let checkpoint = p.pos();
                parse_VarOrIri(p);
                if p.pos() == checkpoint {
                    break;
                }
            }
        }
        SyntaxKind::Star => {
            p.expect(SyntaxKind::Star);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::DescribeQuery);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::Star,
            ]);
        }
    };
    while [SyntaxKind::FROM].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        parse_DatasetClause(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    if p.at_any(&[SyntaxKind::WHERE, SyntaxKind::LCurly]) {
        parse_WhereClause(p);
    }
    parse_SolutionModifier(p);
    p.close(marker, SyntaxKind::DescribeQuery);
}
/// [6] AskQuery -> 'ASK' DatasetClause* WhereClause SolutionModifier
pub(super) fn parse_AskQuery(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::ASK);
    while [SyntaxKind::FROM].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        parse_DatasetClause(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    if !p.at_any(&[SyntaxKind::WHERE, SyntaxKind::LCurly]) && !p.eof() {
        p.error_until(
            vec![SyntaxKind::WHERE, SyntaxKind::LCurly],
            &[SyntaxKind::WHERE, SyntaxKind::LCurly],
        );
    }
    parse_WhereClause(p);
    parse_SolutionModifier(p);
    p.close(marker, SyntaxKind::AskQuery);
}
/// [7] ValuesClause -> ('VALUES' DataBlock)?
pub(super) fn parse_ValuesClause(p: &mut Parser) {
    if !p.at_any(&[SyntaxKind::VALUES]) {
        return;
    }
    let marker = p.open();
    if p.at_any(&[SyntaxKind::VALUES]) {
        p.expect(SyntaxKind::VALUES);
        parse_DataBlock(p);
    }
    p.close(marker, SyntaxKind::ValuesClause);
}
/// [8] UpdateUnit -> Update
pub(super) fn parse_UpdateUnit(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::BASE,
        SyntaxKind::PREFIX,
        SyntaxKind::VERSION,
        SyntaxKind::LOAD,
        SyntaxKind::CLEAR,
        SyntaxKind::DROP,
        SyntaxKind::CREATE,
        SyntaxKind::ADD,
        SyntaxKind::MOVE,
        SyntaxKind::COPY,
        SyntaxKind::INSERT_DATA,
        SyntaxKind::DELETE_DATA,
        SyntaxKind::DELETE_WHERE,
        SyntaxKind::WITH,
        SyntaxKind::DELETE,
        SyntaxKind::INSERT,
    ]) {
        return;
    }
    let marker = p.open();
    parse_Update(p);
    if !p.at(SyntaxKind::Eof) {
        let error_marker = p.open();
        while !p.at(SyntaxKind::Eof) {
            p.advance();
        }
        p.close(error_marker, SyntaxKind::Error);
    }
    p.close(marker, SyntaxKind::UpdateUnit);
}
/// [9] Update -> Prologue (UpdateOne (';' Update)?)?
pub(super) fn parse_Update(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::BASE,
        SyntaxKind::PREFIX,
        SyntaxKind::VERSION,
        SyntaxKind::LOAD,
        SyntaxKind::CLEAR,
        SyntaxKind::DROP,
        SyntaxKind::CREATE,
        SyntaxKind::ADD,
        SyntaxKind::MOVE,
        SyntaxKind::COPY,
        SyntaxKind::INSERT_DATA,
        SyntaxKind::DELETE_DATA,
        SyntaxKind::DELETE_WHERE,
        SyntaxKind::WITH,
        SyntaxKind::DELETE,
        SyntaxKind::INSERT,
    ]) {
        return;
    }
    let marker = p.open();
    parse_Prologue(p);
    if p.at_any(&[
        SyntaxKind::LOAD,
        SyntaxKind::CLEAR,
        SyntaxKind::DROP,
        SyntaxKind::CREATE,
        SyntaxKind::ADD,
        SyntaxKind::MOVE,
        SyntaxKind::COPY,
        SyntaxKind::INSERT_DATA,
        SyntaxKind::DELETE_DATA,
        SyntaxKind::DELETE_WHERE,
        SyntaxKind::WITH,
        SyntaxKind::DELETE,
        SyntaxKind::INSERT,
    ]) {
        parse_UpdateOne(p);
        if p.at_any(&[SyntaxKind::Semicolon]) {
            p.expect(SyntaxKind::Semicolon);
            parse_Update(p);
        }
    }
    p.close(marker, SyntaxKind::Update);
}
/// [10] BaseDecl -> 'BASE' 'IRIREF'
pub(super) fn parse_BaseDecl(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::BASE);
    p.expect(SyntaxKind::IRIREF);
    p.close(marker, SyntaxKind::BaseDecl);
}
/// [11] PrefixDecl -> 'PREFIX' 'PNAME_NS' 'IRIREF'
pub(super) fn parse_PrefixDecl(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::PREFIX);
    p.expect(SyntaxKind::PNAME_NS);
    p.expect(SyntaxKind::IRIREF);
    p.close(marker, SyntaxKind::PrefixDecl);
}
/// [12] VersionDecl -> 'VERSION' VersionSpecifier
pub(super) fn parse_VersionDecl(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::VERSION);
    parse_VersionSpecifier(p);
    p.close(marker, SyntaxKind::VersionDecl);
}
/// [13] VersionSpecifier -> 'STRING_LITERAL1' | 'STRING_LITERAL2'
pub(super) fn parse_VersionSpecifier(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::STRING_LITERAL1 => {
            p.expect(SyntaxKind::STRING_LITERAL1);
        }
        SyntaxKind::STRING_LITERAL2 => {
            p.expect(SyntaxKind::STRING_LITERAL2);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::VersionSpecifier);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
            ]);
        }
    };
    p.close(marker, SyntaxKind::VersionSpecifier);
}
/// [14] SelectClause -> 'SELECT' ('DISTINCT' | 'REDUCED')? ((Var | '(' Expression 'AS' Var ')') (Var | '(' Expression 'AS' Var ')')* | '*')
pub(super) fn parse_SelectClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::SELECT);
    if p.at_any(&[SyntaxKind::DISTINCT, SyntaxKind::REDUCED]) {
        match p.nth(0) {
            SyntaxKind::DISTINCT => {
                p.expect(SyntaxKind::DISTINCT);
            }
            SyntaxKind::REDUCED => {
                p.expect(SyntaxKind::REDUCED);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::SelectClause);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![SyntaxKind::DISTINCT, SyntaxKind::REDUCED]);
            }
        };
    }
    match p.nth(0) {
        SyntaxKind::LParen | SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            match p.nth(0) {
                SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
                    parse_Var(p);
                }
                SyntaxKind::LParen => {
                    p.expect(SyntaxKind::LParen);
                    parse_Expression(p);
                    p.expect(SyntaxKind::AS);
                    parse_Var(p);
                    p.expect(SyntaxKind::RParen);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::SelectClause);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![
                        SyntaxKind::VAR1,
                        SyntaxKind::VAR2,
                        SyntaxKind::LParen,
                    ]);
                }
            };
            while [SyntaxKind::LParen, SyntaxKind::VAR1, SyntaxKind::VAR2].contains(&p.nth(0)) {
                let checkpoint = p.pos();
                match p.nth(0) {
                    SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
                        parse_Var(p);
                    }
                    SyntaxKind::LParen => {
                        p.expect(SyntaxKind::LParen);
                        parse_Expression(p);
                        p.expect(SyntaxKind::AS);
                        parse_Var(p);
                        p.expect(SyntaxKind::RParen);
                    }
                    SyntaxKind::Eof => {
                        p.close(marker, SyntaxKind::SelectClause);
                        let marker = p.open();
                        p.close(marker, SyntaxKind::Error);
                        return;
                    }
                    _ => {
                        p.advance_with_error(vec![
                            SyntaxKind::VAR1,
                            SyntaxKind::VAR2,
                            SyntaxKind::LParen,
                        ]);
                    }
                };
                if p.pos() == checkpoint {
                    break;
                }
            }
        }
        SyntaxKind::Star => {
            p.expect(SyntaxKind::Star);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::SelectClause);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::LParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::Star,
            ]);
        }
    };
    p.close(marker, SyntaxKind::SelectClause);
}
/// [15] DatasetClause -> 'FROM' (DefaultGraphClause | NamedGraphClause)
pub(super) fn parse_DatasetClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::FROM);
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_DefaultGraphClause(p);
        }
        SyntaxKind::NAMED => {
            parse_NamedGraphClause(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::DatasetClause);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::NAMED,
            ]);
        }
    };
    p.close(marker, SyntaxKind::DatasetClause);
}
/// [16] WhereClause -> 'WHERE'? GroupGraphPattern
pub(super) fn parse_WhereClause(p: &mut Parser) {
    let marker = p.open();
    if p.at_any(&[SyntaxKind::WHERE]) {
        p.expect(SyntaxKind::WHERE);
    }
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::WhereClause);
}
/// [17] SolutionModifier -> GroupClause? HavingClause? OrderClause? LimitOffsetClauses?
pub(super) fn parse_SolutionModifier(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::GROUP,
        SyntaxKind::HAVING,
        SyntaxKind::ORDER,
        SyntaxKind::LIMIT,
        SyntaxKind::OFFSET,
    ]) {
        return;
    }
    let marker = p.open();
    if p.at_any(&[SyntaxKind::GROUP]) {
        parse_GroupClause(p);
    }
    if p.at_any(&[SyntaxKind::HAVING]) {
        parse_HavingClause(p);
    }
    if p.at_any(&[SyntaxKind::ORDER]) {
        parse_OrderClause(p);
    }
    if p.at_any(&[SyntaxKind::LIMIT, SyntaxKind::OFFSET]) {
        parse_LimitOffsetClauses(p);
    }
    p.close(marker, SyntaxKind::SolutionModifier);
}
/// [18] SubSelect -> SelectClause WhereClause SolutionModifier ValuesClause
pub(super) fn parse_SubSelect(p: &mut Parser) {
    let marker = p.open();
    parse_SelectClause(p);
    parse_WhereClause(p);
    parse_SolutionModifier(p);
    parse_ValuesClause(p);
    p.close(marker, SyntaxKind::SubSelect);
}
/// [19] Var -> 'VAR1' | 'VAR2'
pub(super) fn parse_Var(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 => {
            p.expect(SyntaxKind::VAR1);
        }
        SyntaxKind::VAR2 => {
            p.expect(SyntaxKind::VAR2);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::Var);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::VAR1, SyntaxKind::VAR2]);
        }
    };
    p.close(marker, SyntaxKind::Var);
}
/// [20] Expression -> ConditionalOrExpression
pub(super) fn parse_Expression(p: &mut Parser) {
    let marker = p.open();
    parse_ConditionalOrExpression(p);
    p.close(marker, SyntaxKind::Expression);
}
/// [21] ConstructTemplate -> '{' ConstructTriples? '}'
pub(super) fn parse_ConstructTemplate(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LCurly);
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        parse_ConstructTriples(p);
    }
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::ConstructTemplate);
}
/// [22] TriplesTemplate -> TriplesSameSubject ('.' TriplesTemplate?)?
pub(super) fn parse_TriplesTemplate(p: &mut Parser) {
    let marker = p.open();
    parse_TriplesSameSubject(p);
    if p.at_any(&[SyntaxKind::Dot]) {
        p.expect(SyntaxKind::Dot);
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::STRING_LITERAL1,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LParen,
            SyntaxKind::INTEGER,
            SyntaxKind::NIL,
            SyntaxKind::LBrack,
            SyntaxKind::DoubleLess,
            SyntaxKind::DoubleLessLParen,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::DECIMAL,
            SyntaxKind::DOUBLE,
            SyntaxKind::INTEGER_POSITIVE,
            SyntaxKind::DECIMAL_POSITIVE,
            SyntaxKind::DOUBLE_POSITIVE,
            SyntaxKind::INTEGER_NEGATIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DOUBLE_NEGATIVE,
            SyntaxKind::True,
            SyntaxKind::False,
            SyntaxKind::STRING_LITERAL_LONG1,
            SyntaxKind::STRING_LITERAL_LONG2,
            SyntaxKind::PNAME_LN,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::ANON,
        ]) {
            parse_TriplesTemplate(p);
        }
    }
    p.close(marker, SyntaxKind::TriplesTemplate);
}
/// [23] VarOrIri -> Var | iri
pub(super) fn parse_VarOrIri(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::VarOrIri);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
            ]);
        }
    };
    p.close(marker, SyntaxKind::VarOrIri);
}
/// [24] DefaultGraphClause -> SourceSelector
pub(super) fn parse_DefaultGraphClause(p: &mut Parser) {
    let marker = p.open();
    parse_SourceSelector(p);
    p.close(marker, SyntaxKind::DefaultGraphClause);
}
/// [25] NamedGraphClause -> 'NAMED' SourceSelector
pub(super) fn parse_NamedGraphClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::NAMED);
    parse_SourceSelector(p);
    p.close(marker, SyntaxKind::NamedGraphClause);
}
/// [26] SourceSelector -> iri
pub(super) fn parse_SourceSelector(p: &mut Parser) {
    let marker = p.open();
    parse_iri(p);
    p.close(marker, SyntaxKind::SourceSelector);
}
/// [27] iri -> 'IRIREF' | PrefixedName
pub(super) fn parse_iri(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF => {
            p.expect(SyntaxKind::IRIREF);
        }
        SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_PrefixedName(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::iri);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
            ]);
        }
    };
    p.close(marker, SyntaxKind::iri);
}
/// [28] GroupGraphPattern -> '{' (SubSelect | GroupGraphPatternSub) '}'
pub(super) fn parse_GroupGraphPattern(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LCurly);
    match p.nth(0) {
        SyntaxKind::SELECT => {
            parse_SubSelect(p);
        }
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::LParen
        | SyntaxKind::LCurly
        | SyntaxKind::INTEGER
        | SyntaxKind::VALUES
        | SyntaxKind::GRAPH
        | SyntaxKind::OPTIONAL
        | SyntaxKind::SERVICE
        | SyntaxKind::BIND
        | SyntaxKind::NIL
        | SyntaxKind::MINUS
        | SyntaxKind::FILTER
        | SyntaxKind::LBrack
        | SyntaxKind::DoubleLess
        | SyntaxKind::DoubleLessLParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE
        | SyntaxKind::True
        | SyntaxKind::False
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2
        | SyntaxKind::PNAME_LN
        | SyntaxKind::BLANK_NODE_LABEL
        | SyntaxKind::ANON => {
            parse_GroupGraphPatternSub(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GroupGraphPattern);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {}
    };
    if !p.at_any(&[SyntaxKind::RCurly]) && !p.eof() {
        p.error_until(vec![SyntaxKind::RCurly], &[SyntaxKind::RCurly]);
    }
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::GroupGraphPattern);
}
/// [29] GroupClause -> 'GROUP' 'BY' GroupCondition GroupCondition*
pub(super) fn parse_GroupClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::GROUP);
    p.expect(SyntaxKind::BY);
    parse_GroupCondition(p);
    while [
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::LParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::NOT,
        SyntaxKind::STR,
        SyntaxKind::LANG,
        SyntaxKind::LANGMATCHES,
        SyntaxKind::LANGDIR,
        SyntaxKind::DATATYPE,
        SyntaxKind::BOUND,
        SyntaxKind::IRI,
        SyntaxKind::URI,
        SyntaxKind::BNODE,
        SyntaxKind::RAND,
        SyntaxKind::ABS,
        SyntaxKind::CEIL,
        SyntaxKind::FLOOR,
        SyntaxKind::ROUND,
        SyntaxKind::CONCAT,
        SyntaxKind::STRLEN,
        SyntaxKind::UCASE,
        SyntaxKind::LCASE,
        SyntaxKind::ENCODE_FOR_URI,
        SyntaxKind::CONTAINS,
        SyntaxKind::STRSTARTS,
        SyntaxKind::STRENDS,
        SyntaxKind::STRBEFORE,
        SyntaxKind::STRAFTER,
        SyntaxKind::YEAR,
        SyntaxKind::MONTH,
        SyntaxKind::DAY,
        SyntaxKind::HOURS,
        SyntaxKind::MINUTES,
        SyntaxKind::SECONDS,
        SyntaxKind::TIMEZONE,
        SyntaxKind::TZ,
        SyntaxKind::NOW,
        SyntaxKind::UUID,
        SyntaxKind::STRUUID,
        SyntaxKind::MD5,
        SyntaxKind::SHA1,
        SyntaxKind::SHA256,
        SyntaxKind::SHA384,
        SyntaxKind::SHA512,
        SyntaxKind::COALESCE,
        SyntaxKind::IF,
        SyntaxKind::STRLANG,
        SyntaxKind::STRLANGDIR,
        SyntaxKind::STRDT,
        SyntaxKind::sameTerm,
        SyntaxKind::isIRI,
        SyntaxKind::isURI,
        SyntaxKind::isBLANK,
        SyntaxKind::isLITERAL,
        SyntaxKind::isNUMERIC,
        SyntaxKind::hasLANG,
        SyntaxKind::hasLANGDIR,
        SyntaxKind::isTRIPLE,
        SyntaxKind::TRIPLE,
        SyntaxKind::SUBJECT,
        SyntaxKind::PREDICATE,
        SyntaxKind::OBJECT,
        SyntaxKind::REGEX,
        SyntaxKind::SUBSTR,
        SyntaxKind::REPLACE,
        SyntaxKind::EXISTS,
        SyntaxKind::COUNT,
        SyntaxKind::SUM,
        SyntaxKind::MIN,
        SyntaxKind::MAX,
        SyntaxKind::AVG,
        SyntaxKind::SAMPLE,
        SyntaxKind::GROUP_CONCAT,
        SyntaxKind::PNAME_LN,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_GroupCondition(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::GroupClause);
}
/// [30] HavingClause -> 'HAVING' HavingCondition HavingCondition*
pub(super) fn parse_HavingClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::HAVING);
    parse_HavingCondition(p);
    while [
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::LParen,
        SyntaxKind::NOT,
        SyntaxKind::STR,
        SyntaxKind::LANG,
        SyntaxKind::LANGMATCHES,
        SyntaxKind::LANGDIR,
        SyntaxKind::DATATYPE,
        SyntaxKind::BOUND,
        SyntaxKind::IRI,
        SyntaxKind::URI,
        SyntaxKind::BNODE,
        SyntaxKind::RAND,
        SyntaxKind::ABS,
        SyntaxKind::CEIL,
        SyntaxKind::FLOOR,
        SyntaxKind::ROUND,
        SyntaxKind::CONCAT,
        SyntaxKind::STRLEN,
        SyntaxKind::UCASE,
        SyntaxKind::LCASE,
        SyntaxKind::ENCODE_FOR_URI,
        SyntaxKind::CONTAINS,
        SyntaxKind::STRSTARTS,
        SyntaxKind::STRENDS,
        SyntaxKind::STRBEFORE,
        SyntaxKind::STRAFTER,
        SyntaxKind::YEAR,
        SyntaxKind::MONTH,
        SyntaxKind::DAY,
        SyntaxKind::HOURS,
        SyntaxKind::MINUTES,
        SyntaxKind::SECONDS,
        SyntaxKind::TIMEZONE,
        SyntaxKind::TZ,
        SyntaxKind::NOW,
        SyntaxKind::UUID,
        SyntaxKind::STRUUID,
        SyntaxKind::MD5,
        SyntaxKind::SHA1,
        SyntaxKind::SHA256,
        SyntaxKind::SHA384,
        SyntaxKind::SHA512,
        SyntaxKind::COALESCE,
        SyntaxKind::IF,
        SyntaxKind::STRLANG,
        SyntaxKind::STRLANGDIR,
        SyntaxKind::STRDT,
        SyntaxKind::sameTerm,
        SyntaxKind::isIRI,
        SyntaxKind::isURI,
        SyntaxKind::isBLANK,
        SyntaxKind::isLITERAL,
        SyntaxKind::isNUMERIC,
        SyntaxKind::hasLANG,
        SyntaxKind::hasLANGDIR,
        SyntaxKind::isTRIPLE,
        SyntaxKind::TRIPLE,
        SyntaxKind::SUBJECT,
        SyntaxKind::PREDICATE,
        SyntaxKind::OBJECT,
        SyntaxKind::REGEX,
        SyntaxKind::SUBSTR,
        SyntaxKind::REPLACE,
        SyntaxKind::EXISTS,
        SyntaxKind::COUNT,
        SyntaxKind::SUM,
        SyntaxKind::MIN,
        SyntaxKind::MAX,
        SyntaxKind::AVG,
        SyntaxKind::SAMPLE,
        SyntaxKind::GROUP_CONCAT,
        SyntaxKind::PNAME_LN,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_HavingCondition(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::HavingClause);
}
/// [31] OrderClause -> 'ORDER' 'BY' OrderCondition OrderCondition*
pub(super) fn parse_OrderClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::ORDER);
    p.expect(SyntaxKind::BY);
    parse_OrderCondition(p);
    while [
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::LParen,
        SyntaxKind::ASC,
        SyntaxKind::DESC,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::NOT,
        SyntaxKind::STR,
        SyntaxKind::LANG,
        SyntaxKind::LANGMATCHES,
        SyntaxKind::LANGDIR,
        SyntaxKind::DATATYPE,
        SyntaxKind::BOUND,
        SyntaxKind::IRI,
        SyntaxKind::URI,
        SyntaxKind::BNODE,
        SyntaxKind::RAND,
        SyntaxKind::ABS,
        SyntaxKind::CEIL,
        SyntaxKind::FLOOR,
        SyntaxKind::ROUND,
        SyntaxKind::CONCAT,
        SyntaxKind::STRLEN,
        SyntaxKind::UCASE,
        SyntaxKind::LCASE,
        SyntaxKind::ENCODE_FOR_URI,
        SyntaxKind::CONTAINS,
        SyntaxKind::STRSTARTS,
        SyntaxKind::STRENDS,
        SyntaxKind::STRBEFORE,
        SyntaxKind::STRAFTER,
        SyntaxKind::YEAR,
        SyntaxKind::MONTH,
        SyntaxKind::DAY,
        SyntaxKind::HOURS,
        SyntaxKind::MINUTES,
        SyntaxKind::SECONDS,
        SyntaxKind::TIMEZONE,
        SyntaxKind::TZ,
        SyntaxKind::NOW,
        SyntaxKind::UUID,
        SyntaxKind::STRUUID,
        SyntaxKind::MD5,
        SyntaxKind::SHA1,
        SyntaxKind::SHA256,
        SyntaxKind::SHA384,
        SyntaxKind::SHA512,
        SyntaxKind::COALESCE,
        SyntaxKind::IF,
        SyntaxKind::STRLANG,
        SyntaxKind::STRLANGDIR,
        SyntaxKind::STRDT,
        SyntaxKind::sameTerm,
        SyntaxKind::isIRI,
        SyntaxKind::isURI,
        SyntaxKind::isBLANK,
        SyntaxKind::isLITERAL,
        SyntaxKind::isNUMERIC,
        SyntaxKind::hasLANG,
        SyntaxKind::hasLANGDIR,
        SyntaxKind::isTRIPLE,
        SyntaxKind::TRIPLE,
        SyntaxKind::SUBJECT,
        SyntaxKind::PREDICATE,
        SyntaxKind::OBJECT,
        SyntaxKind::REGEX,
        SyntaxKind::SUBSTR,
        SyntaxKind::REPLACE,
        SyntaxKind::EXISTS,
        SyntaxKind::COUNT,
        SyntaxKind::SUM,
        SyntaxKind::MIN,
        SyntaxKind::MAX,
        SyntaxKind::AVG,
        SyntaxKind::SAMPLE,
        SyntaxKind::GROUP_CONCAT,
        SyntaxKind::PNAME_LN,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_OrderCondition(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::OrderClause);
}
/// [32] LimitOffsetClauses -> LimitClause OffsetClause? | OffsetClause LimitClause?
pub(super) fn parse_LimitOffsetClauses(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LIMIT => {
            parse_LimitClause(p);
            if p.at_any(&[SyntaxKind::OFFSET]) {
                parse_OffsetClause(p);
            }
        }
        SyntaxKind::OFFSET => {
            parse_OffsetClause(p);
            if p.at_any(&[SyntaxKind::LIMIT]) {
                parse_LimitClause(p);
            }
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::LimitOffsetClauses);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::LIMIT, SyntaxKind::OFFSET]);
        }
    };
    p.close(marker, SyntaxKind::LimitOffsetClauses);
}
/// [33] GroupCondition -> BuiltInCall | FunctionCall | '(' Expression ('AS' Var)? ')' | Var
pub(super) fn parse_GroupCondition(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::NOT
        | SyntaxKind::STR
        | SyntaxKind::LANG
        | SyntaxKind::LANGMATCHES
        | SyntaxKind::LANGDIR
        | SyntaxKind::DATATYPE
        | SyntaxKind::BOUND
        | SyntaxKind::IRI
        | SyntaxKind::URI
        | SyntaxKind::BNODE
        | SyntaxKind::RAND
        | SyntaxKind::ABS
        | SyntaxKind::CEIL
        | SyntaxKind::FLOOR
        | SyntaxKind::ROUND
        | SyntaxKind::CONCAT
        | SyntaxKind::STRLEN
        | SyntaxKind::UCASE
        | SyntaxKind::LCASE
        | SyntaxKind::ENCODE_FOR_URI
        | SyntaxKind::CONTAINS
        | SyntaxKind::STRSTARTS
        | SyntaxKind::STRENDS
        | SyntaxKind::STRBEFORE
        | SyntaxKind::STRAFTER
        | SyntaxKind::YEAR
        | SyntaxKind::MONTH
        | SyntaxKind::DAY
        | SyntaxKind::HOURS
        | SyntaxKind::MINUTES
        | SyntaxKind::SECONDS
        | SyntaxKind::TIMEZONE
        | SyntaxKind::TZ
        | SyntaxKind::NOW
        | SyntaxKind::UUID
        | SyntaxKind::STRUUID
        | SyntaxKind::MD5
        | SyntaxKind::SHA1
        | SyntaxKind::SHA256
        | SyntaxKind::SHA384
        | SyntaxKind::SHA512
        | SyntaxKind::COALESCE
        | SyntaxKind::IF
        | SyntaxKind::STRLANG
        | SyntaxKind::STRLANGDIR
        | SyntaxKind::STRDT
        | SyntaxKind::sameTerm
        | SyntaxKind::isIRI
        | SyntaxKind::isURI
        | SyntaxKind::isBLANK
        | SyntaxKind::isLITERAL
        | SyntaxKind::isNUMERIC
        | SyntaxKind::hasLANG
        | SyntaxKind::hasLANGDIR
        | SyntaxKind::isTRIPLE
        | SyntaxKind::TRIPLE
        | SyntaxKind::SUBJECT
        | SyntaxKind::PREDICATE
        | SyntaxKind::OBJECT
        | SyntaxKind::REGEX
        | SyntaxKind::SUBSTR
        | SyntaxKind::REPLACE
        | SyntaxKind::EXISTS
        | SyntaxKind::COUNT
        | SyntaxKind::SUM
        | SyntaxKind::MIN
        | SyntaxKind::MAX
        | SyntaxKind::AVG
        | SyntaxKind::SAMPLE
        | SyntaxKind::GROUP_CONCAT => {
            parse_BuiltInCall(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_FunctionCall(p);
        }
        SyntaxKind::LParen => {
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            if p.at_any(&[SyntaxKind::AS]) {
                p.expect(SyntaxKind::AS);
                parse_Var(p);
            }
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GroupCondition);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::NOT,
                SyntaxKind::STR,
                SyntaxKind::LANG,
                SyntaxKind::LANGMATCHES,
                SyntaxKind::LANGDIR,
                SyntaxKind::DATATYPE,
                SyntaxKind::BOUND,
                SyntaxKind::IRI,
                SyntaxKind::URI,
                SyntaxKind::BNODE,
                SyntaxKind::RAND,
                SyntaxKind::ABS,
                SyntaxKind::CEIL,
                SyntaxKind::FLOOR,
                SyntaxKind::ROUND,
                SyntaxKind::CONCAT,
                SyntaxKind::STRLEN,
                SyntaxKind::UCASE,
                SyntaxKind::LCASE,
                SyntaxKind::ENCODE_FOR_URI,
                SyntaxKind::CONTAINS,
                SyntaxKind::STRSTARTS,
                SyntaxKind::STRENDS,
                SyntaxKind::STRBEFORE,
                SyntaxKind::STRAFTER,
                SyntaxKind::YEAR,
                SyntaxKind::MONTH,
                SyntaxKind::DAY,
                SyntaxKind::HOURS,
                SyntaxKind::MINUTES,
                SyntaxKind::SECONDS,
                SyntaxKind::TIMEZONE,
                SyntaxKind::TZ,
                SyntaxKind::NOW,
                SyntaxKind::UUID,
                SyntaxKind::STRUUID,
                SyntaxKind::MD5,
                SyntaxKind::SHA1,
                SyntaxKind::SHA256,
                SyntaxKind::SHA384,
                SyntaxKind::SHA512,
                SyntaxKind::COALESCE,
                SyntaxKind::IF,
                SyntaxKind::STRLANG,
                SyntaxKind::STRLANGDIR,
                SyntaxKind::STRDT,
                SyntaxKind::sameTerm,
                SyntaxKind::isIRI,
                SyntaxKind::isURI,
                SyntaxKind::isBLANK,
                SyntaxKind::isLITERAL,
                SyntaxKind::isNUMERIC,
                SyntaxKind::hasLANG,
                SyntaxKind::hasLANGDIR,
                SyntaxKind::isTRIPLE,
                SyntaxKind::TRIPLE,
                SyntaxKind::SUBJECT,
                SyntaxKind::PREDICATE,
                SyntaxKind::OBJECT,
                SyntaxKind::REGEX,
                SyntaxKind::SUBSTR,
                SyntaxKind::REPLACE,
                SyntaxKind::EXISTS,
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::LParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
            ]);
        }
    };
    p.close(marker, SyntaxKind::GroupCondition);
}
/// [34] BuiltInCall -> Aggregate | 'STR' '(' Expression ')' | 'LANG' '(' Expression ')' | 'LANGMATCHES' '(' Expression ',' Expression ')' | 'LANGDIR' '(' Expression ')' | 'DATATYPE' '(' Expression ')' | 'BOUND' '(' Var ')' | 'IRI' '(' Expression ')' | 'URI' '(' Expression ')' | 'BNODE' ('(' Expression ')' | 'NIL') | 'RAND' 'NIL' | 'ABS' '(' Expression ')' | 'CEIL' '(' Expression ')' | 'FLOOR' '(' Expression ')' | 'ROUND' '(' Expression ')' | 'CONCAT' ExpressionList | SubstringExpression | 'STRLEN' '(' Expression ')' | StrReplaceExpression | 'UCASE' '(' Expression ')' | 'LCASE' '(' Expression ')' | 'ENCODE_FOR_URI' '(' Expression ')' | 'CONTAINS' '(' Expression ',' Expression ')' | 'STRSTARTS' '(' Expression ',' Expression ')' | 'STRENDS' '(' Expression ',' Expression ')' | 'STRBEFORE' '(' Expression ',' Expression ')' | 'STRAFTER' '(' Expression ',' Expression ')' | 'YEAR' '(' Expression ')' | 'MONTH' '(' Expression ')' | 'DAY' '(' Expression ')' | 'HOURS' '(' Expression ')' | 'MINUTES' '(' Expression ')' | 'SECONDS' '(' Expression ')' | 'TIMEZONE' '(' Expression ')' | 'TZ' '(' Expression ')' | 'NOW' 'NIL' | 'UUID' 'NIL' | 'STRUUID' 'NIL' | 'MD5' '(' Expression ')' | 'SHA1' '(' Expression ')' | 'SHA256' '(' Expression ')' | 'SHA384' '(' Expression ')' | 'SHA512' '(' Expression ')' | 'COALESCE' ExpressionList | 'IF' '(' Expression ',' Expression ',' Expression ')' | 'STRLANG' '(' Expression ',' Expression ')' | 'STRLANGDIR' '(' Expression ',' Expression ',' Expression ')' | 'STRDT' '(' Expression ',' Expression ')' | 'sameTerm' '(' Expression ',' Expression ')' | 'isIRI' '(' Expression ')' | 'isURI' '(' Expression ')' | 'isBLANK' '(' Expression ')' | 'isLITERAL' '(' Expression ')' | 'isNUMERIC' '(' Expression ')' | 'hasLANG' '(' Expression ')' | 'hasLANGDIR' '(' Expression ')' | RegexExpression | ExistsFunc | NotExistsFunc | 'isTRIPLE' '(' Expression ')' | 'TRIPLE' '(' Expression ',' Expression ',' Expression ')' | 'SUBJECT' '(' Expression ')' | 'PREDICATE' '(' Expression ')' | 'OBJECT' '(' Expression ')'
pub(super) fn parse_BuiltInCall(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::COUNT
        | SyntaxKind::SUM
        | SyntaxKind::MIN
        | SyntaxKind::MAX
        | SyntaxKind::AVG
        | SyntaxKind::SAMPLE
        | SyntaxKind::GROUP_CONCAT => {
            parse_Aggregate(p);
        }
        SyntaxKind::STR => {
            p.expect(SyntaxKind::STR);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::LANG => {
            p.expect(SyntaxKind::LANG);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::LANGMATCHES => {
            p.expect(SyntaxKind::LANGMATCHES);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::LANGDIR => {
            p.expect(SyntaxKind::LANGDIR);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::DATATYPE => {
            p.expect(SyntaxKind::DATATYPE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::BOUND => {
            p.expect(SyntaxKind::BOUND);
            p.expect(SyntaxKind::LParen);
            parse_Var(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::IRI => {
            p.expect(SyntaxKind::IRI);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::URI => {
            p.expect(SyntaxKind::URI);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::BNODE => {
            p.expect(SyntaxKind::BNODE);
            match p.nth(0) {
                SyntaxKind::LParen => {
                    p.expect(SyntaxKind::LParen);
                    parse_Expression(p);
                    p.expect(SyntaxKind::RParen);
                }
                SyntaxKind::NIL => {
                    p.expect(SyntaxKind::NIL);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::BuiltInCall);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![SyntaxKind::LParen, SyntaxKind::NIL]);
                }
            };
        }
        SyntaxKind::RAND => {
            p.expect(SyntaxKind::RAND);
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::ABS => {
            p.expect(SyntaxKind::ABS);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::CEIL => {
            p.expect(SyntaxKind::CEIL);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::FLOOR => {
            p.expect(SyntaxKind::FLOOR);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::ROUND => {
            p.expect(SyntaxKind::ROUND);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::CONCAT => {
            p.expect(SyntaxKind::CONCAT);
            parse_ExpressionList(p);
        }
        SyntaxKind::SUBSTR => {
            parse_SubstringExpression(p);
        }
        SyntaxKind::STRLEN => {
            p.expect(SyntaxKind::STRLEN);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::REPLACE => {
            parse_StrReplaceExpression(p);
        }
        SyntaxKind::UCASE => {
            p.expect(SyntaxKind::UCASE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::LCASE => {
            p.expect(SyntaxKind::LCASE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::ENCODE_FOR_URI => {
            p.expect(SyntaxKind::ENCODE_FOR_URI);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::CONTAINS => {
            p.expect(SyntaxKind::CONTAINS);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRSTARTS => {
            p.expect(SyntaxKind::STRSTARTS);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRENDS => {
            p.expect(SyntaxKind::STRENDS);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRBEFORE => {
            p.expect(SyntaxKind::STRBEFORE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRAFTER => {
            p.expect(SyntaxKind::STRAFTER);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::YEAR => {
            p.expect(SyntaxKind::YEAR);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::MONTH => {
            p.expect(SyntaxKind::MONTH);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::DAY => {
            p.expect(SyntaxKind::DAY);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::HOURS => {
            p.expect(SyntaxKind::HOURS);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::MINUTES => {
            p.expect(SyntaxKind::MINUTES);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SECONDS => {
            p.expect(SyntaxKind::SECONDS);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::TIMEZONE => {
            p.expect(SyntaxKind::TIMEZONE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::TZ => {
            p.expect(SyntaxKind::TZ);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::NOW => {
            p.expect(SyntaxKind::NOW);
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::UUID => {
            p.expect(SyntaxKind::UUID);
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::STRUUID => {
            p.expect(SyntaxKind::STRUUID);
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::MD5 => {
            p.expect(SyntaxKind::MD5);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SHA1 => {
            p.expect(SyntaxKind::SHA1);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SHA256 => {
            p.expect(SyntaxKind::SHA256);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SHA384 => {
            p.expect(SyntaxKind::SHA384);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SHA512 => {
            p.expect(SyntaxKind::SHA512);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::COALESCE => {
            p.expect(SyntaxKind::COALESCE);
            parse_ExpressionList(p);
        }
        SyntaxKind::IF => {
            p.expect(SyntaxKind::IF);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRLANG => {
            p.expect(SyntaxKind::STRLANG);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRLANGDIR => {
            p.expect(SyntaxKind::STRLANGDIR);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::STRDT => {
            p.expect(SyntaxKind::STRDT);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::sameTerm => {
            p.expect(SyntaxKind::sameTerm);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::isIRI => {
            p.expect(SyntaxKind::isIRI);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::isURI => {
            p.expect(SyntaxKind::isURI);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::isBLANK => {
            p.expect(SyntaxKind::isBLANK);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::isLITERAL => {
            p.expect(SyntaxKind::isLITERAL);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::isNUMERIC => {
            p.expect(SyntaxKind::isNUMERIC);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::hasLANG => {
            p.expect(SyntaxKind::hasLANG);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::hasLANGDIR => {
            p.expect(SyntaxKind::hasLANGDIR);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::REGEX => {
            parse_RegexExpression(p);
        }
        SyntaxKind::EXISTS => {
            parse_ExistsFunc(p);
        }
        SyntaxKind::NOT => {
            parse_NotExistsFunc(p);
        }
        SyntaxKind::isTRIPLE => {
            p.expect(SyntaxKind::isTRIPLE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::TRIPLE => {
            p.expect(SyntaxKind::TRIPLE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::Comma);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SUBJECT => {
            p.expect(SyntaxKind::SUBJECT);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::PREDICATE => {
            p.expect(SyntaxKind::PREDICATE);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::OBJECT => {
            p.expect(SyntaxKind::OBJECT);
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::BuiltInCall);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
                SyntaxKind::STR,
                SyntaxKind::LANG,
                SyntaxKind::LANGMATCHES,
                SyntaxKind::LANGDIR,
                SyntaxKind::DATATYPE,
                SyntaxKind::BOUND,
                SyntaxKind::IRI,
                SyntaxKind::URI,
                SyntaxKind::BNODE,
                SyntaxKind::RAND,
                SyntaxKind::ABS,
                SyntaxKind::CEIL,
                SyntaxKind::FLOOR,
                SyntaxKind::ROUND,
                SyntaxKind::CONCAT,
                SyntaxKind::SUBSTR,
                SyntaxKind::STRLEN,
                SyntaxKind::REPLACE,
                SyntaxKind::UCASE,
                SyntaxKind::LCASE,
                SyntaxKind::ENCODE_FOR_URI,
                SyntaxKind::CONTAINS,
                SyntaxKind::STRSTARTS,
                SyntaxKind::STRENDS,
                SyntaxKind::STRBEFORE,
                SyntaxKind::STRAFTER,
                SyntaxKind::YEAR,
                SyntaxKind::MONTH,
                SyntaxKind::DAY,
                SyntaxKind::HOURS,
                SyntaxKind::MINUTES,
                SyntaxKind::SECONDS,
                SyntaxKind::TIMEZONE,
                SyntaxKind::TZ,
                SyntaxKind::NOW,
                SyntaxKind::UUID,
                SyntaxKind::STRUUID,
                SyntaxKind::MD5,
                SyntaxKind::SHA1,
                SyntaxKind::SHA256,
                SyntaxKind::SHA384,
                SyntaxKind::SHA512,
                SyntaxKind::COALESCE,
                SyntaxKind::IF,
                SyntaxKind::STRLANG,
                SyntaxKind::STRLANGDIR,
                SyntaxKind::STRDT,
                SyntaxKind::sameTerm,
                SyntaxKind::isIRI,
                SyntaxKind::isURI,
                SyntaxKind::isBLANK,
                SyntaxKind::isLITERAL,
                SyntaxKind::isNUMERIC,
                SyntaxKind::hasLANG,
                SyntaxKind::hasLANGDIR,
                SyntaxKind::REGEX,
                SyntaxKind::EXISTS,
                SyntaxKind::NOT,
                SyntaxKind::isTRIPLE,
                SyntaxKind::TRIPLE,
                SyntaxKind::SUBJECT,
                SyntaxKind::PREDICATE,
                SyntaxKind::OBJECT,
            ]);
        }
    };
    p.close(marker, SyntaxKind::BuiltInCall);
}
/// [35] FunctionCall -> iri ArgList
pub(super) fn parse_FunctionCall(p: &mut Parser) {
    let marker = p.open();
    parse_iri(p);
    parse_ArgList(p);
    p.close(marker, SyntaxKind::FunctionCall);
}
/// [36] HavingCondition -> Constraint
pub(super) fn parse_HavingCondition(p: &mut Parser) {
    let marker = p.open();
    parse_Constraint(p);
    p.close(marker, SyntaxKind::HavingCondition);
}
/// [37] Constraint -> BrackettedExpression | BuiltInCall | FunctionCall
pub(super) fn parse_Constraint(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LParen => {
            parse_BrackettedExpression(p);
        }
        SyntaxKind::NOT
        | SyntaxKind::STR
        | SyntaxKind::LANG
        | SyntaxKind::LANGMATCHES
        | SyntaxKind::LANGDIR
        | SyntaxKind::DATATYPE
        | SyntaxKind::BOUND
        | SyntaxKind::IRI
        | SyntaxKind::URI
        | SyntaxKind::BNODE
        | SyntaxKind::RAND
        | SyntaxKind::ABS
        | SyntaxKind::CEIL
        | SyntaxKind::FLOOR
        | SyntaxKind::ROUND
        | SyntaxKind::CONCAT
        | SyntaxKind::STRLEN
        | SyntaxKind::UCASE
        | SyntaxKind::LCASE
        | SyntaxKind::ENCODE_FOR_URI
        | SyntaxKind::CONTAINS
        | SyntaxKind::STRSTARTS
        | SyntaxKind::STRENDS
        | SyntaxKind::STRBEFORE
        | SyntaxKind::STRAFTER
        | SyntaxKind::YEAR
        | SyntaxKind::MONTH
        | SyntaxKind::DAY
        | SyntaxKind::HOURS
        | SyntaxKind::MINUTES
        | SyntaxKind::SECONDS
        | SyntaxKind::TIMEZONE
        | SyntaxKind::TZ
        | SyntaxKind::NOW
        | SyntaxKind::UUID
        | SyntaxKind::STRUUID
        | SyntaxKind::MD5
        | SyntaxKind::SHA1
        | SyntaxKind::SHA256
        | SyntaxKind::SHA384
        | SyntaxKind::SHA512
        | SyntaxKind::COALESCE
        | SyntaxKind::IF
        | SyntaxKind::STRLANG
        | SyntaxKind::STRLANGDIR
        | SyntaxKind::STRDT
        | SyntaxKind::sameTerm
        | SyntaxKind::isIRI
        | SyntaxKind::isURI
        | SyntaxKind::isBLANK
        | SyntaxKind::isLITERAL
        | SyntaxKind::isNUMERIC
        | SyntaxKind::hasLANG
        | SyntaxKind::hasLANGDIR
        | SyntaxKind::isTRIPLE
        | SyntaxKind::TRIPLE
        | SyntaxKind::SUBJECT
        | SyntaxKind::PREDICATE
        | SyntaxKind::OBJECT
        | SyntaxKind::REGEX
        | SyntaxKind::SUBSTR
        | SyntaxKind::REPLACE
        | SyntaxKind::EXISTS
        | SyntaxKind::COUNT
        | SyntaxKind::SUM
        | SyntaxKind::MIN
        | SyntaxKind::MAX
        | SyntaxKind::AVG
        | SyntaxKind::SAMPLE
        | SyntaxKind::GROUP_CONCAT => {
            parse_BuiltInCall(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_FunctionCall(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::Constraint);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::LParen,
                SyntaxKind::NOT,
                SyntaxKind::STR,
                SyntaxKind::LANG,
                SyntaxKind::LANGMATCHES,
                SyntaxKind::LANGDIR,
                SyntaxKind::DATATYPE,
                SyntaxKind::BOUND,
                SyntaxKind::IRI,
                SyntaxKind::URI,
                SyntaxKind::BNODE,
                SyntaxKind::RAND,
                SyntaxKind::ABS,
                SyntaxKind::CEIL,
                SyntaxKind::FLOOR,
                SyntaxKind::ROUND,
                SyntaxKind::CONCAT,
                SyntaxKind::STRLEN,
                SyntaxKind::UCASE,
                SyntaxKind::LCASE,
                SyntaxKind::ENCODE_FOR_URI,
                SyntaxKind::CONTAINS,
                SyntaxKind::STRSTARTS,
                SyntaxKind::STRENDS,
                SyntaxKind::STRBEFORE,
                SyntaxKind::STRAFTER,
                SyntaxKind::YEAR,
                SyntaxKind::MONTH,
                SyntaxKind::DAY,
                SyntaxKind::HOURS,
                SyntaxKind::MINUTES,
                SyntaxKind::SECONDS,
                SyntaxKind::TIMEZONE,
                SyntaxKind::TZ,
                SyntaxKind::NOW,
                SyntaxKind::UUID,
                SyntaxKind::STRUUID,
                SyntaxKind::MD5,
                SyntaxKind::SHA1,
                SyntaxKind::SHA256,
                SyntaxKind::SHA384,
                SyntaxKind::SHA512,
                SyntaxKind::COALESCE,
                SyntaxKind::IF,
                SyntaxKind::STRLANG,
                SyntaxKind::STRLANGDIR,
                SyntaxKind::STRDT,
                SyntaxKind::sameTerm,
                SyntaxKind::isIRI,
                SyntaxKind::isURI,
                SyntaxKind::isBLANK,
                SyntaxKind::isLITERAL,
                SyntaxKind::isNUMERIC,
                SyntaxKind::hasLANG,
                SyntaxKind::hasLANGDIR,
                SyntaxKind::isTRIPLE,
                SyntaxKind::TRIPLE,
                SyntaxKind::SUBJECT,
                SyntaxKind::PREDICATE,
                SyntaxKind::OBJECT,
                SyntaxKind::REGEX,
                SyntaxKind::SUBSTR,
                SyntaxKind::REPLACE,
                SyntaxKind::EXISTS,
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
            ]);
        }
    };
    p.close(marker, SyntaxKind::Constraint);
}
/// [38] OrderCondition -> ('ASC' | 'DESC') BrackettedExpression | Constraint | Var
pub(super) fn parse_OrderCondition(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::ASC | SyntaxKind::DESC => {
            match p.nth(0) {
                SyntaxKind::ASC => {
                    p.expect(SyntaxKind::ASC);
                }
                SyntaxKind::DESC => {
                    p.expect(SyntaxKind::DESC);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::OrderCondition);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![SyntaxKind::ASC, SyntaxKind::DESC]);
                }
            };
            parse_BrackettedExpression(p);
        }
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::LParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::NOT
        | SyntaxKind::STR
        | SyntaxKind::LANG
        | SyntaxKind::LANGMATCHES
        | SyntaxKind::LANGDIR
        | SyntaxKind::DATATYPE
        | SyntaxKind::BOUND
        | SyntaxKind::IRI
        | SyntaxKind::URI
        | SyntaxKind::BNODE
        | SyntaxKind::RAND
        | SyntaxKind::ABS
        | SyntaxKind::CEIL
        | SyntaxKind::FLOOR
        | SyntaxKind::ROUND
        | SyntaxKind::CONCAT
        | SyntaxKind::STRLEN
        | SyntaxKind::UCASE
        | SyntaxKind::LCASE
        | SyntaxKind::ENCODE_FOR_URI
        | SyntaxKind::CONTAINS
        | SyntaxKind::STRSTARTS
        | SyntaxKind::STRENDS
        | SyntaxKind::STRBEFORE
        | SyntaxKind::STRAFTER
        | SyntaxKind::YEAR
        | SyntaxKind::MONTH
        | SyntaxKind::DAY
        | SyntaxKind::HOURS
        | SyntaxKind::MINUTES
        | SyntaxKind::SECONDS
        | SyntaxKind::TIMEZONE
        | SyntaxKind::TZ
        | SyntaxKind::NOW
        | SyntaxKind::UUID
        | SyntaxKind::STRUUID
        | SyntaxKind::MD5
        | SyntaxKind::SHA1
        | SyntaxKind::SHA256
        | SyntaxKind::SHA384
        | SyntaxKind::SHA512
        | SyntaxKind::COALESCE
        | SyntaxKind::IF
        | SyntaxKind::STRLANG
        | SyntaxKind::STRLANGDIR
        | SyntaxKind::STRDT
        | SyntaxKind::sameTerm
        | SyntaxKind::isIRI
        | SyntaxKind::isURI
        | SyntaxKind::isBLANK
        | SyntaxKind::isLITERAL
        | SyntaxKind::isNUMERIC
        | SyntaxKind::hasLANG
        | SyntaxKind::hasLANGDIR
        | SyntaxKind::isTRIPLE
        | SyntaxKind::TRIPLE
        | SyntaxKind::SUBJECT
        | SyntaxKind::PREDICATE
        | SyntaxKind::OBJECT
        | SyntaxKind::REGEX
        | SyntaxKind::SUBSTR
        | SyntaxKind::REPLACE
        | SyntaxKind::EXISTS
        | SyntaxKind::COUNT
        | SyntaxKind::SUM
        | SyntaxKind::MIN
        | SyntaxKind::MAX
        | SyntaxKind::AVG
        | SyntaxKind::SAMPLE
        | SyntaxKind::GROUP_CONCAT
        | SyntaxKind::PNAME_LN => {
            match p.nth(0) {
                SyntaxKind::IRIREF
                | SyntaxKind::PNAME_NS
                | SyntaxKind::LParen
                | SyntaxKind::NOT
                | SyntaxKind::STR
                | SyntaxKind::LANG
                | SyntaxKind::LANGMATCHES
                | SyntaxKind::LANGDIR
                | SyntaxKind::DATATYPE
                | SyntaxKind::BOUND
                | SyntaxKind::IRI
                | SyntaxKind::URI
                | SyntaxKind::BNODE
                | SyntaxKind::RAND
                | SyntaxKind::ABS
                | SyntaxKind::CEIL
                | SyntaxKind::FLOOR
                | SyntaxKind::ROUND
                | SyntaxKind::CONCAT
                | SyntaxKind::STRLEN
                | SyntaxKind::UCASE
                | SyntaxKind::LCASE
                | SyntaxKind::ENCODE_FOR_URI
                | SyntaxKind::CONTAINS
                | SyntaxKind::STRSTARTS
                | SyntaxKind::STRENDS
                | SyntaxKind::STRBEFORE
                | SyntaxKind::STRAFTER
                | SyntaxKind::YEAR
                | SyntaxKind::MONTH
                | SyntaxKind::DAY
                | SyntaxKind::HOURS
                | SyntaxKind::MINUTES
                | SyntaxKind::SECONDS
                | SyntaxKind::TIMEZONE
                | SyntaxKind::TZ
                | SyntaxKind::NOW
                | SyntaxKind::UUID
                | SyntaxKind::STRUUID
                | SyntaxKind::MD5
                | SyntaxKind::SHA1
                | SyntaxKind::SHA256
                | SyntaxKind::SHA384
                | SyntaxKind::SHA512
                | SyntaxKind::COALESCE
                | SyntaxKind::IF
                | SyntaxKind::STRLANG
                | SyntaxKind::STRLANGDIR
                | SyntaxKind::STRDT
                | SyntaxKind::sameTerm
                | SyntaxKind::isIRI
                | SyntaxKind::isURI
                | SyntaxKind::isBLANK
                | SyntaxKind::isLITERAL
                | SyntaxKind::isNUMERIC
                | SyntaxKind::hasLANG
                | SyntaxKind::hasLANGDIR
                | SyntaxKind::isTRIPLE
                | SyntaxKind::TRIPLE
                | SyntaxKind::SUBJECT
                | SyntaxKind::PREDICATE
                | SyntaxKind::OBJECT
                | SyntaxKind::REGEX
                | SyntaxKind::SUBSTR
                | SyntaxKind::REPLACE
                | SyntaxKind::EXISTS
                | SyntaxKind::COUNT
                | SyntaxKind::SUM
                | SyntaxKind::MIN
                | SyntaxKind::MAX
                | SyntaxKind::AVG
                | SyntaxKind::SAMPLE
                | SyntaxKind::GROUP_CONCAT
                | SyntaxKind::PNAME_LN => {
                    parse_Constraint(p);
                }
                SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
                    parse_Var(p);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::OrderCondition);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![
                        SyntaxKind::IRIREF,
                        SyntaxKind::PNAME_NS,
                        SyntaxKind::LParen,
                        SyntaxKind::NOT,
                        SyntaxKind::STR,
                        SyntaxKind::LANG,
                        SyntaxKind::LANGMATCHES,
                        SyntaxKind::LANGDIR,
                        SyntaxKind::DATATYPE,
                        SyntaxKind::BOUND,
                        SyntaxKind::IRI,
                        SyntaxKind::URI,
                        SyntaxKind::BNODE,
                        SyntaxKind::RAND,
                        SyntaxKind::ABS,
                        SyntaxKind::CEIL,
                        SyntaxKind::FLOOR,
                        SyntaxKind::ROUND,
                        SyntaxKind::CONCAT,
                        SyntaxKind::STRLEN,
                        SyntaxKind::UCASE,
                        SyntaxKind::LCASE,
                        SyntaxKind::ENCODE_FOR_URI,
                        SyntaxKind::CONTAINS,
                        SyntaxKind::STRSTARTS,
                        SyntaxKind::STRENDS,
                        SyntaxKind::STRBEFORE,
                        SyntaxKind::STRAFTER,
                        SyntaxKind::YEAR,
                        SyntaxKind::MONTH,
                        SyntaxKind::DAY,
                        SyntaxKind::HOURS,
                        SyntaxKind::MINUTES,
                        SyntaxKind::SECONDS,
                        SyntaxKind::TIMEZONE,
                        SyntaxKind::TZ,
                        SyntaxKind::NOW,
                        SyntaxKind::UUID,
                        SyntaxKind::STRUUID,
                        SyntaxKind::MD5,
                        SyntaxKind::SHA1,
                        SyntaxKind::SHA256,
                        SyntaxKind::SHA384,
                        SyntaxKind::SHA512,
                        SyntaxKind::COALESCE,
                        SyntaxKind::IF,
                        SyntaxKind::STRLANG,
                        SyntaxKind::STRLANGDIR,
                        SyntaxKind::STRDT,
                        SyntaxKind::sameTerm,
                        SyntaxKind::isIRI,
                        SyntaxKind::isURI,
                        SyntaxKind::isBLANK,
                        SyntaxKind::isLITERAL,
                        SyntaxKind::isNUMERIC,
                        SyntaxKind::hasLANG,
                        SyntaxKind::hasLANGDIR,
                        SyntaxKind::isTRIPLE,
                        SyntaxKind::TRIPLE,
                        SyntaxKind::SUBJECT,
                        SyntaxKind::PREDICATE,
                        SyntaxKind::OBJECT,
                        SyntaxKind::REGEX,
                        SyntaxKind::SUBSTR,
                        SyntaxKind::REPLACE,
                        SyntaxKind::EXISTS,
                        SyntaxKind::COUNT,
                        SyntaxKind::SUM,
                        SyntaxKind::MIN,
                        SyntaxKind::MAX,
                        SyntaxKind::AVG,
                        SyntaxKind::SAMPLE,
                        SyntaxKind::GROUP_CONCAT,
                        SyntaxKind::PNAME_LN,
                        SyntaxKind::VAR1,
                        SyntaxKind::VAR2,
                    ]);
                }
            };
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::OrderCondition);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::ASC,
                SyntaxKind::DESC,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::LParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::NOT,
                SyntaxKind::STR,
                SyntaxKind::LANG,
                SyntaxKind::LANGMATCHES,
                SyntaxKind::LANGDIR,
                SyntaxKind::DATATYPE,
                SyntaxKind::BOUND,
                SyntaxKind::IRI,
                SyntaxKind::URI,
                SyntaxKind::BNODE,
                SyntaxKind::RAND,
                SyntaxKind::ABS,
                SyntaxKind::CEIL,
                SyntaxKind::FLOOR,
                SyntaxKind::ROUND,
                SyntaxKind::CONCAT,
                SyntaxKind::STRLEN,
                SyntaxKind::UCASE,
                SyntaxKind::LCASE,
                SyntaxKind::ENCODE_FOR_URI,
                SyntaxKind::CONTAINS,
                SyntaxKind::STRSTARTS,
                SyntaxKind::STRENDS,
                SyntaxKind::STRBEFORE,
                SyntaxKind::STRAFTER,
                SyntaxKind::YEAR,
                SyntaxKind::MONTH,
                SyntaxKind::DAY,
                SyntaxKind::HOURS,
                SyntaxKind::MINUTES,
                SyntaxKind::SECONDS,
                SyntaxKind::TIMEZONE,
                SyntaxKind::TZ,
                SyntaxKind::NOW,
                SyntaxKind::UUID,
                SyntaxKind::STRUUID,
                SyntaxKind::MD5,
                SyntaxKind::SHA1,
                SyntaxKind::SHA256,
                SyntaxKind::SHA384,
                SyntaxKind::SHA512,
                SyntaxKind::COALESCE,
                SyntaxKind::IF,
                SyntaxKind::STRLANG,
                SyntaxKind::STRLANGDIR,
                SyntaxKind::STRDT,
                SyntaxKind::sameTerm,
                SyntaxKind::isIRI,
                SyntaxKind::isURI,
                SyntaxKind::isBLANK,
                SyntaxKind::isLITERAL,
                SyntaxKind::isNUMERIC,
                SyntaxKind::hasLANG,
                SyntaxKind::hasLANGDIR,
                SyntaxKind::isTRIPLE,
                SyntaxKind::TRIPLE,
                SyntaxKind::SUBJECT,
                SyntaxKind::PREDICATE,
                SyntaxKind::OBJECT,
                SyntaxKind::REGEX,
                SyntaxKind::SUBSTR,
                SyntaxKind::REPLACE,
                SyntaxKind::EXISTS,
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
                SyntaxKind::PNAME_LN,
            ]);
        }
    };
    p.close(marker, SyntaxKind::OrderCondition);
}
/// [39] BrackettedExpression -> '(' Expression ')'
pub(super) fn parse_BrackettedExpression(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LParen);
    parse_Expression(p);
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::BrackettedExpression);
}
/// [40] LimitClause -> 'LIMIT' 'INTEGER'
pub(super) fn parse_LimitClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LIMIT);
    p.expect(SyntaxKind::INTEGER);
    p.close(marker, SyntaxKind::LimitClause);
}
/// [41] OffsetClause -> 'OFFSET' 'INTEGER'
pub(super) fn parse_OffsetClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::OFFSET);
    p.expect(SyntaxKind::INTEGER);
    p.close(marker, SyntaxKind::OffsetClause);
}
/// [42] DataBlock -> InlineDataOneVar | InlineDataFull
pub(super) fn parse_DataBlock(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_InlineDataOneVar(p);
        }
        SyntaxKind::LParen | SyntaxKind::NIL => {
            parse_InlineDataFull(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::DataBlock);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::LParen,
                SyntaxKind::NIL,
            ]);
        }
    };
    p.close(marker, SyntaxKind::DataBlock);
}
/// [43] UpdateOne -> Load | Clear | Drop | Add | Move | Copy | Create | DeleteWhere | Modify | InsertData | DeleteData
pub(super) fn parse_UpdateOne(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LOAD => {
            parse_Load(p);
        }
        SyntaxKind::CLEAR => {
            parse_Clear(p);
        }
        SyntaxKind::DROP => {
            parse_Drop(p);
        }
        SyntaxKind::ADD => {
            parse_Add(p);
        }
        SyntaxKind::MOVE => {
            parse_Move(p);
        }
        SyntaxKind::COPY => {
            parse_Copy(p);
        }
        SyntaxKind::CREATE => {
            parse_Create(p);
        }
        SyntaxKind::DELETE_WHERE => {
            parse_DeleteWhere(p);
        }
        SyntaxKind::WITH | SyntaxKind::DELETE | SyntaxKind::INSERT => {
            parse_Modify(p);
        }
        SyntaxKind::INSERT_DATA => {
            parse_InsertData(p);
        }
        SyntaxKind::DELETE_DATA => {
            parse_DeleteData(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::UpdateOne);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::LOAD,
                SyntaxKind::CLEAR,
                SyntaxKind::DROP,
                SyntaxKind::ADD,
                SyntaxKind::MOVE,
                SyntaxKind::COPY,
                SyntaxKind::CREATE,
                SyntaxKind::DELETE_WHERE,
                SyntaxKind::WITH,
                SyntaxKind::DELETE,
                SyntaxKind::INSERT,
                SyntaxKind::INSERT_DATA,
                SyntaxKind::DELETE_DATA,
            ]);
        }
    };
    p.close(marker, SyntaxKind::UpdateOne);
}
/// [44] Load -> 'LOAD' 'SILENT'? iri ('INTO' GraphRef)?
pub(super) fn parse_Load(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LOAD);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_iri(p);
    if p.at_any(&[SyntaxKind::INTO]) {
        p.expect(SyntaxKind::INTO);
        parse_GraphRef(p);
    }
    p.close(marker, SyntaxKind::Load);
}
/// [45] Clear -> 'CLEAR' 'SILENT'? GraphRefAll
pub(super) fn parse_Clear(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::CLEAR);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_GraphRefAll(p);
    p.close(marker, SyntaxKind::Clear);
}
/// [46] Drop -> 'DROP' 'SILENT'? GraphRefAll
pub(super) fn parse_Drop(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DROP);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_GraphRefAll(p);
    p.close(marker, SyntaxKind::Drop);
}
/// [47] Add -> 'ADD' 'SILENT'? GraphOrDefault 'TO' GraphOrDefault
pub(super) fn parse_Add(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::ADD);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_GraphOrDefault(p);
    p.expect(SyntaxKind::TO);
    parse_GraphOrDefault(p);
    p.close(marker, SyntaxKind::Add);
}
/// [48] Move -> 'MOVE' 'SILENT'? GraphOrDefault 'TO' GraphOrDefault
pub(super) fn parse_Move(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::MOVE);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_GraphOrDefault(p);
    p.expect(SyntaxKind::TO);
    parse_GraphOrDefault(p);
    p.close(marker, SyntaxKind::Move);
}
/// [49] Copy -> 'COPY' 'SILENT'? GraphOrDefault 'TO' GraphOrDefault
pub(super) fn parse_Copy(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::COPY);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_GraphOrDefault(p);
    p.expect(SyntaxKind::TO);
    parse_GraphOrDefault(p);
    p.close(marker, SyntaxKind::Copy);
}
/// [50] Create -> 'CREATE' 'SILENT'? GraphRef
pub(super) fn parse_Create(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::CREATE);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_GraphRef(p);
    p.close(marker, SyntaxKind::Create);
}
/// [51] DeleteWhere -> 'DELETE_WHERE' QuadPattern
pub(super) fn parse_DeleteWhere(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DELETE_WHERE);
    parse_QuadPattern(p);
    p.close(marker, SyntaxKind::DeleteWhere);
}
/// [52] Modify -> ('WITH' iri)? (DeleteClause InsertClause? | InsertClause) UsingClause* 'WHERE' GroupGraphPattern
pub(super) fn parse_Modify(p: &mut Parser) {
    let marker = p.open();
    if p.at_any(&[SyntaxKind::WITH]) {
        p.expect(SyntaxKind::WITH);
        parse_iri(p);
    }
    match p.nth(0) {
        SyntaxKind::DELETE => {
            parse_DeleteClause(p);
            if p.at_any(&[SyntaxKind::INSERT]) {
                parse_InsertClause(p);
            }
        }
        SyntaxKind::INSERT => {
            parse_InsertClause(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::Modify);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::DELETE, SyntaxKind::INSERT]);
        }
    };
    while [SyntaxKind::USING].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        parse_UsingClause(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.expect(SyntaxKind::WHERE);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::Modify);
}
/// [53] InsertData -> 'INSERT_DATA' QuadData
pub(super) fn parse_InsertData(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::INSERT_DATA);
    parse_QuadData(p);
    p.close(marker, SyntaxKind::InsertData);
}
/// [54] DeleteData -> 'DELETE_DATA' QuadData
pub(super) fn parse_DeleteData(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DELETE_DATA);
    parse_QuadData(p);
    p.close(marker, SyntaxKind::DeleteData);
}
/// [55] GraphRef -> 'GRAPH' iri
pub(super) fn parse_GraphRef(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::GRAPH);
    parse_iri(p);
    p.close(marker, SyntaxKind::GraphRef);
}
/// [56] GraphRefAll -> GraphRef | 'DEFAULT' | 'NAMED' | 'ALL'
pub(super) fn parse_GraphRefAll(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::GRAPH => {
            parse_GraphRef(p);
        }
        SyntaxKind::DEFAULT => {
            p.expect(SyntaxKind::DEFAULT);
        }
        SyntaxKind::NAMED => {
            p.expect(SyntaxKind::NAMED);
        }
        SyntaxKind::ALL => {
            p.expect(SyntaxKind::ALL);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GraphRefAll);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::GRAPH,
                SyntaxKind::DEFAULT,
                SyntaxKind::NAMED,
                SyntaxKind::ALL,
            ]);
        }
    };
    p.close(marker, SyntaxKind::GraphRefAll);
}
/// [57] GraphOrDefault -> 'DEFAULT' | 'GRAPH'? iri
pub(super) fn parse_GraphOrDefault(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::DEFAULT => {
            p.expect(SyntaxKind::DEFAULT);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::GRAPH | SyntaxKind::PNAME_LN => {
            if p.at_any(&[SyntaxKind::GRAPH]) {
                p.expect(SyntaxKind::GRAPH);
            }
            parse_iri(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GraphOrDefault);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::DEFAULT,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::GRAPH,
                SyntaxKind::PNAME_LN,
            ]);
        }
    };
    p.close(marker, SyntaxKind::GraphOrDefault);
}
/// [58] QuadData -> '{' Quads '}'
pub(super) fn parse_QuadData(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LCurly);
    parse_Quads(p);
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::QuadData);
}
/// [59] QuadPattern -> '{' Quads '}'
pub(super) fn parse_QuadPattern(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LCurly);
    parse_Quads(p);
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::QuadPattern);
}
/// [60] DeleteClause -> 'DELETE' QuadPattern
pub(super) fn parse_DeleteClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DELETE);
    parse_QuadPattern(p);
    p.close(marker, SyntaxKind::DeleteClause);
}
/// [61] InsertClause -> 'INSERT' QuadPattern
pub(super) fn parse_InsertClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::INSERT);
    parse_QuadPattern(p);
    p.close(marker, SyntaxKind::InsertClause);
}
/// [62] UsingClause -> 'USING' (iri | 'NAMED' iri)
pub(super) fn parse_UsingClause(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::USING);
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::NAMED => {
            p.expect(SyntaxKind::NAMED);
            parse_iri(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::UsingClause);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::NAMED,
            ]);
        }
    };
    p.close(marker, SyntaxKind::UsingClause);
}
/// [63] Quads -> TriplesTemplate? (QuadsNotTriples '.'? TriplesTemplate?)*
pub(super) fn parse_Quads(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::GRAPH,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        return;
    }
    let marker = p.open();
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        parse_TriplesTemplate(p);
    }
    while [SyntaxKind::GRAPH].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        parse_QuadsNotTriples(p);
        if p.at_any(&[SyntaxKind::Dot]) {
            p.expect(SyntaxKind::Dot);
        }
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::STRING_LITERAL1,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LParen,
            SyntaxKind::INTEGER,
            SyntaxKind::NIL,
            SyntaxKind::LBrack,
            SyntaxKind::DoubleLess,
            SyntaxKind::DoubleLessLParen,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::DECIMAL,
            SyntaxKind::DOUBLE,
            SyntaxKind::INTEGER_POSITIVE,
            SyntaxKind::DECIMAL_POSITIVE,
            SyntaxKind::DOUBLE_POSITIVE,
            SyntaxKind::INTEGER_NEGATIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DOUBLE_NEGATIVE,
            SyntaxKind::True,
            SyntaxKind::False,
            SyntaxKind::STRING_LITERAL_LONG1,
            SyntaxKind::STRING_LITERAL_LONG2,
            SyntaxKind::PNAME_LN,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::ANON,
        ]) {
            parse_TriplesTemplate(p);
        }
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::Quads);
}
/// [64] QuadsNotTriples -> 'GRAPH' VarOrIri '{' TriplesTemplate? '}'
pub(super) fn parse_QuadsNotTriples(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::GRAPH);
    parse_VarOrIri(p);
    p.expect(SyntaxKind::LCurly);
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        parse_TriplesTemplate(p);
    }
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::QuadsNotTriples);
}
/// [65] TriplesSameSubject -> VarOrTerm PropertyListNotEmpty | TriplesNode PropertyList | ReifiedTripleBlock
pub(super) fn parse_TriplesSameSubject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::INTEGER
        | SyntaxKind::NIL
        | SyntaxKind::DoubleLessLParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE
        | SyntaxKind::True
        | SyntaxKind::False
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2
        | SyntaxKind::PNAME_LN
        | SyntaxKind::BLANK_NODE_LABEL
        | SyntaxKind::ANON => {
            parse_VarOrTerm(p);
            parse_PropertyListNotEmpty(p);
        }
        SyntaxKind::LParen | SyntaxKind::LBrack => {
            parse_TriplesNode(p);
            parse_PropertyList(p);
        }
        SyntaxKind::DoubleLess => {
            parse_ReifiedTripleBlock(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TriplesSameSubject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::INTEGER,
                SyntaxKind::NIL,
                SyntaxKind::DoubleLessLParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::LParen,
                SyntaxKind::LBrack,
                SyntaxKind::DoubleLess,
            ]);
        }
    };
    p.close(marker, SyntaxKind::TriplesSameSubject);
}
/// [66] GroupGraphPatternSub -> TriplesBlock? (GraphPatternNotTriples '.'? TriplesBlock?)*
pub(super) fn parse_GroupGraphPatternSub(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::LCurly,
        SyntaxKind::INTEGER,
        SyntaxKind::VALUES,
        SyntaxKind::GRAPH,
        SyntaxKind::OPTIONAL,
        SyntaxKind::SERVICE,
        SyntaxKind::BIND,
        SyntaxKind::NIL,
        SyntaxKind::MINUS,
        SyntaxKind::FILTER,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        return;
    }
    let marker = p.open();
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        parse_TriplesBlock(p);
    }
    while [
        SyntaxKind::LCurly,
        SyntaxKind::VALUES,
        SyntaxKind::GRAPH,
        SyntaxKind::OPTIONAL,
        SyntaxKind::SERVICE,
        SyntaxKind::BIND,
        SyntaxKind::MINUS,
        SyntaxKind::FILTER,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_GraphPatternNotTriples(p);
        if p.at_any(&[SyntaxKind::Dot]) {
            p.expect(SyntaxKind::Dot);
        }
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::STRING_LITERAL1,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LParen,
            SyntaxKind::INTEGER,
            SyntaxKind::NIL,
            SyntaxKind::LBrack,
            SyntaxKind::DoubleLess,
            SyntaxKind::DoubleLessLParen,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::DECIMAL,
            SyntaxKind::DOUBLE,
            SyntaxKind::INTEGER_POSITIVE,
            SyntaxKind::DECIMAL_POSITIVE,
            SyntaxKind::DOUBLE_POSITIVE,
            SyntaxKind::INTEGER_NEGATIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DOUBLE_NEGATIVE,
            SyntaxKind::True,
            SyntaxKind::False,
            SyntaxKind::STRING_LITERAL_LONG1,
            SyntaxKind::STRING_LITERAL_LONG2,
            SyntaxKind::PNAME_LN,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::ANON,
        ]) {
            parse_TriplesBlock(p);
        }
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::GroupGraphPatternSub);
}
/// [67] TriplesBlock -> TriplesSameSubjectPath ('.' TriplesBlock?)?
pub(super) fn parse_TriplesBlock(p: &mut Parser) {
    let marker = p.open();
    parse_TriplesSameSubjectPath(p);
    if p.at_any(&[SyntaxKind::Dot]) {
        p.expect(SyntaxKind::Dot);
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::STRING_LITERAL1,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LParen,
            SyntaxKind::INTEGER,
            SyntaxKind::NIL,
            SyntaxKind::LBrack,
            SyntaxKind::DoubleLess,
            SyntaxKind::DoubleLessLParen,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::DECIMAL,
            SyntaxKind::DOUBLE,
            SyntaxKind::INTEGER_POSITIVE,
            SyntaxKind::DECIMAL_POSITIVE,
            SyntaxKind::DOUBLE_POSITIVE,
            SyntaxKind::INTEGER_NEGATIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DOUBLE_NEGATIVE,
            SyntaxKind::True,
            SyntaxKind::False,
            SyntaxKind::STRING_LITERAL_LONG1,
            SyntaxKind::STRING_LITERAL_LONG2,
            SyntaxKind::PNAME_LN,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::ANON,
        ]) {
            parse_TriplesBlock(p);
        }
    }
    p.close(marker, SyntaxKind::TriplesBlock);
}
/// [68] GraphPatternNotTriples -> GroupOrUnionGraphPattern | OptionalGraphPattern | MinusGraphPattern | GraphGraphPattern | ServiceGraphPattern | Filter | Bind | InlineData
pub(super) fn parse_GraphPatternNotTriples(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LCurly => {
            parse_GroupOrUnionGraphPattern(p);
        }
        SyntaxKind::OPTIONAL => {
            parse_OptionalGraphPattern(p);
        }
        SyntaxKind::MINUS => {
            parse_MinusGraphPattern(p);
        }
        SyntaxKind::GRAPH => {
            parse_GraphGraphPattern(p);
        }
        SyntaxKind::SERVICE => {
            parse_ServiceGraphPattern(p);
        }
        SyntaxKind::FILTER => {
            parse_Filter(p);
        }
        SyntaxKind::BIND => {
            parse_Bind(p);
        }
        SyntaxKind::VALUES => {
            parse_InlineData(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GraphPatternNotTriples);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::LCurly,
                SyntaxKind::OPTIONAL,
                SyntaxKind::MINUS,
                SyntaxKind::GRAPH,
                SyntaxKind::SERVICE,
                SyntaxKind::FILTER,
                SyntaxKind::BIND,
                SyntaxKind::VALUES,
            ]);
        }
    };
    p.close(marker, SyntaxKind::GraphPatternNotTriples);
}
/// [69] TriplesSameSubjectPath -> VarOrTerm PropertyListPathNotEmpty | TriplesNodePath PropertyListPath | ReifiedTripleBlockPath
pub(super) fn parse_TriplesSameSubjectPath(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::INTEGER
        | SyntaxKind::NIL
        | SyntaxKind::DoubleLessLParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE
        | SyntaxKind::True
        | SyntaxKind::False
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2
        | SyntaxKind::PNAME_LN
        | SyntaxKind::BLANK_NODE_LABEL
        | SyntaxKind::ANON => {
            parse_VarOrTerm(p);
            parse_PropertyListPathNotEmpty(p);
        }
        SyntaxKind::LParen | SyntaxKind::LBrack => {
            parse_TriplesNodePath(p);
            parse_PropertyListPath(p);
        }
        SyntaxKind::DoubleLess => {
            parse_ReifiedTripleBlockPath(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TriplesSameSubjectPath);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::INTEGER,
                SyntaxKind::NIL,
                SyntaxKind::DoubleLessLParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::LParen,
                SyntaxKind::LBrack,
                SyntaxKind::DoubleLess,
            ]);
        }
    };
    p.close(marker, SyntaxKind::TriplesSameSubjectPath);
}
/// [70] ReifiedTripleBlock -> ReifiedTriple PropertyList
pub(super) fn parse_ReifiedTripleBlock(p: &mut Parser) {
    let marker = p.open();
    parse_ReifiedTriple(p);
    parse_PropertyList(p);
    p.close(marker, SyntaxKind::ReifiedTripleBlock);
}
/// [71] ReifiedTriple -> '<<' ReifiedTripleSubject Verb ReifiedTripleObject Reifier? '>>'
pub(super) fn parse_ReifiedTriple(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DoubleLess);
    parse_ReifiedTripleSubject(p);
    parse_Verb(p);
    parse_ReifiedTripleObject(p);
    if p.at_any(&[SyntaxKind::Tilde]) {
        parse_Reifier(p);
    }
    p.expect(SyntaxKind::DoubleMore);
    p.close(marker, SyntaxKind::ReifiedTriple);
}
/// [72] PropertyList -> PropertyListNotEmpty?
pub(super) fn parse_PropertyList(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::a,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::PNAME_LN,
    ]) {
        return;
    }
    let marker = p.open();
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::a,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::PNAME_LN,
    ]) {
        parse_PropertyListNotEmpty(p);
    }
    p.close(marker, SyntaxKind::PropertyList);
}
/// [73] ReifiedTripleBlockPath -> ReifiedTriple PropertyListPath
pub(super) fn parse_ReifiedTripleBlockPath(p: &mut Parser) {
    let marker = p.open();
    parse_ReifiedTriple(p);
    parse_PropertyListPath(p);
    p.close(marker, SyntaxKind::ReifiedTripleBlockPath);
}
/// [74] PropertyListPath -> PropertyListPathNotEmpty?
pub(super) fn parse_PropertyListPath(p: &mut Parser) {
    if !p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::LParen,
        SyntaxKind::a,
        SyntaxKind::Zirkumflex,
        SyntaxKind::ExclamationMark,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::PNAME_LN,
    ]) {
        return;
    }
    let marker = p.open();
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::LParen,
        SyntaxKind::a,
        SyntaxKind::Zirkumflex,
        SyntaxKind::ExclamationMark,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::PNAME_LN,
    ]) {
        parse_PropertyListPathNotEmpty(p);
    }
    p.close(marker, SyntaxKind::PropertyListPath);
}
/// [75] GroupOrUnionGraphPattern -> GroupGraphPattern ('UNION' GroupGraphPattern)*
pub(super) fn parse_GroupOrUnionGraphPattern(p: &mut Parser) {
    let marker = p.open();
    parse_GroupGraphPattern(p);
    while [SyntaxKind::UNION].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::UNION);
        parse_GroupGraphPattern(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::GroupOrUnionGraphPattern);
}
/// [76] OptionalGraphPattern -> 'OPTIONAL' GroupGraphPattern
pub(super) fn parse_OptionalGraphPattern(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::OPTIONAL);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::OptionalGraphPattern);
}
/// [77] MinusGraphPattern -> 'MINUS' GroupGraphPattern
pub(super) fn parse_MinusGraphPattern(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::MINUS);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::MinusGraphPattern);
}
/// [78] GraphGraphPattern -> 'GRAPH' VarOrIri GroupGraphPattern
pub(super) fn parse_GraphGraphPattern(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::GRAPH);
    parse_VarOrIri(p);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::GraphGraphPattern);
}
/// [79] ServiceGraphPattern -> 'SERVICE' 'SILENT'? VarOrIri GroupGraphPattern
pub(super) fn parse_ServiceGraphPattern(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::SERVICE);
    if p.at_any(&[SyntaxKind::SILENT]) {
        p.expect(SyntaxKind::SILENT);
    }
    parse_VarOrIri(p);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::ServiceGraphPattern);
}
/// [80] Filter -> 'FILTER' Constraint
pub(super) fn parse_Filter(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::FILTER);
    parse_Constraint(p);
    p.close(marker, SyntaxKind::Filter);
}
/// [81] Bind -> 'BIND' '(' Expression 'AS' Var ')'
pub(super) fn parse_Bind(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::BIND);
    p.expect(SyntaxKind::LParen);
    parse_Expression(p);
    p.expect(SyntaxKind::AS);
    parse_Var(p);
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::Bind);
}
/// [82] InlineData -> 'VALUES' DataBlock
pub(super) fn parse_InlineData(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::VALUES);
    parse_DataBlock(p);
    p.close(marker, SyntaxKind::InlineData);
}
/// [83] InlineDataOneVar -> Var '{' DataBlockValue* '}'
pub(super) fn parse_InlineDataOneVar(p: &mut Parser) {
    let marker = p.open();
    parse_Var(p);
    p.expect(SyntaxKind::LCurly);
    while [
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::INTEGER,
        SyntaxKind::UNDEF,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_DataBlockValue(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::InlineDataOneVar);
}
/// [84] InlineDataFull -> ('NIL' | '(' Var* ')') '{' ('(' DataBlockValue* ')' | 'NIL')* '}'
pub(super) fn parse_InlineDataFull(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::NIL => {
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::LParen => {
            p.expect(SyntaxKind::LParen);
            while [SyntaxKind::VAR1, SyntaxKind::VAR2].contains(&p.nth(0)) {
                let checkpoint = p.pos();
                parse_Var(p);
                if p.pos() == checkpoint {
                    break;
                }
            }
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::InlineDataFull);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::NIL, SyntaxKind::LParen]);
        }
    };
    p.expect(SyntaxKind::LCurly);
    while [SyntaxKind::LParen, SyntaxKind::NIL].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        match p.nth(0) {
            SyntaxKind::LParen => {
                p.expect(SyntaxKind::LParen);
                while [
                    SyntaxKind::IRIREF,
                    SyntaxKind::PNAME_NS,
                    SyntaxKind::STRING_LITERAL1,
                    SyntaxKind::STRING_LITERAL2,
                    SyntaxKind::INTEGER,
                    SyntaxKind::UNDEF,
                    SyntaxKind::DoubleLessLParen,
                    SyntaxKind::DECIMAL,
                    SyntaxKind::DOUBLE,
                    SyntaxKind::INTEGER_POSITIVE,
                    SyntaxKind::DECIMAL_POSITIVE,
                    SyntaxKind::DOUBLE_POSITIVE,
                    SyntaxKind::INTEGER_NEGATIVE,
                    SyntaxKind::DECIMAL_NEGATIVE,
                    SyntaxKind::DOUBLE_NEGATIVE,
                    SyntaxKind::True,
                    SyntaxKind::False,
                    SyntaxKind::STRING_LITERAL_LONG1,
                    SyntaxKind::STRING_LITERAL_LONG2,
                    SyntaxKind::PNAME_LN,
                ]
                .contains(&p.nth(0))
                {
                    let checkpoint = p.pos();
                    parse_DataBlockValue(p);
                    if p.pos() == checkpoint {
                        break;
                    }
                }
                p.expect(SyntaxKind::RParen);
            }
            SyntaxKind::NIL => {
                p.expect(SyntaxKind::NIL);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::InlineDataFull);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![SyntaxKind::LParen, SyntaxKind::NIL]);
            }
        };
        if p.pos() == checkpoint {
            break;
        }
    }
    p.expect(SyntaxKind::RCurly);
    p.close(marker, SyntaxKind::InlineDataFull);
}
/// [85] DataBlockValue -> iri | RDFLiteral | NumericLiteral | BooleanLiteral | 'UNDEF' | TripleTermData
pub(super) fn parse_DataBlockValue(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::UNDEF => {
            p.expect(SyntaxKind::UNDEF);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTermData(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::DataBlockValue);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::UNDEF,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::DataBlockValue);
}
/// [86] RDFLiteral -> String ('LANG_DIR' | '^^' iri)?
pub(super) fn parse_RDFLiteral(p: &mut Parser) {
    let marker = p.open();
    parse_String(p);
    if p.at_any(&[SyntaxKind::LANG_DIR, SyntaxKind::DoubleZirkumflex]) {
        match p.nth(0) {
            SyntaxKind::LANG_DIR => {
                p.expect(SyntaxKind::LANG_DIR);
            }
            SyntaxKind::DoubleZirkumflex => {
                p.expect(SyntaxKind::DoubleZirkumflex);
                parse_iri(p);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::RDFLiteral);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![SyntaxKind::LANG_DIR, SyntaxKind::DoubleZirkumflex]);
            }
        };
    }
    p.close(marker, SyntaxKind::RDFLiteral);
}
/// [87] NumericLiteral -> NumericLiteralUnsigned | NumericLiteralPositive | NumericLiteralNegative
pub(super) fn parse_NumericLiteral(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::INTEGER | SyntaxKind::DECIMAL | SyntaxKind::DOUBLE => {
            parse_NumericLiteralUnsigned(p);
        }
        SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE => {
            parse_NumericLiteralPositive(p);
        }
        SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteralNegative(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::NumericLiteral);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
            ]);
        }
    };
    p.close(marker, SyntaxKind::NumericLiteral);
}
/// [88] BooleanLiteral -> 'true' | 'false'
pub(super) fn parse_BooleanLiteral(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::True => {
            p.expect(SyntaxKind::True);
        }
        SyntaxKind::False => {
            p.expect(SyntaxKind::False);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::BooleanLiteral);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::True, SyntaxKind::False]);
        }
    };
    p.close(marker, SyntaxKind::BooleanLiteral);
}
/// [89] TripleTermData -> '<<(' TripleTermDataSubject (iri | 'a') TripleTermDataObject ')>>'
pub(super) fn parse_TripleTermData(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DoubleLessLParen);
    parse_TripleTermDataSubject(p);
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::a => {
            p.expect(SyntaxKind::a);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TripleTermData);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::a,
            ]);
        }
    };
    parse_TripleTermDataObject(p);
    p.expect(SyntaxKind::RParenDoubleMore);
    p.close(marker, SyntaxKind::TripleTermData);
}
/// [90] Reifier -> '~' VarOrReifierId?
pub(super) fn parse_Reifier(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::Tilde);
    if p.at_any(&[
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]) {
        parse_VarOrReifierId(p);
    }
    p.close(marker, SyntaxKind::Reifier);
}
/// [91] VarOrReifierId -> Var | iri | BlankNode
pub(super) fn parse_VarOrReifierId(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::BLANK_NODE_LABEL | SyntaxKind::ANON => {
            parse_BlankNode(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::VarOrReifierId);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
            ]);
        }
    };
    p.close(marker, SyntaxKind::VarOrReifierId);
}
/// [92] BlankNode -> 'BLANK_NODE_LABEL' | 'ANON'
pub(super) fn parse_BlankNode(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::BLANK_NODE_LABEL => {
            p.expect(SyntaxKind::BLANK_NODE_LABEL);
        }
        SyntaxKind::ANON => {
            p.expect(SyntaxKind::ANON);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::BlankNode);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::BLANK_NODE_LABEL, SyntaxKind::ANON]);
        }
    };
    p.close(marker, SyntaxKind::BlankNode);
}
/// [93] ArgList -> 'NIL' | '(' 'DISTINCT'? Expression (',' Expression)* ')'
pub(super) fn parse_ArgList(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::NIL => {
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::LParen => {
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            while [SyntaxKind::Comma].contains(&p.nth(0)) {
                let checkpoint = p.pos();
                p.expect(SyntaxKind::Comma);
                parse_Expression(p);
                if p.pos() == checkpoint {
                    break;
                }
            }
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ArgList);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::NIL, SyntaxKind::LParen]);
        }
    };
    p.close(marker, SyntaxKind::ArgList);
}
/// [94] ExpressionList -> 'NIL' | '(' Expression (',' Expression)* ')'
pub(super) fn parse_ExpressionList(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::NIL => {
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::LParen => {
            p.expect(SyntaxKind::LParen);
            parse_Expression(p);
            while [SyntaxKind::Comma].contains(&p.nth(0)) {
                let checkpoint = p.pos();
                p.expect(SyntaxKind::Comma);
                parse_Expression(p);
                if p.pos() == checkpoint {
                    break;
                }
            }
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ExpressionList);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::NIL, SyntaxKind::LParen]);
        }
    };
    p.close(marker, SyntaxKind::ExpressionList);
}
/// [95] ConstructTriples -> TriplesSameSubject ('.' ConstructTriples?)?
pub(super) fn parse_ConstructTriples(p: &mut Parser) {
    let marker = p.open();
    parse_TriplesSameSubject(p);
    if p.at_any(&[SyntaxKind::Dot]) {
        p.expect(SyntaxKind::Dot);
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::STRING_LITERAL1,
            SyntaxKind::STRING_LITERAL2,
            SyntaxKind::LParen,
            SyntaxKind::INTEGER,
            SyntaxKind::NIL,
            SyntaxKind::LBrack,
            SyntaxKind::DoubleLess,
            SyntaxKind::DoubleLessLParen,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::DECIMAL,
            SyntaxKind::DOUBLE,
            SyntaxKind::INTEGER_POSITIVE,
            SyntaxKind::DECIMAL_POSITIVE,
            SyntaxKind::DOUBLE_POSITIVE,
            SyntaxKind::INTEGER_NEGATIVE,
            SyntaxKind::DECIMAL_NEGATIVE,
            SyntaxKind::DOUBLE_NEGATIVE,
            SyntaxKind::True,
            SyntaxKind::False,
            SyntaxKind::STRING_LITERAL_LONG1,
            SyntaxKind::STRING_LITERAL_LONG2,
            SyntaxKind::PNAME_LN,
            SyntaxKind::BLANK_NODE_LABEL,
            SyntaxKind::ANON,
        ]) {
            parse_ConstructTriples(p);
        }
    }
    p.close(marker, SyntaxKind::ConstructTriples);
}
/// [96] VarOrTerm -> Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | 'NIL' | TripleTerm
pub(super) fn parse_VarOrTerm(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::BLANK_NODE_LABEL | SyntaxKind::ANON => {
            parse_BlankNode(p);
        }
        SyntaxKind::NIL => {
            p.expect(SyntaxKind::NIL);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::VarOrTerm);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::NIL,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::VarOrTerm);
}
/// [97] PropertyListNotEmpty -> Verb ObjectList (';' (Verb ObjectList)?)*
pub(super) fn parse_PropertyListNotEmpty(p: &mut Parser) {
    let marker = p.open();
    parse_Verb(p);
    parse_ObjectList(p);
    while [SyntaxKind::Semicolon].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::Semicolon);
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::a,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::PNAME_LN,
        ]) {
            parse_Verb(p);
            parse_ObjectList(p);
        }
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::PropertyListNotEmpty);
}
/// [98] TriplesNode -> Collection | BlankNodePropertyList
pub(super) fn parse_TriplesNode(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LParen => {
            parse_Collection(p);
        }
        SyntaxKind::LBrack => {
            parse_BlankNodePropertyList(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TriplesNode);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::LParen, SyntaxKind::LBrack]);
        }
    };
    p.close(marker, SyntaxKind::TriplesNode);
}
/// [99] Verb -> VarOrIri | 'a'
pub(super) fn parse_Verb(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::PNAME_LN => {
            parse_VarOrIri(p);
        }
        SyntaxKind::a => {
            p.expect(SyntaxKind::a);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::Verb);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::a,
            ]);
        }
    };
    p.close(marker, SyntaxKind::Verb);
}
/// [100] ObjectList -> Object (',' Object)*
pub(super) fn parse_ObjectList(p: &mut Parser) {
    let marker = p.open();
    parse_Object(p);
    while [SyntaxKind::Comma].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::Comma);
        parse_Object(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::ObjectList);
}
/// [101] Object -> GraphNode Annotation
pub(super) fn parse_Object(p: &mut Parser) {
    let marker = p.open();
    parse_GraphNode(p);
    parse_Annotation(p);
    p.close(marker, SyntaxKind::Object);
}
/// [102] GraphNode -> VarOrTerm | TriplesNode | ReifiedTriple
pub(super) fn parse_GraphNode(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::INTEGER
        | SyntaxKind::NIL
        | SyntaxKind::DoubleLessLParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE
        | SyntaxKind::True
        | SyntaxKind::False
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2
        | SyntaxKind::PNAME_LN
        | SyntaxKind::BLANK_NODE_LABEL
        | SyntaxKind::ANON => {
            parse_VarOrTerm(p);
        }
        SyntaxKind::LParen | SyntaxKind::LBrack => {
            parse_TriplesNode(p);
        }
        SyntaxKind::DoubleLess => {
            parse_ReifiedTriple(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GraphNode);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::INTEGER,
                SyntaxKind::NIL,
                SyntaxKind::DoubleLessLParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::LParen,
                SyntaxKind::LBrack,
                SyntaxKind::DoubleLess,
            ]);
        }
    };
    p.close(marker, SyntaxKind::GraphNode);
}
/// [103] Annotation -> (Reifier | AnnotationBlock)*
pub(super) fn parse_Annotation(p: &mut Parser) {
    if !p.at_any(&[SyntaxKind::Tilde, SyntaxKind::LCurlyPipe]) {
        return;
    }
    let marker = p.open();
    while [SyntaxKind::Tilde, SyntaxKind::LCurlyPipe].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        match p.nth(0) {
            SyntaxKind::Tilde => {
                parse_Reifier(p);
            }
            SyntaxKind::LCurlyPipe => {
                parse_AnnotationBlock(p);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::Annotation);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![SyntaxKind::Tilde, SyntaxKind::LCurlyPipe]);
            }
        };
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::Annotation);
}
/// [104] PropertyListPathNotEmpty -> (VerbPath | VerbSimple) ObjectListPath (';' ((VerbPath | VerbSimple) ObjectListPath)?)*
pub(super) fn parse_PropertyListPathNotEmpty(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::LParen
        | SyntaxKind::a
        | SyntaxKind::Zirkumflex
        | SyntaxKind::ExclamationMark
        | SyntaxKind::PNAME_LN => {
            parse_VerbPath(p);
        }
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_VerbSimple(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PropertyListPathNotEmpty);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::LParen,
                SyntaxKind::a,
                SyntaxKind::Zirkumflex,
                SyntaxKind::ExclamationMark,
                SyntaxKind::PNAME_LN,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
            ]);
        }
    };
    parse_ObjectListPath(p);
    while [SyntaxKind::Semicolon].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::Semicolon);
        if p.at_any(&[
            SyntaxKind::IRIREF,
            SyntaxKind::PNAME_NS,
            SyntaxKind::LParen,
            SyntaxKind::a,
            SyntaxKind::Zirkumflex,
            SyntaxKind::ExclamationMark,
            SyntaxKind::VAR1,
            SyntaxKind::VAR2,
            SyntaxKind::PNAME_LN,
        ]) {
            match p.nth(0) {
                SyntaxKind::IRIREF
                | SyntaxKind::PNAME_NS
                | SyntaxKind::LParen
                | SyntaxKind::a
                | SyntaxKind::Zirkumflex
                | SyntaxKind::ExclamationMark
                | SyntaxKind::PNAME_LN => {
                    parse_VerbPath(p);
                }
                SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
                    parse_VerbSimple(p);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::PropertyListPathNotEmpty);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![
                        SyntaxKind::IRIREF,
                        SyntaxKind::PNAME_NS,
                        SyntaxKind::LParen,
                        SyntaxKind::a,
                        SyntaxKind::Zirkumflex,
                        SyntaxKind::ExclamationMark,
                        SyntaxKind::PNAME_LN,
                        SyntaxKind::VAR1,
                        SyntaxKind::VAR2,
                    ]);
                }
            };
            parse_ObjectListPath(p);
        }
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::PropertyListPathNotEmpty);
}
/// [105] TriplesNodePath -> CollectionPath | BlankNodePropertyListPath
pub(super) fn parse_TriplesNodePath(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LParen => {
            parse_CollectionPath(p);
        }
        SyntaxKind::LBrack => {
            parse_BlankNodePropertyListPath(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TriplesNodePath);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::LParen, SyntaxKind::LBrack]);
        }
    };
    p.close(marker, SyntaxKind::TriplesNodePath);
}
/// [106] VerbPath -> Path
pub(super) fn parse_VerbPath(p: &mut Parser) {
    let marker = p.open();
    parse_Path(p);
    p.close(marker, SyntaxKind::VerbPath);
}
/// [107] VerbSimple -> Var
pub(super) fn parse_VerbSimple(p: &mut Parser) {
    let marker = p.open();
    parse_Var(p);
    p.close(marker, SyntaxKind::VerbSimple);
}
/// [108] ObjectListPath -> ObjectPath (',' ObjectPath)*
pub(super) fn parse_ObjectListPath(p: &mut Parser) {
    let marker = p.open();
    parse_ObjectPath(p);
    while [SyntaxKind::Comma].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::Comma);
        parse_ObjectPath(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::ObjectListPath);
}
/// [109] Path -> PathAlternative
pub(super) fn parse_Path(p: &mut Parser) {
    let marker = p.open();
    parse_PathAlternative(p);
    p.close(marker, SyntaxKind::Path);
}
/// [110] ObjectPath -> GraphNodePath AnnotationPath
pub(super) fn parse_ObjectPath(p: &mut Parser) {
    let marker = p.open();
    parse_GraphNodePath(p);
    parse_AnnotationPath(p);
    p.close(marker, SyntaxKind::ObjectPath);
}
/// [111] GraphNodePath -> VarOrTerm | TriplesNodePath | ReifiedTriple
pub(super) fn parse_GraphNodePath(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::INTEGER
        | SyntaxKind::NIL
        | SyntaxKind::DoubleLessLParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE
        | SyntaxKind::True
        | SyntaxKind::False
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2
        | SyntaxKind::PNAME_LN
        | SyntaxKind::BLANK_NODE_LABEL
        | SyntaxKind::ANON => {
            parse_VarOrTerm(p);
        }
        SyntaxKind::LParen | SyntaxKind::LBrack => {
            parse_TriplesNodePath(p);
        }
        SyntaxKind::DoubleLess => {
            parse_ReifiedTriple(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::GraphNodePath);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::INTEGER,
                SyntaxKind::NIL,
                SyntaxKind::DoubleLessLParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::PNAME_LN,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::LParen,
                SyntaxKind::LBrack,
                SyntaxKind::DoubleLess,
            ]);
        }
    };
    p.close(marker, SyntaxKind::GraphNodePath);
}
/// [112] AnnotationPath -> (Reifier | AnnotationBlockPath)*
pub(super) fn parse_AnnotationPath(p: &mut Parser) {
    if !p.at_any(&[SyntaxKind::Tilde, SyntaxKind::LCurlyPipe]) {
        return;
    }
    let marker = p.open();
    while [SyntaxKind::Tilde, SyntaxKind::LCurlyPipe].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        match p.nth(0) {
            SyntaxKind::Tilde => {
                parse_Reifier(p);
            }
            SyntaxKind::LCurlyPipe => {
                parse_AnnotationBlockPath(p);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::AnnotationPath);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![SyntaxKind::Tilde, SyntaxKind::LCurlyPipe]);
            }
        };
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::AnnotationPath);
}
/// [113] PathAlternative -> PathSequence ('|' PathSequence)*
pub(super) fn parse_PathAlternative(p: &mut Parser) {
    let marker = p.open();
    parse_PathSequence(p);
    while [SyntaxKind::Pipe].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::Pipe);
        parse_PathSequence(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::PathAlternative);
}
/// [114] PathSequence -> PathEltOrInverse ('/' PathEltOrInverse)*
pub(super) fn parse_PathSequence(p: &mut Parser) {
    let marker = p.open();
    parse_PathEltOrInverse(p);
    while [SyntaxKind::Slash].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::Slash);
        parse_PathEltOrInverse(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::PathSequence);
}
/// [115] PathEltOrInverse -> PathElt | '^' PathElt
pub(super) fn parse_PathEltOrInverse(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::LParen
        | SyntaxKind::a
        | SyntaxKind::ExclamationMark
        | SyntaxKind::PNAME_LN => {
            parse_PathElt(p);
        }
        SyntaxKind::Zirkumflex => {
            p.expect(SyntaxKind::Zirkumflex);
            parse_PathElt(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PathEltOrInverse);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::LParen,
                SyntaxKind::a,
                SyntaxKind::ExclamationMark,
                SyntaxKind::PNAME_LN,
                SyntaxKind::Zirkumflex,
            ]);
        }
    };
    p.close(marker, SyntaxKind::PathEltOrInverse);
}
/// [116] PathElt -> PathPrimary PathMod?
pub(super) fn parse_PathElt(p: &mut Parser) {
    let marker = p.open();
    parse_PathPrimary(p);
    if p.at_any(&[SyntaxKind::Star, SyntaxKind::QuestionMark, SyntaxKind::Plus]) {
        parse_PathMod(p);
    }
    p.close(marker, SyntaxKind::PathElt);
}
/// [117] PathPrimary -> iri | 'a' | '!' PathNegatedPropertySet | '(' Path ')'
pub(super) fn parse_PathPrimary(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::a => {
            p.expect(SyntaxKind::a);
        }
        SyntaxKind::ExclamationMark => {
            p.expect(SyntaxKind::ExclamationMark);
            parse_PathNegatedPropertySet(p);
        }
        SyntaxKind::LParen => {
            p.expect(SyntaxKind::LParen);
            parse_Path(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PathPrimary);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::a,
                SyntaxKind::ExclamationMark,
                SyntaxKind::LParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::PathPrimary);
}
/// [118] PathMod -> '?' | '*' | '+'
pub(super) fn parse_PathMod(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::QuestionMark => {
            p.expect(SyntaxKind::QuestionMark);
        }
        SyntaxKind::Star => {
            p.expect(SyntaxKind::Star);
        }
        SyntaxKind::Plus => {
            p.expect(SyntaxKind::Plus);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PathMod);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::QuestionMark,
                SyntaxKind::Star,
                SyntaxKind::Plus,
            ]);
        }
    };
    p.close(marker, SyntaxKind::PathMod);
}
/// [119] PathNegatedPropertySet -> PathOneInPropertySet | '(' (PathOneInPropertySet ('|' PathOneInPropertySet)*)? ')'
pub(super) fn parse_PathNegatedPropertySet(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::a
        | SyntaxKind::Zirkumflex
        | SyntaxKind::PNAME_LN => {
            parse_PathOneInPropertySet(p);
        }
        SyntaxKind::LParen => {
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::a,
                SyntaxKind::Zirkumflex,
                SyntaxKind::PNAME_LN,
            ]) {
                parse_PathOneInPropertySet(p);
                while [SyntaxKind::Pipe].contains(&p.nth(0)) {
                    let checkpoint = p.pos();
                    p.expect(SyntaxKind::Pipe);
                    parse_PathOneInPropertySet(p);
                    if p.pos() == checkpoint {
                        break;
                    }
                }
            }
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PathNegatedPropertySet);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::a,
                SyntaxKind::Zirkumflex,
                SyntaxKind::PNAME_LN,
                SyntaxKind::LParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::PathNegatedPropertySet);
}
/// [120] PathOneInPropertySet -> iri | 'a' | '^' (iri | 'a')
pub(super) fn parse_PathOneInPropertySet(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::a => {
            p.expect(SyntaxKind::a);
        }
        SyntaxKind::Zirkumflex => {
            p.expect(SyntaxKind::Zirkumflex);
            match p.nth(0) {
                SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
                    parse_iri(p);
                }
                SyntaxKind::a => {
                    p.expect(SyntaxKind::a);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::PathOneInPropertySet);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![
                        SyntaxKind::IRIREF,
                        SyntaxKind::PNAME_NS,
                        SyntaxKind::PNAME_LN,
                        SyntaxKind::a,
                    ]);
                }
            };
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PathOneInPropertySet);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::a,
                SyntaxKind::Zirkumflex,
            ]);
        }
    };
    p.close(marker, SyntaxKind::PathOneInPropertySet);
}
/// [121] Collection -> '(' GraphNode GraphNode* ')'
pub(super) fn parse_Collection(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LParen);
    parse_GraphNode(p);
    while [
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_GraphNode(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::Collection);
}
/// [122] BlankNodePropertyList -> '[' PropertyListNotEmpty ']'
pub(super) fn parse_BlankNodePropertyList(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LBrack);
    parse_PropertyListNotEmpty(p);
    p.expect(SyntaxKind::RBrack);
    p.close(marker, SyntaxKind::BlankNodePropertyList);
}
/// [123] CollectionPath -> '(' GraphNodePath GraphNodePath* ')'
pub(super) fn parse_CollectionPath(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LParen);
    parse_GraphNodePath(p);
    while [
        SyntaxKind::IRIREF,
        SyntaxKind::PNAME_NS,
        SyntaxKind::STRING_LITERAL1,
        SyntaxKind::STRING_LITERAL2,
        SyntaxKind::LParen,
        SyntaxKind::INTEGER,
        SyntaxKind::NIL,
        SyntaxKind::LBrack,
        SyntaxKind::DoubleLess,
        SyntaxKind::DoubleLessLParen,
        SyntaxKind::VAR1,
        SyntaxKind::VAR2,
        SyntaxKind::DECIMAL,
        SyntaxKind::DOUBLE,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
        SyntaxKind::True,
        SyntaxKind::False,
        SyntaxKind::STRING_LITERAL_LONG1,
        SyntaxKind::STRING_LITERAL_LONG2,
        SyntaxKind::PNAME_LN,
        SyntaxKind::BLANK_NODE_LABEL,
        SyntaxKind::ANON,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        parse_GraphNodePath(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::CollectionPath);
}
/// [124] BlankNodePropertyListPath -> '[' PropertyListPathNotEmpty ']'
pub(super) fn parse_BlankNodePropertyListPath(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LBrack);
    parse_PropertyListPathNotEmpty(p);
    p.expect(SyntaxKind::RBrack);
    p.close(marker, SyntaxKind::BlankNodePropertyListPath);
}
/// [125] AnnotationBlockPath -> '{|' PropertyListPathNotEmpty '|}'
pub(super) fn parse_AnnotationBlockPath(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LCurlyPipe);
    parse_PropertyListPathNotEmpty(p);
    p.expect(SyntaxKind::PipeRCurly);
    p.close(marker, SyntaxKind::AnnotationBlockPath);
}
/// [126] AnnotationBlock -> '{|' PropertyListNotEmpty '|}'
pub(super) fn parse_AnnotationBlock(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::LCurlyPipe);
    parse_PropertyListNotEmpty(p);
    p.expect(SyntaxKind::PipeRCurly);
    p.close(marker, SyntaxKind::AnnotationBlock);
}
/// [127] TripleTerm -> '<<(' TripleTermSubject Verb TripleTermObject ')>>'
pub(super) fn parse_TripleTerm(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DoubleLessLParen);
    parse_TripleTermSubject(p);
    parse_Verb(p);
    parse_TripleTermObject(p);
    p.expect(SyntaxKind::RParenDoubleMore);
    p.close(marker, SyntaxKind::TripleTerm);
}
/// [128] ReifiedTripleSubject -> Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | ReifiedTriple | TripleTerm
pub(super) fn parse_ReifiedTripleSubject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::BLANK_NODE_LABEL | SyntaxKind::ANON => {
            parse_BlankNode(p);
        }
        SyntaxKind::DoubleLess => {
            parse_ReifiedTriple(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ReifiedTripleSubject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::DoubleLess,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::ReifiedTripleSubject);
}
/// [129] ReifiedTripleObject -> Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | ReifiedTriple | TripleTerm
pub(super) fn parse_ReifiedTripleObject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::BLANK_NODE_LABEL | SyntaxKind::ANON => {
            parse_BlankNode(p);
        }
        SyntaxKind::DoubleLess => {
            parse_ReifiedTriple(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ReifiedTripleObject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::DoubleLess,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::ReifiedTripleObject);
}
/// [130] TripleTermSubject -> Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | TripleTerm
pub(super) fn parse_TripleTermSubject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::BLANK_NODE_LABEL | SyntaxKind::ANON => {
            parse_BlankNode(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TripleTermSubject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::TripleTermSubject);
}
/// [131] TripleTermObject -> Var | iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | TripleTerm
pub(super) fn parse_TripleTermObject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::BLANK_NODE_LABEL | SyntaxKind::ANON => {
            parse_BlankNode(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TripleTermObject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::BLANK_NODE_LABEL,
                SyntaxKind::ANON,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::TripleTermObject);
}
/// [132] TripleTermDataSubject -> iri
pub(super) fn parse_TripleTermDataSubject(p: &mut Parser) {
    let marker = p.open();
    parse_iri(p);
    p.close(marker, SyntaxKind::TripleTermDataSubject);
}
/// [133] TripleTermDataObject -> iri | RDFLiteral | NumericLiteral | BooleanLiteral | TripleTermData
pub(super) fn parse_TripleTermDataObject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_TripleTermData(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::TripleTermDataObject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::TripleTermDataObject);
}
/// [134] ConditionalOrExpression -> ConditionalAndExpression ('||' ConditionalAndExpression)*
pub(super) fn parse_ConditionalOrExpression(p: &mut Parser) {
    let marker = p.open();
    parse_ConditionalAndExpression(p);
    while [SyntaxKind::DoublePipe].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::DoublePipe);
        parse_ConditionalAndExpression(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::ConditionalOrExpression);
}
/// [135] ConditionalAndExpression -> ValueLogical ('&&' ValueLogical)*
pub(super) fn parse_ConditionalAndExpression(p: &mut Parser) {
    let marker = p.open();
    parse_ValueLogical(p);
    while [SyntaxKind::DoubleAnd].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        p.expect(SyntaxKind::DoubleAnd);
        parse_ValueLogical(p);
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::ConditionalAndExpression);
}
/// [136] ValueLogical -> RelationalExpression
pub(super) fn parse_ValueLogical(p: &mut Parser) {
    let marker = p.open();
    parse_RelationalExpression(p);
    p.close(marker, SyntaxKind::ValueLogical);
}
/// [137] RelationalExpression -> NumericExpression ('=' NumericExpression | '!=' NumericExpression | '<' NumericExpression | '>' NumericExpression | '<=' NumericExpression | '>=' NumericExpression | 'IN' ExpressionList | 'NOT' 'IN' ExpressionList)?
pub(super) fn parse_RelationalExpression(p: &mut Parser) {
    let marker = p.open();
    parse_NumericExpression(p);
    if p.at_any(&[
        SyntaxKind::Equals,
        SyntaxKind::ExclamationMarkEquals,
        SyntaxKind::Less,
        SyntaxKind::More,
        SyntaxKind::LessEquals,
        SyntaxKind::MoreEquals,
        SyntaxKind::IN,
        SyntaxKind::NOT,
    ]) {
        match p.nth(0) {
            SyntaxKind::Equals => {
                p.expect(SyntaxKind::Equals);
                parse_NumericExpression(p);
            }
            SyntaxKind::ExclamationMarkEquals => {
                p.expect(SyntaxKind::ExclamationMarkEquals);
                parse_NumericExpression(p);
            }
            SyntaxKind::Less => {
                p.expect(SyntaxKind::Less);
                parse_NumericExpression(p);
            }
            SyntaxKind::More => {
                p.expect(SyntaxKind::More);
                parse_NumericExpression(p);
            }
            SyntaxKind::LessEquals => {
                p.expect(SyntaxKind::LessEquals);
                parse_NumericExpression(p);
            }
            SyntaxKind::MoreEquals => {
                p.expect(SyntaxKind::MoreEquals);
                parse_NumericExpression(p);
            }
            SyntaxKind::IN => {
                p.expect(SyntaxKind::IN);
                parse_ExpressionList(p);
            }
            SyntaxKind::NOT => {
                p.expect(SyntaxKind::NOT);
                p.expect(SyntaxKind::IN);
                parse_ExpressionList(p);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::RelationalExpression);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![
                    SyntaxKind::Equals,
                    SyntaxKind::ExclamationMarkEquals,
                    SyntaxKind::Less,
                    SyntaxKind::More,
                    SyntaxKind::LessEquals,
                    SyntaxKind::MoreEquals,
                    SyntaxKind::IN,
                    SyntaxKind::NOT,
                ]);
            }
        };
    }
    p.close(marker, SyntaxKind::RelationalExpression);
}
/// [138] NumericExpression -> AdditiveExpression
pub(super) fn parse_NumericExpression(p: &mut Parser) {
    let marker = p.open();
    parse_AdditiveExpression(p);
    p.close(marker, SyntaxKind::NumericExpression);
}
/// [139] AdditiveExpression -> MultiplicativeExpression ('+' MultiplicativeExpression | '-' MultiplicativeExpression | (NumericLiteralPositive | NumericLiteralNegative) ('*' UnaryExpression | '/' UnaryExpression)*)*
pub(super) fn parse_AdditiveExpression(p: &mut Parser) {
    let marker = p.open();
    parse_MultiplicativeExpression(p);
    while [
        SyntaxKind::Plus,
        SyntaxKind::Minus,
        SyntaxKind::INTEGER_POSITIVE,
        SyntaxKind::DECIMAL_POSITIVE,
        SyntaxKind::DOUBLE_POSITIVE,
        SyntaxKind::INTEGER_NEGATIVE,
        SyntaxKind::DECIMAL_NEGATIVE,
        SyntaxKind::DOUBLE_NEGATIVE,
    ]
    .contains(&p.nth(0))
    {
        let checkpoint = p.pos();
        match p.nth(0) {
            SyntaxKind::Plus => {
                p.expect(SyntaxKind::Plus);
                parse_MultiplicativeExpression(p);
            }
            SyntaxKind::Minus => {
                p.expect(SyntaxKind::Minus);
                parse_MultiplicativeExpression(p);
            }
            SyntaxKind::INTEGER_POSITIVE
            | SyntaxKind::DECIMAL_POSITIVE
            | SyntaxKind::DOUBLE_POSITIVE
            | SyntaxKind::INTEGER_NEGATIVE
            | SyntaxKind::DECIMAL_NEGATIVE
            | SyntaxKind::DOUBLE_NEGATIVE => {
                match p.nth(0) {
                    SyntaxKind::INTEGER_POSITIVE
                    | SyntaxKind::DECIMAL_POSITIVE
                    | SyntaxKind::DOUBLE_POSITIVE => {
                        parse_NumericLiteralPositive(p);
                    }
                    SyntaxKind::INTEGER_NEGATIVE
                    | SyntaxKind::DECIMAL_NEGATIVE
                    | SyntaxKind::DOUBLE_NEGATIVE => {
                        parse_NumericLiteralNegative(p);
                    }
                    SyntaxKind::Eof => {
                        p.close(marker, SyntaxKind::AdditiveExpression);
                        let marker = p.open();
                        p.close(marker, SyntaxKind::Error);
                        return;
                    }
                    _ => {
                        p.advance_with_error(vec![
                            SyntaxKind::INTEGER_POSITIVE,
                            SyntaxKind::DECIMAL_POSITIVE,
                            SyntaxKind::DOUBLE_POSITIVE,
                            SyntaxKind::INTEGER_NEGATIVE,
                            SyntaxKind::DECIMAL_NEGATIVE,
                            SyntaxKind::DOUBLE_NEGATIVE,
                        ]);
                    }
                };
                while [SyntaxKind::Star, SyntaxKind::Slash].contains(&p.nth(0)) {
                    let checkpoint = p.pos();
                    match p.nth(0) {
                        SyntaxKind::Star => {
                            p.expect(SyntaxKind::Star);
                            parse_UnaryExpression(p);
                        }
                        SyntaxKind::Slash => {
                            p.expect(SyntaxKind::Slash);
                            parse_UnaryExpression(p);
                        }
                        SyntaxKind::Eof => {
                            p.close(marker, SyntaxKind::AdditiveExpression);
                            let marker = p.open();
                            p.close(marker, SyntaxKind::Error);
                            return;
                        }
                        _ => {
                            p.advance_with_error(vec![SyntaxKind::Star, SyntaxKind::Slash]);
                        }
                    };
                    if p.pos() == checkpoint {
                        break;
                    }
                }
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::AdditiveExpression);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![
                    SyntaxKind::Plus,
                    SyntaxKind::Minus,
                    SyntaxKind::INTEGER_POSITIVE,
                    SyntaxKind::DECIMAL_POSITIVE,
                    SyntaxKind::DOUBLE_POSITIVE,
                    SyntaxKind::INTEGER_NEGATIVE,
                    SyntaxKind::DECIMAL_NEGATIVE,
                    SyntaxKind::DOUBLE_NEGATIVE,
                ]);
            }
        };
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::AdditiveExpression);
}
/// [140] MultiplicativeExpression -> UnaryExpression ('*' UnaryExpression | '/' UnaryExpression)*
pub(super) fn parse_MultiplicativeExpression(p: &mut Parser) {
    let marker = p.open();
    parse_UnaryExpression(p);
    while [SyntaxKind::Star, SyntaxKind::Slash].contains(&p.nth(0)) {
        let checkpoint = p.pos();
        match p.nth(0) {
            SyntaxKind::Star => {
                p.expect(SyntaxKind::Star);
                parse_UnaryExpression(p);
            }
            SyntaxKind::Slash => {
                p.expect(SyntaxKind::Slash);
                parse_UnaryExpression(p);
            }
            SyntaxKind::Eof => {
                p.close(marker, SyntaxKind::MultiplicativeExpression);
                let marker = p.open();
                p.close(marker, SyntaxKind::Error);
                return;
            }
            _ => {
                p.advance_with_error(vec![SyntaxKind::Star, SyntaxKind::Slash]);
            }
        };
        if p.pos() == checkpoint {
            break;
        }
    }
    p.close(marker, SyntaxKind::MultiplicativeExpression);
}
/// [141] NumericLiteralPositive -> 'INTEGER_POSITIVE' | 'DECIMAL_POSITIVE' | 'DOUBLE_POSITIVE'
pub(super) fn parse_NumericLiteralPositive(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::INTEGER_POSITIVE => {
            p.expect(SyntaxKind::INTEGER_POSITIVE);
        }
        SyntaxKind::DECIMAL_POSITIVE => {
            p.expect(SyntaxKind::DECIMAL_POSITIVE);
        }
        SyntaxKind::DOUBLE_POSITIVE => {
            p.expect(SyntaxKind::DOUBLE_POSITIVE);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::NumericLiteralPositive);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
            ]);
        }
    };
    p.close(marker, SyntaxKind::NumericLiteralPositive);
}
/// [142] NumericLiteralNegative -> 'INTEGER_NEGATIVE' | 'DECIMAL_NEGATIVE' | 'DOUBLE_NEGATIVE'
pub(super) fn parse_NumericLiteralNegative(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::INTEGER_NEGATIVE => {
            p.expect(SyntaxKind::INTEGER_NEGATIVE);
        }
        SyntaxKind::DECIMAL_NEGATIVE => {
            p.expect(SyntaxKind::DECIMAL_NEGATIVE);
        }
        SyntaxKind::DOUBLE_NEGATIVE => {
            p.expect(SyntaxKind::DOUBLE_NEGATIVE);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::NumericLiteralNegative);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
            ]);
        }
    };
    p.close(marker, SyntaxKind::NumericLiteralNegative);
}
/// [143] UnaryExpression -> '!' UnaryExpression | '+' PrimaryExpression | '-' PrimaryExpression | PrimaryExpression
pub(super) fn parse_UnaryExpression(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::ExclamationMark => {
            p.expect(SyntaxKind::ExclamationMark);
            parse_UnaryExpression(p);
        }
        SyntaxKind::Plus => {
            p.expect(SyntaxKind::Plus);
            parse_PrimaryExpression(p);
        }
        SyntaxKind::Minus => {
            p.expect(SyntaxKind::Minus);
            parse_PrimaryExpression(p);
        }
        SyntaxKind::IRIREF
        | SyntaxKind::PNAME_NS
        | SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::LParen
        | SyntaxKind::INTEGER
        | SyntaxKind::DoubleLessLParen
        | SyntaxKind::VAR1
        | SyntaxKind::VAR2
        | SyntaxKind::NOT
        | SyntaxKind::STR
        | SyntaxKind::LANG
        | SyntaxKind::LANGMATCHES
        | SyntaxKind::LANGDIR
        | SyntaxKind::DATATYPE
        | SyntaxKind::BOUND
        | SyntaxKind::IRI
        | SyntaxKind::URI
        | SyntaxKind::BNODE
        | SyntaxKind::RAND
        | SyntaxKind::ABS
        | SyntaxKind::CEIL
        | SyntaxKind::FLOOR
        | SyntaxKind::ROUND
        | SyntaxKind::CONCAT
        | SyntaxKind::STRLEN
        | SyntaxKind::UCASE
        | SyntaxKind::LCASE
        | SyntaxKind::ENCODE_FOR_URI
        | SyntaxKind::CONTAINS
        | SyntaxKind::STRSTARTS
        | SyntaxKind::STRENDS
        | SyntaxKind::STRBEFORE
        | SyntaxKind::STRAFTER
        | SyntaxKind::YEAR
        | SyntaxKind::MONTH
        | SyntaxKind::DAY
        | SyntaxKind::HOURS
        | SyntaxKind::MINUTES
        | SyntaxKind::SECONDS
        | SyntaxKind::TIMEZONE
        | SyntaxKind::TZ
        | SyntaxKind::NOW
        | SyntaxKind::UUID
        | SyntaxKind::STRUUID
        | SyntaxKind::MD5
        | SyntaxKind::SHA1
        | SyntaxKind::SHA256
        | SyntaxKind::SHA384
        | SyntaxKind::SHA512
        | SyntaxKind::COALESCE
        | SyntaxKind::IF
        | SyntaxKind::STRLANG
        | SyntaxKind::STRLANGDIR
        | SyntaxKind::STRDT
        | SyntaxKind::sameTerm
        | SyntaxKind::isIRI
        | SyntaxKind::isURI
        | SyntaxKind::isBLANK
        | SyntaxKind::isLITERAL
        | SyntaxKind::isNUMERIC
        | SyntaxKind::hasLANG
        | SyntaxKind::hasLANGDIR
        | SyntaxKind::isTRIPLE
        | SyntaxKind::TRIPLE
        | SyntaxKind::SUBJECT
        | SyntaxKind::PREDICATE
        | SyntaxKind::OBJECT
        | SyntaxKind::REGEX
        | SyntaxKind::SUBSTR
        | SyntaxKind::REPLACE
        | SyntaxKind::EXISTS
        | SyntaxKind::COUNT
        | SyntaxKind::SUM
        | SyntaxKind::MIN
        | SyntaxKind::MAX
        | SyntaxKind::AVG
        | SyntaxKind::SAMPLE
        | SyntaxKind::GROUP_CONCAT
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE
        | SyntaxKind::True
        | SyntaxKind::False
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2
        | SyntaxKind::PNAME_LN => {
            parse_PrimaryExpression(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::UnaryExpression);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::ExclamationMark,
                SyntaxKind::Plus,
                SyntaxKind::Minus,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::LParen,
                SyntaxKind::INTEGER,
                SyntaxKind::DoubleLessLParen,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::NOT,
                SyntaxKind::STR,
                SyntaxKind::LANG,
                SyntaxKind::LANGMATCHES,
                SyntaxKind::LANGDIR,
                SyntaxKind::DATATYPE,
                SyntaxKind::BOUND,
                SyntaxKind::IRI,
                SyntaxKind::URI,
                SyntaxKind::BNODE,
                SyntaxKind::RAND,
                SyntaxKind::ABS,
                SyntaxKind::CEIL,
                SyntaxKind::FLOOR,
                SyntaxKind::ROUND,
                SyntaxKind::CONCAT,
                SyntaxKind::STRLEN,
                SyntaxKind::UCASE,
                SyntaxKind::LCASE,
                SyntaxKind::ENCODE_FOR_URI,
                SyntaxKind::CONTAINS,
                SyntaxKind::STRSTARTS,
                SyntaxKind::STRENDS,
                SyntaxKind::STRBEFORE,
                SyntaxKind::STRAFTER,
                SyntaxKind::YEAR,
                SyntaxKind::MONTH,
                SyntaxKind::DAY,
                SyntaxKind::HOURS,
                SyntaxKind::MINUTES,
                SyntaxKind::SECONDS,
                SyntaxKind::TIMEZONE,
                SyntaxKind::TZ,
                SyntaxKind::NOW,
                SyntaxKind::UUID,
                SyntaxKind::STRUUID,
                SyntaxKind::MD5,
                SyntaxKind::SHA1,
                SyntaxKind::SHA256,
                SyntaxKind::SHA384,
                SyntaxKind::SHA512,
                SyntaxKind::COALESCE,
                SyntaxKind::IF,
                SyntaxKind::STRLANG,
                SyntaxKind::STRLANGDIR,
                SyntaxKind::STRDT,
                SyntaxKind::sameTerm,
                SyntaxKind::isIRI,
                SyntaxKind::isURI,
                SyntaxKind::isBLANK,
                SyntaxKind::isLITERAL,
                SyntaxKind::isNUMERIC,
                SyntaxKind::hasLANG,
                SyntaxKind::hasLANGDIR,
                SyntaxKind::isTRIPLE,
                SyntaxKind::TRIPLE,
                SyntaxKind::SUBJECT,
                SyntaxKind::PREDICATE,
                SyntaxKind::OBJECT,
                SyntaxKind::REGEX,
                SyntaxKind::SUBSTR,
                SyntaxKind::REPLACE,
                SyntaxKind::EXISTS,
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::PNAME_LN,
            ]);
        }
    };
    p.close(marker, SyntaxKind::UnaryExpression);
}
/// [144] PrimaryExpression -> BrackettedExpression | BuiltInCall | iriOrFunction | RDFLiteral | NumericLiteral | BooleanLiteral | Var | ExprTripleTerm
pub(super) fn parse_PrimaryExpression(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::LParen => {
            parse_BrackettedExpression(p);
        }
        SyntaxKind::NOT
        | SyntaxKind::STR
        | SyntaxKind::LANG
        | SyntaxKind::LANGMATCHES
        | SyntaxKind::LANGDIR
        | SyntaxKind::DATATYPE
        | SyntaxKind::BOUND
        | SyntaxKind::IRI
        | SyntaxKind::URI
        | SyntaxKind::BNODE
        | SyntaxKind::RAND
        | SyntaxKind::ABS
        | SyntaxKind::CEIL
        | SyntaxKind::FLOOR
        | SyntaxKind::ROUND
        | SyntaxKind::CONCAT
        | SyntaxKind::STRLEN
        | SyntaxKind::UCASE
        | SyntaxKind::LCASE
        | SyntaxKind::ENCODE_FOR_URI
        | SyntaxKind::CONTAINS
        | SyntaxKind::STRSTARTS
        | SyntaxKind::STRENDS
        | SyntaxKind::STRBEFORE
        | SyntaxKind::STRAFTER
        | SyntaxKind::YEAR
        | SyntaxKind::MONTH
        | SyntaxKind::DAY
        | SyntaxKind::HOURS
        | SyntaxKind::MINUTES
        | SyntaxKind::SECONDS
        | SyntaxKind::TIMEZONE
        | SyntaxKind::TZ
        | SyntaxKind::NOW
        | SyntaxKind::UUID
        | SyntaxKind::STRUUID
        | SyntaxKind::MD5
        | SyntaxKind::SHA1
        | SyntaxKind::SHA256
        | SyntaxKind::SHA384
        | SyntaxKind::SHA512
        | SyntaxKind::COALESCE
        | SyntaxKind::IF
        | SyntaxKind::STRLANG
        | SyntaxKind::STRLANGDIR
        | SyntaxKind::STRDT
        | SyntaxKind::sameTerm
        | SyntaxKind::isIRI
        | SyntaxKind::isURI
        | SyntaxKind::isBLANK
        | SyntaxKind::isLITERAL
        | SyntaxKind::isNUMERIC
        | SyntaxKind::hasLANG
        | SyntaxKind::hasLANGDIR
        | SyntaxKind::isTRIPLE
        | SyntaxKind::TRIPLE
        | SyntaxKind::SUBJECT
        | SyntaxKind::PREDICATE
        | SyntaxKind::OBJECT
        | SyntaxKind::REGEX
        | SyntaxKind::SUBSTR
        | SyntaxKind::REPLACE
        | SyntaxKind::EXISTS
        | SyntaxKind::COUNT
        | SyntaxKind::SUM
        | SyntaxKind::MIN
        | SyntaxKind::MAX
        | SyntaxKind::AVG
        | SyntaxKind::SAMPLE
        | SyntaxKind::GROUP_CONCAT => {
            parse_BuiltInCall(p);
        }
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iriOrFunction(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_ExprTripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PrimaryExpression);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::LParen,
                SyntaxKind::NOT,
                SyntaxKind::STR,
                SyntaxKind::LANG,
                SyntaxKind::LANGMATCHES,
                SyntaxKind::LANGDIR,
                SyntaxKind::DATATYPE,
                SyntaxKind::BOUND,
                SyntaxKind::IRI,
                SyntaxKind::URI,
                SyntaxKind::BNODE,
                SyntaxKind::RAND,
                SyntaxKind::ABS,
                SyntaxKind::CEIL,
                SyntaxKind::FLOOR,
                SyntaxKind::ROUND,
                SyntaxKind::CONCAT,
                SyntaxKind::STRLEN,
                SyntaxKind::UCASE,
                SyntaxKind::LCASE,
                SyntaxKind::ENCODE_FOR_URI,
                SyntaxKind::CONTAINS,
                SyntaxKind::STRSTARTS,
                SyntaxKind::STRENDS,
                SyntaxKind::STRBEFORE,
                SyntaxKind::STRAFTER,
                SyntaxKind::YEAR,
                SyntaxKind::MONTH,
                SyntaxKind::DAY,
                SyntaxKind::HOURS,
                SyntaxKind::MINUTES,
                SyntaxKind::SECONDS,
                SyntaxKind::TIMEZONE,
                SyntaxKind::TZ,
                SyntaxKind::NOW,
                SyntaxKind::UUID,
                SyntaxKind::STRUUID,
                SyntaxKind::MD5,
                SyntaxKind::SHA1,
                SyntaxKind::SHA256,
                SyntaxKind::SHA384,
                SyntaxKind::SHA512,
                SyntaxKind::COALESCE,
                SyntaxKind::IF,
                SyntaxKind::STRLANG,
                SyntaxKind::STRLANGDIR,
                SyntaxKind::STRDT,
                SyntaxKind::sameTerm,
                SyntaxKind::isIRI,
                SyntaxKind::isURI,
                SyntaxKind::isBLANK,
                SyntaxKind::isLITERAL,
                SyntaxKind::isNUMERIC,
                SyntaxKind::hasLANG,
                SyntaxKind::hasLANGDIR,
                SyntaxKind::isTRIPLE,
                SyntaxKind::TRIPLE,
                SyntaxKind::SUBJECT,
                SyntaxKind::PREDICATE,
                SyntaxKind::OBJECT,
                SyntaxKind::REGEX,
                SyntaxKind::SUBSTR,
                SyntaxKind::REPLACE,
                SyntaxKind::EXISTS,
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::PrimaryExpression);
}
/// [145] iriOrFunction -> iri ArgList?
pub(super) fn parse_iriOrFunction(p: &mut Parser) {
    let marker = p.open();
    parse_iri(p);
    if p.at_any(&[SyntaxKind::LParen, SyntaxKind::NIL]) {
        parse_ArgList(p);
    }
    p.close(marker, SyntaxKind::iriOrFunction);
}
/// [146] ExprTripleTerm -> '<<(' ExprTripleTermSubject Verb ExprTripleTermObject ')>>'
pub(super) fn parse_ExprTripleTerm(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::DoubleLessLParen);
    parse_ExprTripleTermSubject(p);
    parse_Verb(p);
    parse_ExprTripleTermObject(p);
    p.expect(SyntaxKind::RParenDoubleMore);
    p.close(marker, SyntaxKind::ExprTripleTerm);
}
/// [147] ExprTripleTermSubject -> iri | Var
pub(super) fn parse_ExprTripleTermSubject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ExprTripleTermSubject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
            ]);
        }
    };
    p.close(marker, SyntaxKind::ExprTripleTermSubject);
}
/// [148] ExprTripleTermObject -> iri | RDFLiteral | NumericLiteral | BooleanLiteral | Var | ExprTripleTerm
pub(super) fn parse_ExprTripleTermObject(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::IRIREF | SyntaxKind::PNAME_NS | SyntaxKind::PNAME_LN => {
            parse_iri(p);
        }
        SyntaxKind::STRING_LITERAL1
        | SyntaxKind::STRING_LITERAL2
        | SyntaxKind::STRING_LITERAL_LONG1
        | SyntaxKind::STRING_LITERAL_LONG2 => {
            parse_RDFLiteral(p);
        }
        SyntaxKind::INTEGER
        | SyntaxKind::DECIMAL
        | SyntaxKind::DOUBLE
        | SyntaxKind::INTEGER_POSITIVE
        | SyntaxKind::DECIMAL_POSITIVE
        | SyntaxKind::DOUBLE_POSITIVE
        | SyntaxKind::INTEGER_NEGATIVE
        | SyntaxKind::DECIMAL_NEGATIVE
        | SyntaxKind::DOUBLE_NEGATIVE => {
            parse_NumericLiteral(p);
        }
        SyntaxKind::True | SyntaxKind::False => {
            parse_BooleanLiteral(p);
        }
        SyntaxKind::VAR1 | SyntaxKind::VAR2 => {
            parse_Var(p);
        }
        SyntaxKind::DoubleLessLParen => {
            parse_ExprTripleTerm(p);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::ExprTripleTermObject);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::IRIREF,
                SyntaxKind::PNAME_NS,
                SyntaxKind::PNAME_LN,
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
                SyntaxKind::INTEGER_POSITIVE,
                SyntaxKind::DECIMAL_POSITIVE,
                SyntaxKind::DOUBLE_POSITIVE,
                SyntaxKind::INTEGER_NEGATIVE,
                SyntaxKind::DECIMAL_NEGATIVE,
                SyntaxKind::DOUBLE_NEGATIVE,
                SyntaxKind::True,
                SyntaxKind::False,
                SyntaxKind::VAR1,
                SyntaxKind::VAR2,
                SyntaxKind::DoubleLessLParen,
            ]);
        }
    };
    p.close(marker, SyntaxKind::ExprTripleTermObject);
}
/// [149] Aggregate -> 'COUNT' '(' 'DISTINCT'? ('*' | Expression) ')' | 'SUM' '(' 'DISTINCT'? Expression ')' | 'MIN' '(' 'DISTINCT'? Expression ')' | 'MAX' '(' 'DISTINCT'? Expression ')' | 'AVG' '(' 'DISTINCT'? Expression ')' | 'SAMPLE' '(' 'DISTINCT'? Expression ')' | 'GROUP_CONCAT' '(' 'DISTINCT'? Expression (';' 'SEPARATOR' '=' String)? ')'
pub(super) fn parse_Aggregate(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::COUNT => {
            p.expect(SyntaxKind::COUNT);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            match p.nth(0) {
                SyntaxKind::Star => {
                    p.expect(SyntaxKind::Star);
                }
                SyntaxKind::IRIREF
                | SyntaxKind::PNAME_NS
                | SyntaxKind::STRING_LITERAL1
                | SyntaxKind::STRING_LITERAL2
                | SyntaxKind::LParen
                | SyntaxKind::INTEGER
                | SyntaxKind::Plus
                | SyntaxKind::ExclamationMark
                | SyntaxKind::DoubleLessLParen
                | SyntaxKind::VAR1
                | SyntaxKind::VAR2
                | SyntaxKind::NOT
                | SyntaxKind::Minus
                | SyntaxKind::STR
                | SyntaxKind::LANG
                | SyntaxKind::LANGMATCHES
                | SyntaxKind::LANGDIR
                | SyntaxKind::DATATYPE
                | SyntaxKind::BOUND
                | SyntaxKind::IRI
                | SyntaxKind::URI
                | SyntaxKind::BNODE
                | SyntaxKind::RAND
                | SyntaxKind::ABS
                | SyntaxKind::CEIL
                | SyntaxKind::FLOOR
                | SyntaxKind::ROUND
                | SyntaxKind::CONCAT
                | SyntaxKind::STRLEN
                | SyntaxKind::UCASE
                | SyntaxKind::LCASE
                | SyntaxKind::ENCODE_FOR_URI
                | SyntaxKind::CONTAINS
                | SyntaxKind::STRSTARTS
                | SyntaxKind::STRENDS
                | SyntaxKind::STRBEFORE
                | SyntaxKind::STRAFTER
                | SyntaxKind::YEAR
                | SyntaxKind::MONTH
                | SyntaxKind::DAY
                | SyntaxKind::HOURS
                | SyntaxKind::MINUTES
                | SyntaxKind::SECONDS
                | SyntaxKind::TIMEZONE
                | SyntaxKind::TZ
                | SyntaxKind::NOW
                | SyntaxKind::UUID
                | SyntaxKind::STRUUID
                | SyntaxKind::MD5
                | SyntaxKind::SHA1
                | SyntaxKind::SHA256
                | SyntaxKind::SHA384
                | SyntaxKind::SHA512
                | SyntaxKind::COALESCE
                | SyntaxKind::IF
                | SyntaxKind::STRLANG
                | SyntaxKind::STRLANGDIR
                | SyntaxKind::STRDT
                | SyntaxKind::sameTerm
                | SyntaxKind::isIRI
                | SyntaxKind::isURI
                | SyntaxKind::isBLANK
                | SyntaxKind::isLITERAL
                | SyntaxKind::isNUMERIC
                | SyntaxKind::hasLANG
                | SyntaxKind::hasLANGDIR
                | SyntaxKind::isTRIPLE
                | SyntaxKind::TRIPLE
                | SyntaxKind::SUBJECT
                | SyntaxKind::PREDICATE
                | SyntaxKind::OBJECT
                | SyntaxKind::REGEX
                | SyntaxKind::SUBSTR
                | SyntaxKind::REPLACE
                | SyntaxKind::EXISTS
                | SyntaxKind::COUNT
                | SyntaxKind::SUM
                | SyntaxKind::MIN
                | SyntaxKind::MAX
                | SyntaxKind::AVG
                | SyntaxKind::SAMPLE
                | SyntaxKind::GROUP_CONCAT
                | SyntaxKind::DECIMAL
                | SyntaxKind::DOUBLE
                | SyntaxKind::INTEGER_POSITIVE
                | SyntaxKind::DECIMAL_POSITIVE
                | SyntaxKind::DOUBLE_POSITIVE
                | SyntaxKind::INTEGER_NEGATIVE
                | SyntaxKind::DECIMAL_NEGATIVE
                | SyntaxKind::DOUBLE_NEGATIVE
                | SyntaxKind::True
                | SyntaxKind::False
                | SyntaxKind::STRING_LITERAL_LONG1
                | SyntaxKind::STRING_LITERAL_LONG2
                | SyntaxKind::PNAME_LN => {
                    parse_Expression(p);
                }
                SyntaxKind::Eof => {
                    p.close(marker, SyntaxKind::Aggregate);
                    let marker = p.open();
                    p.close(marker, SyntaxKind::Error);
                    return;
                }
                _ => {
                    p.advance_with_error(vec![
                        SyntaxKind::Star,
                        SyntaxKind::IRIREF,
                        SyntaxKind::PNAME_NS,
                        SyntaxKind::STRING_LITERAL1,
                        SyntaxKind::STRING_LITERAL2,
                        SyntaxKind::LParen,
                        SyntaxKind::INTEGER,
                        SyntaxKind::Plus,
                        SyntaxKind::ExclamationMark,
                        SyntaxKind::DoubleLessLParen,
                        SyntaxKind::VAR1,
                        SyntaxKind::VAR2,
                        SyntaxKind::NOT,
                        SyntaxKind::Minus,
                        SyntaxKind::STR,
                        SyntaxKind::LANG,
                        SyntaxKind::LANGMATCHES,
                        SyntaxKind::LANGDIR,
                        SyntaxKind::DATATYPE,
                        SyntaxKind::BOUND,
                        SyntaxKind::IRI,
                        SyntaxKind::URI,
                        SyntaxKind::BNODE,
                        SyntaxKind::RAND,
                        SyntaxKind::ABS,
                        SyntaxKind::CEIL,
                        SyntaxKind::FLOOR,
                        SyntaxKind::ROUND,
                        SyntaxKind::CONCAT,
                        SyntaxKind::STRLEN,
                        SyntaxKind::UCASE,
                        SyntaxKind::LCASE,
                        SyntaxKind::ENCODE_FOR_URI,
                        SyntaxKind::CONTAINS,
                        SyntaxKind::STRSTARTS,
                        SyntaxKind::STRENDS,
                        SyntaxKind::STRBEFORE,
                        SyntaxKind::STRAFTER,
                        SyntaxKind::YEAR,
                        SyntaxKind::MONTH,
                        SyntaxKind::DAY,
                        SyntaxKind::HOURS,
                        SyntaxKind::MINUTES,
                        SyntaxKind::SECONDS,
                        SyntaxKind::TIMEZONE,
                        SyntaxKind::TZ,
                        SyntaxKind::NOW,
                        SyntaxKind::UUID,
                        SyntaxKind::STRUUID,
                        SyntaxKind::MD5,
                        SyntaxKind::SHA1,
                        SyntaxKind::SHA256,
                        SyntaxKind::SHA384,
                        SyntaxKind::SHA512,
                        SyntaxKind::COALESCE,
                        SyntaxKind::IF,
                        SyntaxKind::STRLANG,
                        SyntaxKind::STRLANGDIR,
                        SyntaxKind::STRDT,
                        SyntaxKind::sameTerm,
                        SyntaxKind::isIRI,
                        SyntaxKind::isURI,
                        SyntaxKind::isBLANK,
                        SyntaxKind::isLITERAL,
                        SyntaxKind::isNUMERIC,
                        SyntaxKind::hasLANG,
                        SyntaxKind::hasLANGDIR,
                        SyntaxKind::isTRIPLE,
                        SyntaxKind::TRIPLE,
                        SyntaxKind::SUBJECT,
                        SyntaxKind::PREDICATE,
                        SyntaxKind::OBJECT,
                        SyntaxKind::REGEX,
                        SyntaxKind::SUBSTR,
                        SyntaxKind::REPLACE,
                        SyntaxKind::EXISTS,
                        SyntaxKind::COUNT,
                        SyntaxKind::SUM,
                        SyntaxKind::MIN,
                        SyntaxKind::MAX,
                        SyntaxKind::AVG,
                        SyntaxKind::SAMPLE,
                        SyntaxKind::GROUP_CONCAT,
                        SyntaxKind::DECIMAL,
                        SyntaxKind::DOUBLE,
                        SyntaxKind::INTEGER_POSITIVE,
                        SyntaxKind::DECIMAL_POSITIVE,
                        SyntaxKind::DOUBLE_POSITIVE,
                        SyntaxKind::INTEGER_NEGATIVE,
                        SyntaxKind::DECIMAL_NEGATIVE,
                        SyntaxKind::DOUBLE_NEGATIVE,
                        SyntaxKind::True,
                        SyntaxKind::False,
                        SyntaxKind::STRING_LITERAL_LONG1,
                        SyntaxKind::STRING_LITERAL_LONG2,
                        SyntaxKind::PNAME_LN,
                    ]);
                }
            };
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SUM => {
            p.expect(SyntaxKind::SUM);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::MIN => {
            p.expect(SyntaxKind::MIN);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::MAX => {
            p.expect(SyntaxKind::MAX);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::AVG => {
            p.expect(SyntaxKind::AVG);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::SAMPLE => {
            p.expect(SyntaxKind::SAMPLE);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::GROUP_CONCAT => {
            p.expect(SyntaxKind::GROUP_CONCAT);
            p.expect(SyntaxKind::LParen);
            if p.at_any(&[SyntaxKind::DISTINCT]) {
                p.expect(SyntaxKind::DISTINCT);
            }
            parse_Expression(p);
            if p.at_any(&[SyntaxKind::Semicolon]) {
                p.expect(SyntaxKind::Semicolon);
                p.expect(SyntaxKind::SEPARATOR);
                p.expect(SyntaxKind::Equals);
                parse_String(p);
            }
            p.expect(SyntaxKind::RParen);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::Aggregate);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::COUNT,
                SyntaxKind::SUM,
                SyntaxKind::MIN,
                SyntaxKind::MAX,
                SyntaxKind::AVG,
                SyntaxKind::SAMPLE,
                SyntaxKind::GROUP_CONCAT,
            ]);
        }
    };
    p.close(marker, SyntaxKind::Aggregate);
}
/// [150] SubstringExpression -> 'SUBSTR' '(' Expression ',' Expression (',' Expression)? ')'
pub(super) fn parse_SubstringExpression(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::SUBSTR);
    p.expect(SyntaxKind::LParen);
    parse_Expression(p);
    p.expect(SyntaxKind::Comma);
    parse_Expression(p);
    if p.at_any(&[SyntaxKind::Comma]) {
        p.expect(SyntaxKind::Comma);
        parse_Expression(p);
    }
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::SubstringExpression);
}
/// [151] StrReplaceExpression -> 'REPLACE' '(' Expression ',' Expression ',' Expression (',' Expression)? ')'
pub(super) fn parse_StrReplaceExpression(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::REPLACE);
    p.expect(SyntaxKind::LParen);
    parse_Expression(p);
    p.expect(SyntaxKind::Comma);
    parse_Expression(p);
    p.expect(SyntaxKind::Comma);
    parse_Expression(p);
    if p.at_any(&[SyntaxKind::Comma]) {
        p.expect(SyntaxKind::Comma);
        parse_Expression(p);
    }
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::StrReplaceExpression);
}
/// [152] RegexExpression -> 'REGEX' '(' Expression ',' Expression (',' Expression)? ')'
pub(super) fn parse_RegexExpression(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::REGEX);
    p.expect(SyntaxKind::LParen);
    parse_Expression(p);
    p.expect(SyntaxKind::Comma);
    parse_Expression(p);
    if p.at_any(&[SyntaxKind::Comma]) {
        p.expect(SyntaxKind::Comma);
        parse_Expression(p);
    }
    p.expect(SyntaxKind::RParen);
    p.close(marker, SyntaxKind::RegexExpression);
}
/// [153] ExistsFunc -> 'EXISTS' GroupGraphPattern
pub(super) fn parse_ExistsFunc(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::EXISTS);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::ExistsFunc);
}
/// [154] NotExistsFunc -> 'NOT' 'EXISTS' GroupGraphPattern
pub(super) fn parse_NotExistsFunc(p: &mut Parser) {
    let marker = p.open();
    p.expect(SyntaxKind::NOT);
    p.expect(SyntaxKind::EXISTS);
    parse_GroupGraphPattern(p);
    p.close(marker, SyntaxKind::NotExistsFunc);
}
/// [155] String -> 'STRING_LITERAL1' | 'STRING_LITERAL2' | 'STRING_LITERAL_LONG1' | 'STRING_LITERAL_LONG2'
pub(super) fn parse_String(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::STRING_LITERAL1 => {
            p.expect(SyntaxKind::STRING_LITERAL1);
        }
        SyntaxKind::STRING_LITERAL2 => {
            p.expect(SyntaxKind::STRING_LITERAL2);
        }
        SyntaxKind::STRING_LITERAL_LONG1 => {
            p.expect(SyntaxKind::STRING_LITERAL_LONG1);
        }
        SyntaxKind::STRING_LITERAL_LONG2 => {
            p.expect(SyntaxKind::STRING_LITERAL_LONG2);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::String);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::STRING_LITERAL1,
                SyntaxKind::STRING_LITERAL2,
                SyntaxKind::STRING_LITERAL_LONG1,
                SyntaxKind::STRING_LITERAL_LONG2,
            ]);
        }
    };
    p.close(marker, SyntaxKind::String);
}
/// [156] NumericLiteralUnsigned -> 'INTEGER' | 'DECIMAL' | 'DOUBLE'
pub(super) fn parse_NumericLiteralUnsigned(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::INTEGER => {
            p.expect(SyntaxKind::INTEGER);
        }
        SyntaxKind::DECIMAL => {
            p.expect(SyntaxKind::DECIMAL);
        }
        SyntaxKind::DOUBLE => {
            p.expect(SyntaxKind::DOUBLE);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::NumericLiteralUnsigned);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![
                SyntaxKind::INTEGER,
                SyntaxKind::DECIMAL,
                SyntaxKind::DOUBLE,
            ]);
        }
    };
    p.close(marker, SyntaxKind::NumericLiteralUnsigned);
}
/// [157] PrefixedName -> 'PNAME_LN' | 'PNAME_NS'
pub(super) fn parse_PrefixedName(p: &mut Parser) {
    let marker = p.open();
    match p.nth(0) {
        SyntaxKind::PNAME_LN => {
            p.expect(SyntaxKind::PNAME_LN);
        }
        SyntaxKind::PNAME_NS => {
            p.expect(SyntaxKind::PNAME_NS);
        }
        SyntaxKind::Eof => {
            p.close(marker, SyntaxKind::PrefixedName);
            let marker = p.open();
            p.close(marker, SyntaxKind::Error);
            return;
        }
        _ => {
            p.advance_with_error(vec![SyntaxKind::PNAME_LN, SyntaxKind::PNAME_NS]);
        }
    };
    p.close(marker, SyntaxKind::PrefixedName);
}
