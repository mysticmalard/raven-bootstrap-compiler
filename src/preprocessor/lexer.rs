use logos::Logos;
use token_macros::define_tokens;

define_tokens! {
    AND, OR, NOT,
    AS, IN,
    FN, MACRO, SHADER, API,
    STRUCT, ENUM, TRAIT,
    IMPL, DYN,

    MUT, PUB, STATIC, CONST,

    IMPORT, USE, MOD,
    LET,

    BREAK, CONTINUE, RETRY,
    RETURN, YIELD,

    FOR,
    MATCH,
    ATTEMPT, IF, WITH, TRY, WHILE, UNTIL,
    ELSE, DO, DEFER, FINALLY, EXCEPT,

    COMMA -> ",",
    COLON -> ":",
    SEMI -> ";",
    DOT -> ".",
    DCOLON -> "::",
    BARW -> "=>",
    MARW -> "-mut>",
    ARW["<-", "->"],

    PAREN["(", ")"],
    BRACKET["[", "]"],
    BRACE["{", "}"],
    ANGLE["<", ">"],

    Char(r"'(\\)?.'"),
    Lifetime(r"'[a-zA-Z_][a-zA-Z_0-9]*\s"),
    Number(r"[0-9]+"),
    Ident(r"[a-zA-Z_][a-zA-Z_0-9]*"),
    Literal(r#"".*?\""#),
    FStringLiteral(r#"f".*?""#),
    Unknown(".")
}