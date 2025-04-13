use logos::Logos;

#[allow(non_camel_case_types)]
#[derive(Logos, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
#[repr(u16)]
pub enum SyntaxKind {
    Eof = 0,
    Error,
    #[regex(r#"[ \t\n\f]+"#)]
    WHITESPACE,
    #[token("BASE", ignore(case))]
    BASE,
    #[regex(r#"<[^<>\"{}|^`\\\u{00}-\u{20}]*>"#)]
    IRIREF,
    #[token("PREFIX", ignore(case))]
    PREFIX,
    #[regex("[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}]([A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}_.-]*[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}_-])?:")]
    PNAME_NS,
    #[token("SELECT", ignore(case))]
    SELECT,
    #[token("DISTINCT", ignore(case))]
    DISTINCT,
    #[token("REDUCED", ignore(case))]
    REDUCED,
    #[token("(")]
    LParen,
    #[token("AS", ignore(case))]
    AS,
    #[token(")")]
    RParen,
    #[token("*")]
    Star,
    #[token("CONSTRUCT", ignore(case))]
    CONSTRUCT,
    #[token("WHERE", ignore(case))]
    WHERE,
    #[token("{")]
    LCurly,
    #[token("}")]
    RCurly,
    #[token("DESCRIBE", ignore(case))]
    DESCRIBE,
    #[token("ASK", ignore(case))]
    ASK,
    #[token("FROM", ignore(case))]
    FROM,
    #[token("NAMED", ignore(case))]
    NAMED,
    #[token("GROUP", ignore(case))]
    GROUP,
    #[token("BY", ignore(case))]
    BY,
    #[token("HAVING", ignore(case))]
    HAVING,
    #[token("ORDER", ignore(case))]
    ORDER,
    #[token("ASC", ignore(case))]
    ASC,
    #[token("DESC", ignore(case))]
    DESC,
    #[token("LIMIT", ignore(case))]
    LIMIT,
    #[token("OFFSET", ignore(case))]
    OFFSET,
    #[token("VALUES", ignore(case))]
    VALUES,
    #[token(";")]
    Semicolon,
    #[token("LOAD", ignore(case))]
    LOAD,
    #[token("SILENT", ignore(case))]
    SILENT,
    #[token("INTO", ignore(case))]
    INTO,
    #[token("CLEAR", ignore(case))]
    CLEAR,
    #[token("DROP", ignore(case))]
    DROP,
    #[token("CREATE", ignore(case))]
    CREATE,
    #[token("ADD", ignore(case))]
    ADD,
    #[token("TO", ignore(case))]
    TO,
    #[token("MOVE", ignore(case))]
    MOVE,
    #[token("COPY", ignore(case))]
    COPY,
    #[token("INSERT", ignore(case))]
    INSERT,
    #[token("DATA", ignore(case))]
    DATA,
    INSERT_DATA,
    #[regex(r"(?i)DELETE\s+DATA")]
    DELETE_DATA,
    #[token(r"(?i)DELETE\s+WHERE")]
    DELETE_WHERE,
    #[token("DELETE", ignore(case))]
    DELETE,
    #[token("WITH", ignore(case))]
    WITH,
    #[token("USING", ignore(case))]
    USING,
    #[token("DEFAULT", ignore(case))]
    DEFAULT,
    #[token("GRAPH", ignore(case))]
    GRAPH,
    #[token("ALL", ignore(case))]
    ALL,
    #[token(".")]
    Dot,
    #[token("OPTIONAL", ignore(case))]
    OPTIONAL,
    #[token("SERVICE", ignore(case))]
    SERVICE,
    #[token("BIND", ignore(case))]
    BIND,
    #[token("NIL", ignore(case))]
    NIL,
    #[token("UNDEF", ignore(case))]
    UNDEF,
    #[token("MINUS", ignore(case))]
    MINUS,
    #[token("UNION", ignore(case))]
    UNION,
    #[token("FILTER", ignore(case))]
    FILTER,
    #[token(",")]
    Colon,
    #[regex(r" a ")]
    a,
    #[token("|")]
    Pipe,
    #[token("/")]
    Slash,
    #[token("^")]
    Zirkumflex,
    #[token("?")]
    QuestionMark,
    #[token("+")]
    Plus,
    #[token("!")]
    ExclamationMark,
    #[token("[")]
    LBrack,
    #[token("]")]
    RBrack,
    // (?:(?:(?:[A-Z]|[a-z]|[\u{00C0}-\u{00D6}]|[\u{00D8}-\u{00F6}]|[\u{00F8}-\u{02FF}]|[\u{0370}-\u{037D}]|[\u{037F}-\u{1FFF}]|[\u{200C}-\u{200D}]|[\u{2070}-\u{218F}]|[\u{2C00}-\u{2FEF}]|[\u{3001}-\u{D7FF}]|[\u{F900}-\u{FDCF}]|[\u{FDF0}-\u{FFFD}]|[\u{10000}-\u{EFFFF}])|_)|[0-9])(?:(?:(?:[A-Z]|[a-z]|[\u{00C0}-\u{00D6}]|[\u{00D8}-\u{00F6}]|[\u{00F8}-\u{02FF}]|[\u{0370}-\u{037D}]|[\u{037F}-\u{1FFF}]|[\u{200C}-\u{200D}]|[\u{2070}-\u{218F}]|[\u{2C00}-\u{2FEF}]|[\u{3001}-\u{D7FF}]|[\u{F900}-\u{FDCF}]|[\u{FDF0}-\u{FFFD}]|[\u{10000}-\u{EFFFF}])|_)|[0-9]|\u{00B7}|[\u{0300}-\u{036F}]|[\u{203F}-\u{2040}])*
    #[regex(
        r"\?(?:(?:(?:[A-Z]|[a-z]|[\u{00C0}-\u{00D6}]|[\u{00D8}-\u{00F6}]|[\u{00F8}-\u{02FF}]|[\u{0370}-\u{037D}]|[\u{037F}-\u{1FFF}]|[\u{200C}-\u{200D}]|[\u{2070}-\u{218F}]|[\u{2C00}-\u{2FEF}]|[\u{3001}-\u{D7FF}]|[\u{F900}-\u{FDCF}]|[\u{FDF0}-\u{FFFD}]|[\u{10000}-\u{EFFFF}])|_)|[0-9])(?:(?:(?:[A-Z]|[a-z]|[\u{00C0}-\u{00D6}]|[\u{00D8}-\u{00F6}]|[\u{00F8}-\u{02FF}]|[\u{0370}-\u{037D}]|[\u{037F}-\u{1FFF}]|[\u{200C}-\u{200D}]|[\u{2070}-\u{218F}]|[\u{2C00}-\u{2FEF}]|[\u{3001}-\u{D7FF}]|[\u{F900}-\u{FDCF}]|[\u{FDF0}-\u{FFFD}]|[\u{10000}-\u{EFFFF}])|_)|[0-9]|\u{00B7}|[\u{0300}-\u{036F}]|[\u{203F}-\u{2040}])*"
    )]
    VAR1,
    #[regex(
        r"\$(?:(?:(?:[A-Z]|[a-z]|[\u{00C0}-\u{00D6}]|[\u{00D8}-\u{00F6}]|[\u{00F8}-\u{02FF}]|[\u{0370}-\u{037D}]|[\u{037F}-\u{1FFF}]|[\u{200C}-\u{200D}]|[\u{2070}-\u{218F}]|[\u{2C00}-\u{2FEF}]|[\u{3001}-\u{D7FF}]|[\u{F900}-\u{FDCF}]|[\u{FDF0}-\u{FFFD}]|[\u{10000}-\u{EFFFF}])|_)|[0-9])(?:(?:(?:[A-Z]|[a-z]|[\u{00C0}-\u{00D6}]|[\u{00D8}-\u{00F6}]|[\u{00F8}-\u{02FF}]|[\u{0370}-\u{037D}]|[\u{037F}-\u{1FFF}]|[\u{200C}-\u{200D}]|[\u{2070}-\u{218F}]|[\u{2C00}-\u{2FEF}]|[\u{3001}-\u{D7FF}]|[\u{F900}-\u{FDCF}]|[\u{FDF0}-\u{FFFD}]|[\u{10000}-\u{EFFFF}])|_)|[0-9]|\u{00B7}|[\u{0300}-\u{036F}]|[\u{203F}-\u{2040}])*"
    )]
    VAR2,
    #[token("||")]
    DoublePipe,
    #[token("&&")]
    DoubleAnd,
    #[token("=")]
    Equals,
    #[token("!=")]
    ExclamationMarkEquals,
    #[token("<")]
    Less,
    #[token(">")]
    More,
    #[token("<=")]
    LessEquals,
    #[token(">=")]
    MoreEquals,
    #[token("IN")]
    IN,
    #[token("NOT")]
    NOT,
    #[token("-")]
    Minus,
    #[token("STR")]
    STR,
    #[token("LANG")]
    LANG,
    #[token("LANGMATCHES")]
    LANGMATCHES,
    #[token("DATATYPE")]
    DATATYPE,
    #[token("BOUND")]
    BOUND,
    #[token("IRI")]
    IRI,
    #[token("URI")]
    URI,
    #[token("BNODE")]
    BNODE,
    #[token("RAND")]
    RAND,
    #[token("ABS")]
    ABS,
    #[token("CEIL")]
    CEIL,
    #[token("FLOOR")]
    FLOOR,
    #[token("ROUND")]
    ROUND,
    #[token("CONCAT")]
    CONCAT,
    #[token("STRLEN")]
    STRLEN,
    #[token("UCASE")]
    UCASE,
    #[token("LCASE")]
    LCASE,
    #[token("ENCODE_FOR_URI")]
    ENCODE_FOR_URI,
    #[token("CONTAINS")]
    CONTAINS,
    #[token("STRSTARTS")]
    STRSTARTS,
    #[token("STRENDS")]
    STRENDS,
    #[token("STRBEFORE")]
    STRBEFORE,
    #[token("STRAFTER")]
    STRAFTER,
    #[token("YEAR")]
    YEAR,
    #[token("MONTH")]
    MONTH,
    #[token("DAY")]
    DAY,
    #[token("HOURS")]
    HOURS,
    #[token("MINUTES")]
    MINUTES,
    #[token("SECONDS")]
    SECONDS,
    #[token("TIMEZONE")]
    TIMEZONE,
    #[token("TZ")]
    TZ,
    #[token("NOW")]
    NOW,
    #[token("UUID")]
    UUID,
    #[token("STRUUID")]
    STRUUID,
    #[token("MD5")]
    MD5,
    #[token("SHA1")]
    SHA1,
    #[token("SHA256")]
    SHA256,
    #[token("SHA384")]
    SHA384,
    #[token("SHA512")]
    SHA512,
    #[token("COALESCE")]
    COALESCE,
    #[token("IF")]
    IF,
    #[token("STRLANG")]
    STRLANG,
    #[token("STRDT")]
    STRDT,
    #[token("sameTerm")]
    sameTerm,
    #[token("isIRI")]
    isIRI,
    #[token("isURI")]
    isURI,
    #[token("isBLANK")]
    isBLANK,
    #[token("isLITERAL")]
    isLITERAL,
    #[token("isNUMERIC")]
    isNUMERIC,
    #[token("REGEX")]
    REGEX,
    #[token("SUBSTR")]
    SUBSTR,
    #[token("REPLACE")]
    REPLACE,
    #[token("EXISTS")]
    EXISTS,
    #[token("COUNT")]
    COUNT,
    #[token("SUM")]
    SUM,
    #[token("MIN")]
    MIN,
    #[token("MAX")]
    MAX,
    #[token("AVG")]
    AVG,
    #[token("SAMPLE")]
    SAMPLE,
    #[token("GROUP_CONCAT")]
    GROUP_CONCAT,
    #[token("SEPARATOR")]
    SEPARATOR,
    #[token("LANGTAG")]
    LANGTAG,
    #[token("DoubleZirkumflex")]
    DoubleZirkumflex,
    #[regex(r"\d+")]
    INTEGER,
    #[regex(r"\d*\.\d+")]
    DECIMAL,
    #[regex(r"(?:\d+\.\d*(?:[eE][+-]?\d+))|(?:\.\d+([eE][+-]?\d+))|(?:\d+([eE][+-]?\d+))")]
    DOUBLE,
    #[regex(r"\+\d+")]
    INTEGER_POSITIVE,
    #[regex(r"\+\d*\.\d+")]
    DECIMAL_POSITIVE,
    #[regex(r"\+(?:\d+\.\d*(?:[eE][+-]?\d+))|\+(?:\.\d+([eE][+-]?\d+))|\+(?:\d+([eE][+-]?\d+))")]
    DOUBLE_POSITIVE,
    #[regex(r"-\d+")]
    INTEGER_NEGATIVE,
    #[regex(r"-\d*\.\d+")]
    DECIMAL_NEGATIVE,
    #[regex(r"-(?:\d+\.\d*(?:[eE][+-]?\d+))|-(?:\.\d+([eE][+-]?\d+))|-(?:\d+([eE][+-]?\d+))")]
    DOUBLE_NEGATIVE,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[regex("'[^\u{27}\u{5C}\u{A}\u{D}]*'")]
    STRING_LITERAL1,
    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#)]
    STRING_LITERAL2,
    // TODO: add regex
    #[token("STRING_LITERAL_LONG1")]
    STRING_LITERAL_LONG1,
    // TODO: add regex
    #[token("STRING_LITERAL_LONG2")]
    STRING_LITERAL_LONG2,
    #[regex(r"[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}](?:[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}_.-]*[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}_-])?:(?:[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}_0-9:]|%[0-9A-Fa-f])(?:[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}_:\.-]|%[0-9A-Fa-f])*(?:[A-Za-z\u{00C0}-\u{00D6}\u{00D8}-\u{00F6}\u{00F8}-\u{02FF}\u{0370}-\u{037D}\u{037F}-\u{1FFF}\u{200C}-\u{200D}\u{2070}-\u{218F}\u{2C00}-\u{2FEF}\u{3001}-\u{D7FF}\u{F900}-\u{FDCF}\u{FDF0}-\u{FFFD}\u{10000}-\u{EFFFF}0-9\u{00B7}\u{0300}-\u{036F}\u{203F}-\u{2040}_:-]|%[0-9A-Fa-f])?")]
    PNAME_LN,
    #[token("BLANK_NODE_LABEL")]
    BLANK_NODE_LABEL,
    #[regex(r"\[\s*\]")]
    ANON,

    // Composite nodes
    /// QueryUnit => Query
    QueryUnit,
    /// Query => Prologue (SelectQuery | ConstructQuery | DescribeQuery | AskQuery) ValuesClause
    Query,
    /// Prologue => (BaseDecl | PrefixDecl)*
    Prologue,
    /// SelectQuery => SelectClause DatasetClause* WhereClause SolutionModifier
    SelectQuery,
    /// ConstructQuery => 'CONSTRUCT' (ConstructTemplate DatasetClause* WhereClause SolutionModifier | DatasetClause* 'WHERE' '{' TriplesTemplate? '}' SolutionModifier)
    ConstructQuery,
    /// DescribeQuery => 'DESCRIBE' (VarOrIri VarOrIri* | '*') DatasetClause* WhereClause? SolutionModifier
    DescribeQuery,
    /// AskQuery => 'ASK' DatasetClause* WhereClause SolutionModifier
    AskQuery,
    /// ValuesClause => ('VALUES' DataBlock)?
    ValuesClause,
    /// UpdateUnit => Update
    UpdateUnit,
    /// Update => Prologue (UpdateOne (';' Update)?)?
    Update,
    /// BaseDecl => 'BASE' 'IRIREF'
    BaseDecl,
    /// PrefixDecl => 'PREFIX' 'PNAME_NS' 'IRIREF'
    PrefixDecl,
    /// SelectClause => 'SELECT' ('DISTINCT' | 'REDUCED')? ((Var | '(' Expression 'AS' Var ')') (Var | '(' Expression 'AS' Var ')')* | '*')
    SelectClause,
    /// DatasetClause => 'FROM' (DefaultGraphClause | NamedGraphClause)
    DatasetClause,
    /// WhereClause => 'WHERE'? GroupGraphPattern
    WhereClause,
    /// SolutionModifier => GroupClause? HavingClause? OrderClause? LimitOffsetClauses?
    SolutionModifier,
    /// SubSelect => SelectClause WhereClause SolutionModifier ValuesClause
    SubSelect,
    /// Var => 'VAR1' | 'VAR2'
    Var,
    /// Expression => ConditionalOrExpression
    Expression,
    /// ConstructTemplate => '{' ConstructTriples? '}'
    ConstructTemplate,
    /// TriplesTemplate => TriplesSameSubject ('.' TriplesTemplate?)?
    TriplesTemplate,
    /// VarOrIri => Var | iri
    VarOrIri,
    /// DefaultGraphClause => SourceSelector
    DefaultGraphClause,
    /// NamedGraphClause => 'NAMED' SourceSelector
    NamedGraphClause,
    /// SourceSelector => iri
    SourceSelector,
    /// iri => 'IRIREF' | PrefixedName
    iri,
    /// GroupGraphPattern => '{' (SubSelect | GroupGraphPatternSub) '}'
    GroupGraphPattern,
    /// GroupClause => 'GROUP' 'BY' GroupCondition GroupCondition*
    GroupClause,
    /// HavingClause => 'HAVING' HavingCondition HavingCondition*
    HavingClause,
    /// OrderClause => 'ORDER' 'BY' OrderCondition OrderCondition*
    OrderClause,
    /// LimitOffsetClauses => LimitClause OffsetClause? | OffsetClause LimitClause?
    LimitOffsetClauses,
    /// GroupCondition => BuiltInCall | FunctionCall | '(' Expression ('AS' Var)? ')' | Var
    GroupCondition,
    /// BuiltInCall => Aggregate | 'STR' '(' Expression ')' | 'LANG' '(' Expression ')' | 'LANGMATCHES' '(' Expression ',' Expression ')' | 'DATATYPE' '(' Expression ')' | 'BOUND' '(' Var ')' | 'IRI' '(' Expression ')' | 'URI' '(' Expression ')' | 'BNODE' ('(' Expression ')' | 'NIL') | 'RAND' 'NIL' | 'ABS' '(' Expression ')' | 'CEIL' '(' Expression ')' | 'FLOOR' '(' Expression ')' | 'ROUND' '(' Expression ')' | 'CONCAT' ExpressionList | SubstringExpression | 'STRLEN' '(' Expression ')' | StrReplaceExpression | 'UCASE' '(' Expression ')' | 'LCASE' '(' Expression ')' | 'ENCODE_FOR_URI' '(' Expression ')' | 'CONTAINS' '(' Expression ',' Expression ')' | 'STRSTARTS' '(' Expression ',' Expression ')' | 'STRENDS' '(' Expression ',' Expression ')' | 'STRBEFORE' '(' Expression ',' Expression ')' | 'STRAFTER' '(' Expression ',' Expression ')' | 'YEAR' '(' Expression ')' | 'MONTH' '(' Expression ')' | 'DAY' '(' Expression ')' | 'HOURS' '(' Expression ')' | 'MINUTES' '(' Expression ')' | 'SECONDS' '(' Expression ')' | 'TIMEZONE' '(' Expression ')' | 'TZ' '(' Expression ')' | 'NOW' 'NIL' | 'UUID' 'NIL' | 'STRUUID' 'NIL' | 'MD5' '(' Expression ')' | 'SHA1' '(' Expression ')' | 'SHA256' '(' Expression ')' | 'SHA384' '(' Expression ')' | 'SHA512' '(' Expression ')' | 'COALESCE' ExpressionList | 'IF' '(' Expression ',' Expression ',' Expression ')' | 'STRLANG' '(' Expression ',' Expression ')' | 'STRDT' '(' Expression ',' Expression ')' | 'sameTerm' '(' Expression ',' Expression ')' | 'isIRI' '(' Expression ')' | 'isURI' '(' Expression ')' | 'isBLANK' '(' Expression ')' | 'isLITERAL' '(' Expression ')' | 'isNUMERIC' '(' Expression ')' | RegexExpression | ExistsFunc | NotExistsFunc
    BuiltInCall,
    /// FunctionCall => iri ArgList
    FunctionCall,
    /// HavingCondition => Constraint
    HavingCondition,
    /// Constraint => BrackettedExpression | BuiltInCall | FunctionCall
    Constraint,
    /// OrderCondition => ('ASC' | 'DESC') BrackettedExpression | Constraint | Var
    OrderCondition,
    /// BrackettedExpression => '(' Expression ')'
    BrackettedExpression,
    /// LimitClause => 'LIMIT' 'INTEGER'
    LimitClause,
    /// OffsetClause => 'OFFSET' 'INTEGER'
    OffsetClause,
    /// DataBlock => InlineDataOneVar | InlineDataFull
    DataBlock,
    /// UpdateOne => Load | Clear | Drop | Add | Move | Copy | Create | InsertData | DeleteData | DeleteWhere | Modify
    UpdateOne,
    /// Load => 'LOAD' 'SILENT'? iri ('INTO' GraphRef)?
    Load,
    /// Clear => 'CLEAR' 'SILENT'? GraphRefAll
    Clear,
    /// Drop => 'DROP' 'SILENT'? GraphRefAll
    Drop,
    /// Add => 'ADD' 'SILENT'? GraphOrDefault 'TO' GraphOrDefault
    Add,
    /// Move => 'MOVE' 'SILENT'? GraphOrDefault 'TO' GraphOrDefault
    Move,
    /// Copy => 'COPY' 'SILENT'? GraphOrDefault 'TO' GraphOrDefault
    Copy,
    /// Create => 'CREATE' 'SILENT'? GraphRef
    Create,
    /// InsertData => 'INSERT_DATA' QuadData
    InsertData,
    /// DeleteData => 'DELETE_DATA' QuadData
    DeleteData,
    /// DeleteWhere => 'DELETE_WHERE' QuadPattern
    DeleteWhere,
    /// Modify => ('WITH' iri)? (DeleteClause InsertClause? | InsertClause) UsingClause* 'WHERE' GroupGraphPattern
    Modify,
    /// GraphRef => 'GRAPH' iri
    GraphRef,
    /// GraphRefAll => GraphRef | 'DEFAULT' | 'NAMED' | 'ALL'
    GraphRefAll,
    /// GraphOrDefault => 'DEFAULT' | 'GRAPH'? iri
    GraphOrDefault,
    /// QuadData => '{' Quads '}'
    QuadData,
    /// QuadPattern => '{' Quads '}'
    QuadPattern,
    /// DeleteClause => 'DELETE' QuadPattern
    DeleteClause,
    /// InsertClause => 'INSERT' QuadPattern
    InsertClause,
    /// UsingClause => 'USING' (iri | 'NAMED' iri)
    UsingClause,
    /// Quads => TriplesTemplate? (QuadsNotTriples '.'? TriplesTemplate?)*
    Quads,
    /// QuadsNotTriples => 'GRAPH' VarOrIri '{' TriplesTemplate? '}'
    QuadsNotTriples,
    /// TriplesSameSubject => VarOrTerm PropertyListNotEmpty | TriplesNode PropertyList
    TriplesSameSubject,
    /// GroupGraphPatternSub => TriplesBlock? (GraphPatternNotTriples '.'? TriplesBlock?)*
    GroupGraphPatternSub,
    /// TriplesBlock => TriplesSameSubjectPath ('.' TriplesBlock?)?
    TriplesBlock,
    /// GraphPatternNotTriples => GroupOrUnionGraphPattern | OptionalGraphPattern | MinusGraphPattern | GraphGraphPattern | ServiceGraphPattern | Filter | Bind | InlineData
    GraphPatternNotTriples,
    /// TriplesSameSubjectPath => VarOrTerm PropertyListPathNotEmpty | TriplesNodePath PropertyListPath
    TriplesSameSubjectPath,
    /// GroupOrUnionGraphPattern => GroupGraphPattern ('UNION' GroupGraphPattern)*
    GroupOrUnionGraphPattern,
    /// OptionalGraphPattern => 'OPTIONAL' GroupGraphPattern
    OptionalGraphPattern,
    /// MinusGraphPattern => 'MINUS' GroupGraphPattern
    MinusGraphPattern,
    /// GraphGraphPattern => 'GRAPH' VarOrIri GroupGraphPattern
    GraphGraphPattern,
    /// ServiceGraphPattern => 'SERVICE' 'SILENT'? VarOrIri GroupGraphPattern
    ServiceGraphPattern,
    /// Filter => 'FILTER' Constraint
    Filter,
    /// Bind => 'BIND' '(' Expression 'AS' Var ')'
    Bind,
    /// InlineData => 'VALUES' DataBlock
    InlineData,
    /// InlineDataOneVar => Var '{' DataBlockValue* '}'
    InlineDataOneVar,
    /// InlineDataFull => ('NIL' | '(' Var* ')') '{' ('(' DataBlockValue* ')' | 'NIL')* '}'
    InlineDataFull,
    /// DataBlockValue => iri | RDFLiteral | NumericLiteral | BooleanLiteral | 'UNDEF'
    DataBlockValue,
    /// RDFLiteral => String ('LANGTAG' | '^^' iri)?
    RDFLiteral,
    /// NumericLiteral => NumericLiteralUnsigned | NumericLiteralPositive | NumericLiteralNegative
    NumericLiteral,
    /// BooleanLiteral => 'true' | 'false'
    BooleanLiteral,
    /// ArgList => 'NIL' | '(' 'DISTINCT'? Expression (',' Expression)* ')'
    ArgList,
    /// ExpressionList => 'NIL' | '(' Expression (',' Expression)* ')'
    ExpressionList,
    /// ConstructTriples => TriplesSameSubject ('.' ConstructTriples?)?
    ConstructTriples,
    /// VarOrTerm => Var | GraphTerm
    VarOrTerm,
    /// PropertyListNotEmpty => Verb ObjectList (';' (Verb ObjectList)?)*
    PropertyListNotEmpty,
    /// TriplesNode => Collection | BlankNodePropertyList
    TriplesNode,
    /// PropertyList => PropertyListNotEmpty?
    PropertyList,
    /// Verb => VarOrIri | 'a'
    Verb,
    /// ObjectList => Object (',' Object)*
    ObjectList,
    /// Object => GraphNode
    Object,
    /// GraphNode => VarOrTerm | TriplesNode
    GraphNode,
    /// PropertyListPathNotEmpty => (VerbPath | VerbSimple) ObjectListPath (';' ((VerbPath | VerbSimple) ObjectList)?)*
    PropertyListPathNotEmpty,
    /// TriplesNodePath => CollectionPath | BlankNodePropertyListPath
    TriplesNodePath,
    /// PropertyListPath => PropertyListPathNotEmpty?
    PropertyListPath,
    /// VerbPath => Path
    VerbPath,
    /// VerbSimple => Var
    VerbSimple,
    /// ObjectListPath => ObjectPath (',' ObjectPath)*
    ObjectListPath,
    /// Path => PathAlternative
    Path,
    /// ObjectPath => GraphNodePath
    ObjectPath,
    /// GraphNodePath => VarOrTerm | TriplesNodePath
    GraphNodePath,
    /// PathAlternative => PathSequence ('|' PathSequence)*
    PathAlternative,
    /// PathSequence => PathEltOrInverse ('/' PathEltOrInverse)*
    PathSequence,
    /// PathEltOrInverse => PathElt | '^' PathElt
    PathEltOrInverse,
    /// PathElt => PathPrimary PathMod?
    PathElt,
    /// PathPrimary => iri | 'a' | '!' PathNegatedPropertySet | '(' Path ')'
    PathPrimary,
    /// PathMod => '?' | '*' | '+'
    PathMod,
    /// PathNegatedPropertySet => PathOneInPropertySet | '(' (PathOneInPropertySet ('|' PathOneInPropertySet)*)? ')'
    PathNegatedPropertySet,
    /// PathOneInPropertySet => iri | 'a' | '^' (iri | 'a')
    PathOneInPropertySet,
    /// Integer => 'INTEGER'
    Integer,
    /// Collection => '(' GraphNode GraphNode* ')'
    Collection,
    /// BlankNodePropertyList => '[' PropertyListNotEmpty ']'
    BlankNodePropertyList,
    /// CollectionPath => '(' GraphNodePath GraphNodePath* ')'
    CollectionPath,
    /// BlankNodePropertyListPath => '[' PropertyListPathNotEmpty ']'
    BlankNodePropertyListPath,
    /// GraphTerm => iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | 'NIL'
    GraphTerm,
    /// BlankNode => 'BLANK_NODE_LABEL' | 'ANON'
    BlankNode,
    /// ConditionalOrExpression => ConditionalAndExpression ('||' ConditionalAndExpression)*
    ConditionalOrExpression,
    /// ConditionalAndExpression => ValueLogical ('&&' ValueLogical)*
    ConditionalAndExpression,
    /// ValueLogical => RelationalExpression
    ValueLogical,
    /// RelationalExpression => NumericExpression ('=' NumericExpression | '!=' NumericExpression | '<' NumericExpression | '>' NumericExpression | '<=' NumericExpression | '>=' NumericExpression | 'IN' ExpressionList | 'NOT' 'IN' ExpressionList)?
    RelationalExpression,
    /// NumericExpression => AdditiveExpression
    NumericExpression,
    /// AdditiveExpression => MultiplicativeExpression ('+' MultiplicativeExpression | '-' MultiplicativeExpression | (NumericLiteralPositive | NumericLiteralNegative) ('*' UnaryExpression | '/' UnaryExpression)*)*
    AdditiveExpression,
    /// MultiplicativeExpression => UnaryExpression ('*' UnaryExpression | '/' UnaryExpression)*
    MultiplicativeExpression,
    /// NumericLiteralPositive => 'INTEGER_POSITIVE' | 'DECIMAL_POSITIVE' | 'DOUBLE_POSITIVE'
    NumericLiteralPositive,
    /// NumericLiteralNegative => 'INTEGER_NEGATIVE' | 'DECIMAL_NEGATIVE' | 'DOUBLE_NEGATIVE'
    NumericLiteralNegative,
    /// UnaryExpression => '!' PrimaryExpression | '+' PrimaryExpression | '-' PrimaryExpression | PrimaryExpression
    UnaryExpression,
    /// PrimaryExpression => BrackettedExpression | BuiltInCall | iriOrFunction | RDFLiteral | NumericLiteral | BooleanLiteral | Var
    PrimaryExpression,
    /// iriOrFunction => iri ArgList?
    iriOrFunction,
    /// Aggregate => 'COUNT' '(' 'DISTINCT'? ('*' | Expression) ')' | 'SUM' '(' 'DISTINCT'? Expression ')' | 'MIN' '(' 'DISTINCT'? Expression ')' | 'MAX' '(' 'DISTINCT'? Expression ')' | 'AVG' '(' 'DISTINCT'? Expression ')' | 'SAMPLE' '(' 'DISTINCT'? Expression ')' | 'GROUP_CONCAT' '(' 'DISTINCT'? Expression (';' 'SEPARATOR' '=' String)? ')'
    Aggregate,
    /// SubstringExpression => 'SUBSTR' '(' Expression ',' Expression (',' Expression)? ')'
    SubstringExpression,
    /// StrReplaceExpression => 'REPLACE' '(' Expression ',' Expression ',' Expression (',' Expression)? ')'
    StrReplaceExpression,
    /// RegexExpression => 'REGEX' '(' Expression ',' Expression (',' Expression)? ')'
    RegexExpression,
    /// ExistsFunc => 'EXISTS' GroupGraphPattern
    ExistsFunc,
    /// NotExistsFunc => 'NOT' 'EXISTS' GroupGraphPattern
    NotExistsFunc,
    /// String => 'STRING_LITERAL1' | 'STRING_LITERAL2' | 'STRING_LITERAL_LONG1' | 'STRING_LITERAL_LONG2'
    String,
    /// NumericLiteralUnsigned => 'INTEGER' | 'DECIMAL' | 'DOUBLE'
    NumericLiteralUnsigned,
    /// PrefixedName => 'PNAME_LN' | 'PNAME_NS'
    PrefixedName,
}
