use crate::syntax_kind::SyntaxKind;
#[derive(Debug, Clone)]
pub(super) enum Rule {
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
            Rule::Seq(rules) => {
                let mut first = vec![];
                for rule in rules {
                    first.extend(rule.first().iter());
                    if !rule.is_nullable() {
                        break;
                    }
                }
                return first;
            }
            Rule::Alt(rules) => rules.iter().flat_map(|rule| rule.first()).collect(),
            Rule::Opt(rule) => rule.first(),
            Rule::Rep(rule) => rule.first(),
        }
    }
    pub(super) fn is_nullable(&self) -> bool {
        match self {
            Rule::Opt(_rule) | Rule::Rep(_rule) => true,
            Rule::Node(syntax_kind) => {
                Rule::from_node_kind(*syntax_kind).map_or(false, |rule| rule.is_nullable())
            }
            Rule::Token(_syntax_kind) => false,
            Rule::Seq(rules) => rules.iter().all(|rule| rule.is_nullable()),
            Rule::Alt(rules) => rules.iter().any(|rule| rule.is_nullable()),
        }
    }
    pub(super) fn from_node_kind(kind: SyntaxKind) -> Option<Self> {
        match kind {
            SyntaxKind::QueryUnit => Some(Rule::Node(SyntaxKind::Query)),
            SyntaxKind::Query => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::Prologue),
                Rule::Alt(vec![
                    Rule::Node(SyntaxKind::SelectQuery),
                    Rule::Node(SyntaxKind::ConstructQuery),
                    Rule::Node(SyntaxKind::DescribeQuery),
                    Rule::Node(SyntaxKind::AskQuery),
                ]),
                Rule::Node(SyntaxKind::ValuesClause),
            ])),
            SyntaxKind::Prologue => Some(Rule::Rep(Box::new(Rule::Alt(vec![
                Rule::Node(SyntaxKind::BaseDecl),
                Rule::Node(SyntaxKind::PrefixDecl),
            ])))),
            SyntaxKind::SelectQuery => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::SelectClause),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::DatasetClause))),
                Rule::Node(SyntaxKind::WhereClause),
                Rule::Node(SyntaxKind::SolutionModifier),
            ])),
            SyntaxKind::ConstructQuery => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::CONSTRUCT),
                Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Node(SyntaxKind::ConstructTemplate),
                        Rule::Rep(Box::new(Rule::Node(SyntaxKind::DatasetClause))),
                        Rule::Node(SyntaxKind::WhereClause),
                        Rule::Node(SyntaxKind::SolutionModifier),
                    ]),
                    Rule::Seq(vec![
                        Rule::Rep(Box::new(Rule::Node(SyntaxKind::DatasetClause))),
                        Rule::Token(SyntaxKind::WHERE),
                        Rule::Token(SyntaxKind::LCurly),
                        Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesTemplate))),
                        Rule::Token(SyntaxKind::RCurly),
                        Rule::Node(SyntaxKind::SolutionModifier),
                    ]),
                ]),
            ])),
            SyntaxKind::DescribeQuery => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::DESCRIBE),
                Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Node(SyntaxKind::VarOrIri),
                        Rule::Rep(Box::new(Rule::Node(SyntaxKind::VarOrIri))),
                    ]),
                    Rule::Token(SyntaxKind::Star),
                ]),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::DatasetClause))),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::WhereClause))),
                Rule::Node(SyntaxKind::SolutionModifier),
            ])),
            SyntaxKind::AskQuery => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::ASK),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::DatasetClause))),
                Rule::Node(SyntaxKind::WhereClause),
                Rule::Node(SyntaxKind::SolutionModifier),
            ])),
            SyntaxKind::ValuesClause => Some(Rule::Opt(Box::new(Rule::Seq(vec![
                Rule::Token(SyntaxKind::VALUES),
                Rule::Node(SyntaxKind::DataBlock),
            ])))),
            SyntaxKind::UpdateUnit => Some(Rule::Node(SyntaxKind::Update)),
            SyntaxKind::Update => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::Prologue),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Node(SyntaxKind::UpdateOne),
                    Rule::Opt(Box::new(Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Semicolon),
                        Rule::Node(SyntaxKind::Update),
                    ]))),
                ]))),
            ])),
            SyntaxKind::BaseDecl => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::BASE),
                Rule::Token(SyntaxKind::IRIREF),
            ])),
            SyntaxKind::PrefixDecl => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::PREFIX),
                Rule::Token(SyntaxKind::PNAME_NS),
                Rule::Token(SyntaxKind::IRIREF),
            ])),
            SyntaxKind::SelectClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::SELECT),
                Rule::Opt(Box::new(Rule::Alt(vec![
                    Rule::Token(SyntaxKind::DISTINCT),
                    Rule::Token(SyntaxKind::REDUCED),
                ]))),
                Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Alt(vec![
                            Rule::Node(SyntaxKind::Var),
                            Rule::Seq(vec![
                                Rule::Token(SyntaxKind::LParen),
                                Rule::Node(SyntaxKind::Expression),
                                Rule::Token(SyntaxKind::AS),
                                Rule::Node(SyntaxKind::Var),
                                Rule::Token(SyntaxKind::RParen),
                            ]),
                        ]),
                        Rule::Rep(Box::new(Rule::Alt(vec![
                            Rule::Node(SyntaxKind::Var),
                            Rule::Seq(vec![
                                Rule::Token(SyntaxKind::LParen),
                                Rule::Node(SyntaxKind::Expression),
                                Rule::Token(SyntaxKind::AS),
                                Rule::Node(SyntaxKind::Var),
                                Rule::Token(SyntaxKind::RParen),
                            ]),
                        ]))),
                    ]),
                    Rule::Token(SyntaxKind::Star),
                ]),
            ])),
            SyntaxKind::DatasetClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::FROM),
                Rule::Alt(vec![
                    Rule::Node(SyntaxKind::DefaultGraphClause),
                    Rule::Node(SyntaxKind::NamedGraphClause),
                ]),
            ])),
            SyntaxKind::WhereClause => Some(Rule::Seq(vec![
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::WHERE))),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::SolutionModifier => Some(Rule::Seq(vec![
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::GroupClause))),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::HavingClause))),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::OrderClause))),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::LimitOffsetClauses))),
            ])),
            SyntaxKind::SubSelect => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::SelectClause),
                Rule::Node(SyntaxKind::WhereClause),
                Rule::Node(SyntaxKind::SolutionModifier),
                Rule::Node(SyntaxKind::ValuesClause),
            ])),
            SyntaxKind::Var => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::VAR1),
                Rule::Token(SyntaxKind::VAR2),
            ])),
            SyntaxKind::Expression => Some(Rule::Node(SyntaxKind::ConditionalOrExpression)),
            SyntaxKind::ConstructTemplate => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LCurly),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::ConstructTriples))),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::TriplesTemplate => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::TriplesSameSubject),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Dot),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesTemplate))),
                ]))),
            ])),
            SyntaxKind::VarOrIri => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::Var),
                Rule::Node(SyntaxKind::iri),
            ])),
            SyntaxKind::DefaultGraphClause => Some(Rule::Node(SyntaxKind::SourceSelector)),
            SyntaxKind::NamedGraphClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::NAMED),
                Rule::Node(SyntaxKind::SourceSelector),
            ])),
            SyntaxKind::SourceSelector => Some(Rule::Node(SyntaxKind::iri)),
            SyntaxKind::iri => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::IRIREF),
                Rule::Node(SyntaxKind::PrefixedName),
            ])),
            SyntaxKind::GroupGraphPattern => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LCurly),
                Rule::Alt(vec![
                    Rule::Node(SyntaxKind::SubSelect),
                    Rule::Node(SyntaxKind::GroupGraphPatternSub),
                ]),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::GroupClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::GROUP),
                Rule::Token(SyntaxKind::BY),
                Rule::Node(SyntaxKind::GroupCondition),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::GroupCondition))),
            ])),
            SyntaxKind::HavingClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::HAVING),
                Rule::Node(SyntaxKind::HavingCondition),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::HavingCondition))),
            ])),
            SyntaxKind::OrderClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::ORDER),
                Rule::Token(SyntaxKind::BY),
                Rule::Node(SyntaxKind::OrderCondition),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::OrderCondition))),
            ])),
            SyntaxKind::LimitOffsetClauses => Some(Rule::Alt(vec![
                Rule::Seq(vec![
                    Rule::Node(SyntaxKind::LimitClause),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::OffsetClause))),
                ]),
                Rule::Seq(vec![
                    Rule::Node(SyntaxKind::OffsetClause),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::LimitClause))),
                ]),
            ])),
            SyntaxKind::GroupCondition => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::BuiltInCall),
                Rule::Node(SyntaxKind::FunctionCall),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Opt(Box::new(Rule::Seq(vec![
                        Rule::Token(SyntaxKind::AS),
                        Rule::Node(SyntaxKind::Var),
                    ]))),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Node(SyntaxKind::Var),
            ])),
            SyntaxKind::BuiltInCall => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::Aggregate),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STR),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LANG),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LANGMATCHES),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::DATATYPE),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::BOUND),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Var),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::IRI),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::URI),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::BNODE),
                    Rule::Alt(vec![
                        Rule::Seq(vec![
                            Rule::Token(SyntaxKind::LParen),
                            Rule::Node(SyntaxKind::Expression),
                            Rule::Token(SyntaxKind::RParen),
                        ]),
                        Rule::Token(SyntaxKind::NIL),
                    ]),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::RAND),
                    Rule::Token(SyntaxKind::NIL),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::ABS),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::CEIL),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::FLOOR),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::ROUND),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::CONCAT),
                    Rule::Node(SyntaxKind::ExpressionList),
                ]),
                Rule::Node(SyntaxKind::SubstringExpression),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRLEN),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Node(SyntaxKind::StrReplaceExpression),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::UCASE),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LCASE),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::ENCODE_FOR_URI),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::CONTAINS),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRSTARTS),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRENDS),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRBEFORE),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRAFTER),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::YEAR),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::MONTH),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::DAY),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::HOURS),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::MINUTES),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SECONDS),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::TIMEZONE),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::TZ),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::NOW),
                    Rule::Token(SyntaxKind::NIL),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::UUID),
                    Rule::Token(SyntaxKind::NIL),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRUUID),
                    Rule::Token(SyntaxKind::NIL),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::MD5),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SHA1),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SHA256),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SHA384),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SHA512),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::COALESCE),
                    Rule::Node(SyntaxKind::ExpressionList),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::IF),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRLANG),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::STRDT),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::sameTerm),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::isIRI),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::isURI),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::isBLANK),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::isLITERAL),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::isNUMERIC),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Node(SyntaxKind::RegexExpression),
                Rule::Node(SyntaxKind::ExistsFunc),
                Rule::Node(SyntaxKind::NotExistsFunc),
            ])),
            SyntaxKind::FunctionCall => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::iri),
                Rule::Node(SyntaxKind::ArgList),
            ])),
            SyntaxKind::HavingCondition => Some(Rule::Node(SyntaxKind::Constraint)),
            SyntaxKind::Constraint => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::BrackettedExpression),
                Rule::Node(SyntaxKind::BuiltInCall),
                Rule::Node(SyntaxKind::FunctionCall),
            ])),
            SyntaxKind::OrderCondition => Some(Rule::Alt(vec![
                Rule::Seq(vec![
                    Rule::Alt(vec![
                        Rule::Token(SyntaxKind::ASC),
                        Rule::Token(SyntaxKind::DESC),
                    ]),
                    Rule::Node(SyntaxKind::BrackettedExpression),
                ]),
                Rule::Alt(vec![
                    Rule::Node(SyntaxKind::Constraint),
                    Rule::Node(SyntaxKind::Var),
                ]),
            ])),
            SyntaxKind::BrackettedExpression => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::Expression),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::LimitClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LIMIT),
                Rule::Token(SyntaxKind::INTEGER),
            ])),
            SyntaxKind::OffsetClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::OFFSET),
                Rule::Token(SyntaxKind::INTEGER),
            ])),
            SyntaxKind::DataBlock => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::InlineDataOneVar),
                Rule::Node(SyntaxKind::InlineDataFull),
            ])),
            SyntaxKind::UpdateOne => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::Load),
                Rule::Node(SyntaxKind::Clear),
                Rule::Node(SyntaxKind::Drop),
                Rule::Node(SyntaxKind::Add),
                Rule::Node(SyntaxKind::Move),
                Rule::Node(SyntaxKind::Copy),
                Rule::Node(SyntaxKind::Create),
                Rule::Node(SyntaxKind::InsertData),
                Rule::Node(SyntaxKind::DeleteData),
                Rule::Node(SyntaxKind::DeleteWhere),
                Rule::Node(SyntaxKind::Modify),
            ])),
            SyntaxKind::Load => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LOAD),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::iri),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::INTO),
                    Rule::Node(SyntaxKind::GraphRef),
                ]))),
            ])),
            SyntaxKind::Clear => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::CLEAR),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::GraphRefAll),
            ])),
            SyntaxKind::Drop => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::DROP),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::GraphRefAll),
            ])),
            SyntaxKind::Add => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::ADD),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::GraphOrDefault),
                Rule::Token(SyntaxKind::TO),
                Rule::Node(SyntaxKind::GraphOrDefault),
            ])),
            SyntaxKind::Move => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::MOVE),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::GraphOrDefault),
                Rule::Token(SyntaxKind::TO),
                Rule::Node(SyntaxKind::GraphOrDefault),
            ])),
            SyntaxKind::Copy => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::COPY),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::GraphOrDefault),
                Rule::Token(SyntaxKind::TO),
                Rule::Node(SyntaxKind::GraphOrDefault),
            ])),
            SyntaxKind::Create => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::CREATE),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::GraphRef),
            ])),
            SyntaxKind::InsertData => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::INSERT_DATA),
                Rule::Node(SyntaxKind::QuadData),
            ])),
            SyntaxKind::DeleteData => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::DELETE_DATA),
                Rule::Node(SyntaxKind::QuadData),
            ])),
            SyntaxKind::DeleteWhere => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::DELETE_WHERE),
                Rule::Node(SyntaxKind::QuadPattern),
            ])),
            SyntaxKind::Modify => Some(Rule::Seq(vec![
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::WITH),
                    Rule::Node(SyntaxKind::iri),
                ]))),
                Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Node(SyntaxKind::DeleteClause),
                        Rule::Opt(Box::new(Rule::Node(SyntaxKind::InsertClause))),
                    ]),
                    Rule::Node(SyntaxKind::InsertClause),
                ]),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::UsingClause))),
                Rule::Token(SyntaxKind::WHERE),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::GraphRef => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::GRAPH),
                Rule::Node(SyntaxKind::iri),
            ])),
            SyntaxKind::GraphRefAll => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::GraphRef),
                Rule::Token(SyntaxKind::DEFAULT),
                Rule::Token(SyntaxKind::NAMED),
                Rule::Token(SyntaxKind::ALL),
            ])),
            SyntaxKind::GraphOrDefault => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::DEFAULT),
                Rule::Seq(vec![
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::GRAPH))),
                    Rule::Node(SyntaxKind::iri),
                ]),
            ])),
            SyntaxKind::QuadData => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LCurly),
                Rule::Node(SyntaxKind::Quads),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::QuadPattern => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LCurly),
                Rule::Node(SyntaxKind::Quads),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::DeleteClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::DELETE),
                Rule::Node(SyntaxKind::QuadPattern),
            ])),
            SyntaxKind::InsertClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::INSERT),
                Rule::Node(SyntaxKind::QuadPattern),
            ])),
            SyntaxKind::UsingClause => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::USING),
                Rule::Alt(vec![
                    Rule::Node(SyntaxKind::iri),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::NAMED),
                        Rule::Node(SyntaxKind::iri),
                    ]),
                ]),
            ])),
            SyntaxKind::Quads => Some(Rule::Seq(vec![
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesTemplate))),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Node(SyntaxKind::QuadsNotTriples),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::Dot))),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesTemplate))),
                ]))),
            ])),
            SyntaxKind::QuadsNotTriples => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::GRAPH),
                Rule::Node(SyntaxKind::VarOrIri),
                Rule::Token(SyntaxKind::LCurly),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesTemplate))),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::TriplesSameSubject => Some(Rule::Alt(vec![
                Rule::Seq(vec![
                    Rule::Node(SyntaxKind::VarOrTerm),
                    Rule::Node(SyntaxKind::PropertyListNotEmpty),
                ]),
                Rule::Seq(vec![
                    Rule::Node(SyntaxKind::TriplesNode),
                    Rule::Node(SyntaxKind::PropertyList),
                ]),
            ])),
            SyntaxKind::GroupGraphPatternSub => Some(Rule::Seq(vec![
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesBlock))),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Node(SyntaxKind::GraphPatternNotTriples),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::Dot))),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesBlock))),
                ]))),
            ])),
            SyntaxKind::TriplesBlock => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::TriplesSameSubjectPath),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Dot),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::TriplesBlock))),
                ]))),
            ])),
            SyntaxKind::GraphPatternNotTriples => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::GroupOrUnionGraphPattern),
                Rule::Node(SyntaxKind::OptionalGraphPattern),
                Rule::Node(SyntaxKind::MinusGraphPattern),
                Rule::Node(SyntaxKind::GraphGraphPattern),
                Rule::Node(SyntaxKind::ServiceGraphPattern),
                Rule::Node(SyntaxKind::Filter),
                Rule::Node(SyntaxKind::Bind),
                Rule::Node(SyntaxKind::InlineData),
            ])),
            SyntaxKind::TriplesSameSubjectPath => Some(Rule::Alt(vec![
                Rule::Seq(vec![
                    Rule::Node(SyntaxKind::VarOrTerm),
                    Rule::Node(SyntaxKind::PropertyListPathNotEmpty),
                ]),
                Rule::Seq(vec![
                    Rule::Node(SyntaxKind::TriplesNodePath),
                    Rule::Node(SyntaxKind::PropertyListPath),
                ]),
            ])),
            SyntaxKind::GroupOrUnionGraphPattern => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::GroupGraphPattern),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::UNION),
                    Rule::Node(SyntaxKind::GroupGraphPattern),
                ]))),
            ])),
            SyntaxKind::OptionalGraphPattern => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::OPTIONAL),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::MinusGraphPattern => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::MINUS),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::GraphGraphPattern => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::GRAPH),
                Rule::Node(SyntaxKind::VarOrIri),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::ServiceGraphPattern => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::SERVICE),
                Rule::Opt(Box::new(Rule::Token(SyntaxKind::SILENT))),
                Rule::Node(SyntaxKind::VarOrIri),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::Filter => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::FILTER),
                Rule::Node(SyntaxKind::Constraint),
            ])),
            SyntaxKind::Bind => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::BIND),
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::Expression),
                Rule::Token(SyntaxKind::AS),
                Rule::Node(SyntaxKind::Var),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::InlineData => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::VALUES),
                Rule::Node(SyntaxKind::DataBlock),
            ])),
            SyntaxKind::InlineDataOneVar => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::Var),
                Rule::Token(SyntaxKind::LCurly),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::DataBlockValue))),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::InlineDataFull => Some(Rule::Seq(vec![
                Rule::Alt(vec![
                    Rule::Token(SyntaxKind::NIL),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::LParen),
                        Rule::Rep(Box::new(Rule::Node(SyntaxKind::Var))),
                        Rule::Token(SyntaxKind::RParen),
                    ]),
                ]),
                Rule::Token(SyntaxKind::LCurly),
                Rule::Rep(Box::new(Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::LParen),
                        Rule::Rep(Box::new(Rule::Node(SyntaxKind::DataBlockValue))),
                        Rule::Token(SyntaxKind::RParen),
                    ]),
                    Rule::Token(SyntaxKind::NIL),
                ]))),
                Rule::Token(SyntaxKind::RCurly),
            ])),
            SyntaxKind::DataBlockValue => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::iri),
                Rule::Node(SyntaxKind::RDFLiteral),
                Rule::Node(SyntaxKind::NumericLiteral),
                Rule::Node(SyntaxKind::BooleanLiteral),
                Rule::Token(SyntaxKind::UNDEF),
            ])),
            SyntaxKind::RDFLiteral => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::String),
                Rule::Opt(Box::new(Rule::Alt(vec![
                    Rule::Token(SyntaxKind::LANGTAG),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::DoubleZirkumflex),
                        Rule::Node(SyntaxKind::iri),
                    ]),
                ]))),
            ])),
            SyntaxKind::NumericLiteral => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::NumericLiteralUnsigned),
                Rule::Node(SyntaxKind::NumericLiteralPositive),
                Rule::Node(SyntaxKind::NumericLiteralNegative),
            ])),
            SyntaxKind::BooleanLiteral => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::True),
                Rule::Token(SyntaxKind::False),
            ])),
            SyntaxKind::ArgList => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::NIL),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Rep(Box::new(Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Comma),
                        Rule::Node(SyntaxKind::Expression),
                    ]))),
                    Rule::Token(SyntaxKind::RParen),
                ]),
            ])),
            SyntaxKind::ExpressionList => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::NIL),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Rep(Box::new(Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Comma),
                        Rule::Node(SyntaxKind::Expression),
                    ]))),
                    Rule::Token(SyntaxKind::RParen),
                ]),
            ])),
            SyntaxKind::ConstructTriples => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::TriplesSameSubject),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Dot),
                    Rule::Opt(Box::new(Rule::Node(SyntaxKind::ConstructTriples))),
                ]))),
            ])),
            SyntaxKind::VarOrTerm => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::Var),
                Rule::Node(SyntaxKind::GraphTerm),
            ])),
            SyntaxKind::PropertyListNotEmpty => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::Verb),
                Rule::Node(SyntaxKind::ObjectList),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Semicolon),
                    Rule::Opt(Box::new(Rule::Seq(vec![
                        Rule::Node(SyntaxKind::Verb),
                        Rule::Node(SyntaxKind::ObjectList),
                    ]))),
                ]))),
            ])),
            SyntaxKind::TriplesNode => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::Collection),
                Rule::Node(SyntaxKind::BlankNodePropertyList),
            ])),
            SyntaxKind::PropertyList => Some(Rule::Opt(Box::new(Rule::Node(
                SyntaxKind::PropertyListNotEmpty,
            )))),
            SyntaxKind::Verb => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::VarOrIri),
                Rule::Token(SyntaxKind::a),
            ])),
            SyntaxKind::ObjectList => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::Object),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Object),
                ]))),
            ])),
            SyntaxKind::Object => Some(Rule::Node(SyntaxKind::GraphNode)),
            SyntaxKind::GraphNode => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::VarOrTerm),
                Rule::Node(SyntaxKind::TriplesNode),
            ])),
            SyntaxKind::PropertyListPathNotEmpty => Some(Rule::Seq(vec![
                Rule::Alt(vec![
                    Rule::Node(SyntaxKind::VerbPath),
                    Rule::Node(SyntaxKind::VerbSimple),
                ]),
                Rule::Node(SyntaxKind::ObjectListPath),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Semicolon),
                    Rule::Opt(Box::new(Rule::Seq(vec![
                        Rule::Alt(vec![
                            Rule::Node(SyntaxKind::VerbPath),
                            Rule::Node(SyntaxKind::VerbSimple),
                        ]),
                        Rule::Node(SyntaxKind::ObjectList),
                    ]))),
                ]))),
            ])),
            SyntaxKind::TriplesNodePath => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::CollectionPath),
                Rule::Node(SyntaxKind::BlankNodePropertyListPath),
            ])),
            SyntaxKind::PropertyListPath => Some(Rule::Opt(Box::new(Rule::Node(
                SyntaxKind::PropertyListPathNotEmpty,
            )))),
            SyntaxKind::VerbPath => Some(Rule::Node(SyntaxKind::Path)),
            SyntaxKind::VerbSimple => Some(Rule::Node(SyntaxKind::Var)),
            SyntaxKind::ObjectListPath => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::ObjectPath),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::ObjectPath),
                ]))),
            ])),
            SyntaxKind::Path => Some(Rule::Node(SyntaxKind::PathAlternative)),
            SyntaxKind::ObjectPath => Some(Rule::Node(SyntaxKind::GraphNodePath)),
            SyntaxKind::GraphNodePath => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::VarOrTerm),
                Rule::Node(SyntaxKind::TriplesNodePath),
            ])),
            SyntaxKind::PathAlternative => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::PathSequence),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Pipe),
                    Rule::Node(SyntaxKind::PathSequence),
                ]))),
            ])),
            SyntaxKind::PathSequence => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::PathEltOrInverse),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Slash),
                    Rule::Node(SyntaxKind::PathEltOrInverse),
                ]))),
            ])),
            SyntaxKind::PathEltOrInverse => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::PathElt),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Zirkumflex),
                    Rule::Node(SyntaxKind::PathElt),
                ]),
            ])),
            SyntaxKind::PathElt => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::PathPrimary),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::PathMod))),
            ])),
            SyntaxKind::PathPrimary => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::iri),
                Rule::Token(SyntaxKind::a),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::ExclamationMark),
                    Rule::Node(SyntaxKind::PathNegatedPropertySet),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Node(SyntaxKind::Path),
                    Rule::Token(SyntaxKind::RParen),
                ]),
            ])),
            SyntaxKind::PathMod => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::QuestionMark),
                Rule::Token(SyntaxKind::Star),
                Rule::Token(SyntaxKind::Plus),
            ])),
            SyntaxKind::PathNegatedPropertySet => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::PathOneInPropertySet),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Seq(vec![
                        Rule::Node(SyntaxKind::PathOneInPropertySet),
                        Rule::Rep(Box::new(Rule::Seq(vec![
                            Rule::Token(SyntaxKind::Pipe),
                            Rule::Node(SyntaxKind::PathOneInPropertySet),
                        ]))),
                    ]))),
                    Rule::Token(SyntaxKind::RParen),
                ]),
            ])),
            SyntaxKind::PathOneInPropertySet => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::iri),
                Rule::Token(SyntaxKind::a),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Zirkumflex),
                    Rule::Alt(vec![
                        Rule::Node(SyntaxKind::iri),
                        Rule::Token(SyntaxKind::a),
                    ]),
                ]),
            ])),
            SyntaxKind::Integer => Some(Rule::Token(SyntaxKind::INTEGER)),
            SyntaxKind::Collection => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::GraphNode),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::GraphNode))),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::BlankNodePropertyList => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LBrack),
                Rule::Node(SyntaxKind::PropertyListNotEmpty),
                Rule::Token(SyntaxKind::RBrack),
            ])),
            SyntaxKind::CollectionPath => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::GraphNodePath),
                Rule::Rep(Box::new(Rule::Node(SyntaxKind::GraphNodePath))),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::BlankNodePropertyListPath => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::LBrack),
                Rule::Node(SyntaxKind::PropertyListPathNotEmpty),
                Rule::Token(SyntaxKind::RBrack),
            ])),
            SyntaxKind::GraphTerm => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::iri),
                Rule::Node(SyntaxKind::RDFLiteral),
                Rule::Node(SyntaxKind::NumericLiteral),
                Rule::Node(SyntaxKind::BooleanLiteral),
                Rule::Node(SyntaxKind::BlankNode),
                Rule::Token(SyntaxKind::NIL),
            ])),
            SyntaxKind::BlankNode => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::BLANK_NODE_LABEL),
                Rule::Token(SyntaxKind::ANON),
            ])),
            SyntaxKind::ConditionalOrExpression => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::ConditionalAndExpression),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::DoublePipe),
                    Rule::Node(SyntaxKind::ConditionalAndExpression),
                ]))),
            ])),
            SyntaxKind::ConditionalAndExpression => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::ValueLogical),
                Rule::Rep(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::DoubleAnd),
                    Rule::Node(SyntaxKind::ValueLogical),
                ]))),
            ])),
            SyntaxKind::ValueLogical => Some(Rule::Node(SyntaxKind::RelationalExpression)),
            SyntaxKind::RelationalExpression => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::NumericExpression),
                Rule::Opt(Box::new(Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Equals),
                        Rule::Node(SyntaxKind::NumericExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::ExclamationMarkEquals),
                        Rule::Node(SyntaxKind::NumericExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Less),
                        Rule::Node(SyntaxKind::NumericExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::More),
                        Rule::Node(SyntaxKind::NumericExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::LessEquals),
                        Rule::Node(SyntaxKind::NumericExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::MoreEquals),
                        Rule::Node(SyntaxKind::NumericExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::IN),
                        Rule::Node(SyntaxKind::ExpressionList),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::NOT),
                        Rule::Token(SyntaxKind::IN),
                        Rule::Node(SyntaxKind::ExpressionList),
                    ]),
                ]))),
            ])),
            SyntaxKind::NumericExpression => Some(Rule::Node(SyntaxKind::AdditiveExpression)),
            SyntaxKind::AdditiveExpression => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::MultiplicativeExpression),
                Rule::Rep(Box::new(Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Plus),
                        Rule::Node(SyntaxKind::MultiplicativeExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Minus),
                        Rule::Node(SyntaxKind::MultiplicativeExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Alt(vec![
                            Rule::Node(SyntaxKind::NumericLiteralPositive),
                            Rule::Node(SyntaxKind::NumericLiteralNegative),
                        ]),
                        Rule::Rep(Box::new(Rule::Alt(vec![
                            Rule::Seq(vec![
                                Rule::Token(SyntaxKind::Star),
                                Rule::Node(SyntaxKind::UnaryExpression),
                            ]),
                            Rule::Seq(vec![
                                Rule::Token(SyntaxKind::Slash),
                                Rule::Node(SyntaxKind::UnaryExpression),
                            ]),
                        ]))),
                    ]),
                ]))),
            ])),
            SyntaxKind::MultiplicativeExpression => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::UnaryExpression),
                Rule::Rep(Box::new(Rule::Alt(vec![
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Star),
                        Rule::Node(SyntaxKind::UnaryExpression),
                    ]),
                    Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Slash),
                        Rule::Node(SyntaxKind::UnaryExpression),
                    ]),
                ]))),
            ])),
            SyntaxKind::NumericLiteralPositive => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::INTEGER_POSITIVE),
                Rule::Token(SyntaxKind::DECIMAL_POSITIVE),
                Rule::Token(SyntaxKind::DOUBLE_POSITIVE),
            ])),
            SyntaxKind::NumericLiteralNegative => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::INTEGER_NEGATIVE),
                Rule::Token(SyntaxKind::DECIMAL_NEGATIVE),
                Rule::Token(SyntaxKind::DOUBLE_NEGATIVE),
            ])),
            SyntaxKind::UnaryExpression => Some(Rule::Alt(vec![
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::ExclamationMark),
                    Rule::Node(SyntaxKind::PrimaryExpression),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Plus),
                    Rule::Node(SyntaxKind::PrimaryExpression),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Minus),
                    Rule::Node(SyntaxKind::PrimaryExpression),
                ]),
                Rule::Node(SyntaxKind::PrimaryExpression),
            ])),
            SyntaxKind::PrimaryExpression => Some(Rule::Alt(vec![
                Rule::Node(SyntaxKind::BrackettedExpression),
                Rule::Node(SyntaxKind::BuiltInCall),
                Rule::Node(SyntaxKind::iriOrFunction),
                Rule::Node(SyntaxKind::RDFLiteral),
                Rule::Node(SyntaxKind::NumericLiteral),
                Rule::Node(SyntaxKind::BooleanLiteral),
                Rule::Node(SyntaxKind::Var),
            ])),
            SyntaxKind::iriOrFunction => Some(Rule::Seq(vec![
                Rule::Node(SyntaxKind::iri),
                Rule::Opt(Box::new(Rule::Node(SyntaxKind::ArgList))),
            ])),
            SyntaxKind::Aggregate => Some(Rule::Alt(vec![
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::COUNT),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Alt(vec![
                        Rule::Token(SyntaxKind::Star),
                        Rule::Node(SyntaxKind::Expression),
                    ]),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SUM),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::MIN),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::MAX),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::AVG),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::SAMPLE),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Token(SyntaxKind::RParen),
                ]),
                Rule::Seq(vec![
                    Rule::Token(SyntaxKind::GROUP_CONCAT),
                    Rule::Token(SyntaxKind::LParen),
                    Rule::Opt(Box::new(Rule::Token(SyntaxKind::DISTINCT))),
                    Rule::Node(SyntaxKind::Expression),
                    Rule::Opt(Box::new(Rule::Seq(vec![
                        Rule::Token(SyntaxKind::Semicolon),
                        Rule::Token(SyntaxKind::SEPARATOR),
                        Rule::Token(SyntaxKind::Equals),
                        Rule::Node(SyntaxKind::String),
                    ]))),
                    Rule::Token(SyntaxKind::RParen),
                ]),
            ])),
            SyntaxKind::SubstringExpression => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::SUBSTR),
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::Expression),
                Rule::Token(SyntaxKind::Comma),
                Rule::Node(SyntaxKind::Expression),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                ]))),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::StrReplaceExpression => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::REPLACE),
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::Expression),
                Rule::Token(SyntaxKind::Comma),
                Rule::Node(SyntaxKind::Expression),
                Rule::Token(SyntaxKind::Comma),
                Rule::Node(SyntaxKind::Expression),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                ]))),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::RegexExpression => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::REGEX),
                Rule::Token(SyntaxKind::LParen),
                Rule::Node(SyntaxKind::Expression),
                Rule::Token(SyntaxKind::Comma),
                Rule::Node(SyntaxKind::Expression),
                Rule::Opt(Box::new(Rule::Seq(vec![
                    Rule::Token(SyntaxKind::Comma),
                    Rule::Node(SyntaxKind::Expression),
                ]))),
                Rule::Token(SyntaxKind::RParen),
            ])),
            SyntaxKind::ExistsFunc => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::EXISTS),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::NotExistsFunc => Some(Rule::Seq(vec![
                Rule::Token(SyntaxKind::NOT),
                Rule::Token(SyntaxKind::EXISTS),
                Rule::Node(SyntaxKind::GroupGraphPattern),
            ])),
            SyntaxKind::String => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::STRING_LITERAL1),
                Rule::Token(SyntaxKind::STRING_LITERAL2),
                Rule::Token(SyntaxKind::STRING_LITERAL_LONG1),
                Rule::Token(SyntaxKind::STRING_LITERAL_LONG2),
            ])),
            SyntaxKind::NumericLiteralUnsigned => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::INTEGER),
                Rule::Token(SyntaxKind::DECIMAL),
                Rule::Token(SyntaxKind::DOUBLE),
            ])),
            SyntaxKind::PrefixedName => Some(Rule::Alt(vec![
                Rule::Token(SyntaxKind::PNAME_LN),
                Rule::Token(SyntaxKind::PNAME_NS),
            ])),
            _ => None,
        }
    }
}
