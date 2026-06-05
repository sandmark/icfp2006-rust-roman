//! # Roman <-> Decimal Converter
//!
//! ## Overview
//! 任意長の [`String`] をスキャンし、ローマ数字と10進数を相互変換する。
//! `String` にローマ数字/10進数以外の文字が含まれている場合は無視し、
//! 変換結果にはそのまま含まれるものとする。
//!
//! ## Core Principles
//! `String` に含まれるローマ数字/10進数の抽出ルールは以下の条件に従う。
//! - `1` 以上 `3999` 以下
//! - 大文字アルファベット
//! - 行頭 / スペース / タブ / `(` / `)` のいずれかで区切られている
//!
//! ### Example
//! ローマ数字を含む以下の文字列は:
//! ```text
//! V        REM  +------------------------------------------------+
//! X        REM  | HACK.BAS      (c) 19100   fr33 v4r14bl3z       |
//! XV       REM  |                                                |
//! XX       REM  | Brute-forces passwords on UM vIX.0 systems.    |
//! XXV      REM  | Compile with Qvickbasic VII.0 or later:        |
//! XXX      REM  |    /bin/qbasic hack.bas                        |
//! XXXV     REM  | Then run:                                      |
//! XL       REM  |   ./hack.exe username                          |
//! XLV      REM  |                                                |
//! L        REM  | This program is for educational purposes only! |
//! LV       REM  +------------------------------------------------+
//! ```
//!
//! 10進数に変換されると以下のようになる:
//! ```text
//! 5        REM  +------------------------------------------------+
//! 10       REM  | HACK.BAS      (c) 19100   fr33 v4r14bl3z       |
//! 15       REM  |                                                |
//! 20       REM  | Brute-forces passwords on UM vIX.0 systems.    |
//! 25       REM  | Compile with Qvickbasic VII.0 or later:        |
//! 30       REM  |    /bin/qbasic hack.bas                        |
//! 35       REM  | Then run:                                      |
//! 40       REM  |   ./hack.exe username                          |
//! 45       REM  |                                                |
//! 50       REM  | This program is for educational purposes only! |
//! 55       REM  +------------------------------------------------+
//! ```

// TODO: Scan(字句解析/str) -> Parse(ドメイン/境界/型) -> Logic(変換)

use std::{borrow::Cow, fmt::Display, num::ParseIntError};

/// ローマ数字 <-> 10進数相互変換テーブル
const SYMBOLS: [(u16, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

/// ローマ数字型。
#[derive(Debug, PartialEq)]
struct Roman<'a>(Cow<'a, str>);

/// 1..=3999 の10進数型。
#[derive(Debug, PartialEq)]
struct Decimal(u16);

/// [tokenize] 結果。
#[derive(Debug, PartialEq)]
enum Token<'a> {
    Roman(Roman<'a>),
    Decimal(Decimal),
    Other(&'a str),
}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Roman(r) => write!(f, "{}", r.0),
            Token::Decimal(d) => write!(f, "{}", d.0),
            Token::Other(s) => write!(f, "{}", s),
        }
    }
}

/// ローマ数字に変換可能 (1..=3999) な `n` を受け取り、
/// [SYMBOLS] からもっとも大きな数と結び付いたタプルを返す。
///
/// [SYMBOLS] に候補が存在しない場合は論理的バグであるため
/// 明示的に panic  するものとする。
fn next_symbol(n: u16) -> (u16, &'static str) {
    SYMBOLS
        .iter()
        .find(|(v, _)| n >= *v)
        .copied()
        .expect("n must be >= 1")
}

/// 10進数 [Decimal] をローマ数字 [Roman] に変換する [From] トレイト実装。
impl From<Decimal> for Roman<'_> {
    fn from(decimal: Decimal) -> Self {
        let mut buf = String::new();
        let mut remaining = decimal.0;

        while remaining > 0 {
            let (amount, symbol) = next_symbol(remaining);
            remaining -= amount;
            buf.push_str(symbol);
        }

        Self(Cow::from(buf))
    }
}

/// ローマ数字文字列を先頭から走査し、各トークンに該当する [SYMBOLS] を順に返すイテレータ。
/// 先頭に一致する記号がなくなった時点で停止する。
fn tokens(roman: &str) -> impl Iterator<Item = (u16, &str)> + '_ {
    let mut rest = roman;
    std::iter::from_fn(move || {
        let (value, symbol, next) = SYMBOLS
            .iter()
            .find_map(|(value, sym)| rest.strip_prefix(sym).map(|tail| (*value, *sym, tail)))?;
        rest = next;
        Some((value, symbol))
    })
}

/// ローマ数字文字列 [Roman] を10進数 [Decimal] に変換する [From] トレイト実装。
impl From<Roman<'_>> for Decimal {
    fn from(roman: Roman) -> Self {
        Self(tokens(&roman.0).map(|(value, _)| value).sum())
    }
}

#[derive(Debug, PartialEq)]
/// [Roman] の [TryFrom] (&str) で使用されるエラーバリアント。
enum ParseRomanError {
    Malformed(String),
    Parse(String),
}

/// 文字列からローマ数字型 [Roman] に変換するトレイト実装。
impl<'a> TryFrom<&'a str> for Roman<'a> {
    type Error = ParseRomanError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // Check: empty?
        if value.is_empty() {
            return Err(ParseRomanError::Parse(value.to_string()));
        }

        // Check: every symbol?
        if !value.chars().all(|c| "IVXLCDM".contains(c)) {
            return Err(ParseRomanError::Parse(value.to_string()));
        }

        // Check: malformed input
        // NOTE: 入力を Decimal に変換し、さらに Roman にした上で diff を取る。
        // - "IIII" → 4 → Roman::from(4) == "IV" → "IV" != "IIII" → Err
        // - "III" → 3 → "III" → Ok
        // - "VIM" → 1006 → "MVI" → Err
        // - "IVIV" → 8 → "VIII" → Err
        let decimal = Decimal::from(Roman(Cow::from(value)));
        let roman = Roman::from(decimal);
        if roman.0 != value {
            return Err(ParseRomanError::Malformed(value.to_string()));
        }

        Ok(roman)
    }
}

#[derive(Debug, PartialEq)]
/// [Decimal] の [TryFrom] (&str) で使用されるエラーバリアント。
///
/// - OutOfRange: 1..=3999 でないときのドメインエラー。
/// - Parse(_): `parse()` したときのエラーを閉じ込める非ドメインエラー。
enum ParseDecimalError {
    OutOfRange(u16),
    Parse(ParseIntError),
}

/// [ParseIntError] を [ParseDecimalError] に変換する。
///
/// NOTE: [Decimal::try_from] で parse()? するため。
impl From<ParseIntError> for ParseDecimalError {
    fn from(value: ParseIntError) -> Self {
        ParseDecimalError::Parse(value)
    }
}

/// [str] を [Decimal] に変換する。
///
/// - ローマ数字に変換できる範囲 1..=3999 でない(ドメインエラー) -> [ParseDecimalError::OutOfRange]
/// - 空文字列または数値に変換できない `parse()` の非ドメインエラー -> [ParseDecimalError::Parse]
impl TryFrom<&str> for Decimal {
    type Error = ParseDecimalError;
    fn try_from(value: &str) -> Result<Decimal, Self::Error> {
        let n = value.parse::<u16>()?;

        if !(1..=3999).contains(&n) {
            return Err(ParseDecimalError::OutOfRange(n));
        }

        Ok(Decimal(n))
    }
}

/// QvickBasic の一行 `input` を読み込み、字句で分解した [Vec] を返す。
fn scan(input: &str) -> Vec<String> {
    let mut acc = String::new();
    let mut segments: Vec<String> = Vec::new();
    let mut inside_quote = false;

    for c in input.chars() {
        match c {
            ' ' | '\t' => {
                if inside_quote {
                    acc.push(c);
                } else {
                    segments.push(acc);
                    acc = String::new();
                }
            }
            '(' | ')' | ',' => {
                if inside_quote {
                    acc.push(c);
                } else {
                    segments.push(acc);
                    segments.push(c.to_string());
                    acc = String::new();
                }
            }
            '"' => {
                if inside_quote {
                    inside_quote = false;
                    acc.push(c);
                    segments.push(acc);
                    acc = String::new();
                } else {
                    inside_quote = true;
                    segments.push(acc);
                    acc = String::new();
                    acc.push(c);
                }
            }
            _ => {
                acc.push(c);
            }
        }
    }

    segments.push(acc);

    return segments.into_iter().filter(|s| !s.is_empty()).collect();
}

/// [scan] 結果を受け取り、 [Token] に変換した [Vec] を返す。
fn tokenize<'a>(segments: &'a [String]) -> Vec<Token<'a>> {
    segments
        .iter()
        .map(
            |s| match (Roman::try_from(s.as_str()), Decimal::try_from(s.as_str())) {
                (Ok(r), _) => Token::Roman(r),
                (_, Ok(d)) => Token::Decimal(d),
                (Err(_), Err(_)) => Token::Other(s.as_str()),
            },
        )
        .collect()
}

/// [Token::Roman] を [Token::Decimal] へ変換して返す。
fn decimalize(token: Token) -> Token {
    if let Token::Roman(r) = token {
        Token::Decimal(Decimal::from(r))
    } else {
        token
    }
}

/// [Token::Decimal] を [Token::Roman] へ変換して返す。
fn romanize<'a>(token: Token) -> Token {
    if let Token::Decimal(d) = token {
        Token::Roman(Roman::from(d))
    } else {
        token
    }
}

/// 一行分の [Token] の [Vec] を [String] にして返す。
fn render(tokens: Vec<Token>) -> String {
    tokens
        .into_iter()
        .map(|token| token.to_string())
        .collect::<Vec<String>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod render {
        use super::*;

        #[test]
        fn displays_tokens_as_string() {
            assert_eq!(
                render(vec![
                    Token::Roman(Roman(Cow::from("IV"))),
                    Token::Other("REM"),
                    Token::Other("words"),
                    Token::Other("("),
                    Token::Roman(Roman(Cow::from("II"))),
                    Token::Other(")"),
                    Token::Other("="),
                    Token::Other("\"plane\"")
                ]),
                "IV REM words ( II ) = \"plane\""
            );
        }
    }
    mod convert {
        use super::*;

        #[test]
        fn decimalize_roman() {
            assert_eq!(
                decimalize(Token::Roman(Roman(Cow::from("IV")))),
                Token::Decimal(Decimal(4))
            );
        }

        #[test]
        fn romanize_decimal() {
            assert_eq!(
                romanize(Token::Decimal(Decimal(4))),
                Token::Roman(Roman(Cow::from("IV")))
            );
        }
    }
    mod tokenize {
        use super::*;

        #[test]
        fn converts_string_into_roman() {
            assert_eq!(
                tokenize(&["IV".to_string()]),
                vec![Token::Roman(Roman(Cow::from("IV")))]
            );
        }

        #[test]
        fn converts_string_into_decimal() {
            assert_eq!(
                tokenize(&["4".to_string()]),
                vec![Token::Decimal(Decimal(4))]
            );
        }

        #[test]
        fn converts_string_into_other() {
            assert_eq!(
                tokenize(&["0.VII".to_string()]),
                vec![Token::Other("0.VII")]
            );
        }
    }
    mod scan {
        use super::*;

        // 正常系(回帰): 終端のないダブルクォートはトークンを分割するだけ。
        #[test]
        fn splits_quote_as_segment_midword() {
            assert_eq!(scan("ab\"cdef"), vec!["ab", "\"cdef"]);
        }

        // 正常系: 空白でトークンに分割する。
        #[test]
        fn splits_on_whitespace() {
            assert_eq!(scan("V REM"), vec!["V", "REM"]);
        }

        // 正常系: 先頭・末尾・連続する空白は除去され、空トークンは残らない。
        #[test]
        fn ignores_surrounding_whitespace() {
            assert_eq!(scan(" V REM "), vec!["V", "REM"]);
            assert_eq!(scan("V    REM"), vec!["V", "REM"]);
        }

        // 正常系: `(` `)` はそれぞれ独立したトークンになり、隣接する語を区切る。
        #[test]
        fn splits_parentheses_into_separate_tokens() {
            assert_eq!(scan("words(IV)"), vec!["words", "(", "IV", ")"]);
        }

        // 正常系: ダブルクォートで囲まれた範囲は、内部の空白・括弧を含めて 1 トークンとして保持する。
        #[test]
        fn keeps_quoted_text_as_single_token() {
            assert_eq!(scan("X \"a b (c)\" Y"), vec!["X", "\"a b (c)\"", "Y"]);
        }

        // 正常系: hack.bas に現れる実際の行を分割できる（クォート保持・括弧分割・空白除去の複合）。
        #[test]
        fn tokenizes_realistic_basic_line() {
            assert_eq!(
                scan("CDXXX    PRINT \"found match!! (for user) \" + username + CHR(X)"),
                vec![
                    "CDXXX",
                    "PRINT",
                    "\"found match!! (for user) \"",
                    "+",
                    "username",
                    "+",
                    "CHR",
                    "(",
                    "X",
                    ")"
                ]
            );
        }

        // 異常系: 空文字列・空白のみの入力はトークンを生まない。
        #[test]
        fn returns_empty_for_blank_input() {
            assert!(scan("").is_empty());
            assert!(scan("   ").is_empty());
        }

        // 正常系: `.` は区切り文字ではない。REM コメント中にしか現れないため
        //        トークナイズ対象外とし、連結したまま 1 トークンとする
        //        （例: "VII.0"）。NOTE: lib.rs 冒頭 docstring の区切り規則に
        //        挙がっている `.` は誤記で、後日修正予定。
        #[test]
        fn keeps_period_glued() {
            assert_eq!(scan("VII.0"), vec!["VII.0"]);
        }

        // 正常系: タブはスペースと同じ区切り文字として扱う。
        #[test]
        fn splits_on_tab() {
            assert_eq!(scan("V\tREM"), vec!["V", "REM"]);
            assert_eq!(scan("\tV\t"), vec!["V"]);
        }

        // 正常系: `,` は単独トークンになる。
        #[test]
        fn splits_comma_into_separate_token() {
            assert_eq!(scan("a,b"), vec!["a", ",", "b"]);
            assert_eq!(scan("username, words"), vec!["username", ",", "words"]);
        }

        // 正常系(回帰): 空引数呼び出しは括弧だけが残る（空 acc は除去される）。
        #[test]
        fn keeps_parentheses_of_empty_call() {
            assert_eq!(scan("ARGS()"), vec!["ARGS", "(", ")"]);
        }

        // 正常系(回帰): ネストした呼び出しの連続する閉じ括弧は個別トークンに割れる。
        #[test]
        fn splits_consecutive_close_parentheses() {
            assert_eq!(
                scan("OUTER(INNER(i))"),
                vec!["OUTER", "(", "INNER", "(", "i", ")", ")"]
            );
        }
    }
    mod try_from_roman {
        use super::*;

        // 正常系: 妥当なローマ数字の文字列は Ok(Roman) に変換される。
        #[test]
        fn converts_valid_roman_string() {
            assert_eq!(Roman::try_from("I").unwrap().0, "I");
            assert_eq!(Roman::try_from("III").unwrap().0, "III");
            assert_eq!(Roman::try_from("XIV").unwrap().0, "XIV");
            assert_eq!(Roman::try_from("MMMCMXCIX").unwrap().0, "MMMCMXCIX");
        }

        // 異常系: 不正なローマ数字はエラー。
        #[test]
        fn rejects_malformed_input() {
            assert!(matches!(
                Roman::try_from("IIII"),
                Err(ParseRomanError::Malformed(_))
            ));
            assert!(matches!(
                Roman::try_from("VIM"),
                Err(ParseRomanError::Malformed(_))
            ));
            assert!(matches!(
                Roman::try_from("XIVI"),
                Err(ParseRomanError::Malformed(_))
            ));
            assert!(matches!(
                Roman::try_from("IVIV"),
                Err(ParseRomanError::Malformed(_))
            ));
        }

        // 異常系: 空文字列。
        #[test]
        fn rejects_empty_input() {
            assert!(matches!(
                Roman::try_from(""),
                Err(ParseRomanError::Parse(_))
            ));
        }

        // 異常系: ローマ字シンボルではない文字列。
        #[test]
        fn rejects_not_symbols() {
            assert!(matches!(
                Roman::try_from("1"),
                Err(ParseRomanError::Parse(_))
            ));
        }
    }
    mod try_from_decimal {
        use super::*;

        // 正常系: 妥当な10進数文字列は Ok(Decimal) に変換される。
        #[test]
        fn converts_valid_decimal_string() {
            assert_eq!(Decimal::try_from("1").unwrap().0, 1);
            assert_eq!(Decimal::try_from("42").unwrap().0, 42);
            assert_eq!(Decimal::try_from("3999").unwrap().0, 3999);
        }

        // 異常系: ローマ数字の範囲外の文字列はドメインエラー。
        #[test]
        fn rejects_out_of_range() {
            assert_eq!(
                Decimal::try_from("0"),
                Err(ParseDecimalError::OutOfRange(0))
            );
            assert_eq!(
                Decimal::try_from("4000"),
                Err(ParseDecimalError::OutOfRange(4000))
            );
        }

        // 異常系: 空文字列、数値にできない場合は Parse エラー。
        #[test]
        fn rejects_unparseable_input() {
            assert!(matches!(
                Decimal::try_from("XIV"),
                Err(ParseDecimalError::Parse(_))
            ));
            assert!(matches!(
                Decimal::try_from(""),
                Err(ParseDecimalError::Parse(_))
            ));
        }
    }
    mod from_decimal {
        use super::*;

        // 基本記号: ローマ数字の単一記号と対応する10進数。
        #[test]
        fn converts_basic_symbols() {
            assert_eq!(Roman::from(Decimal(1)).0, "I");
            assert_eq!(Roman::from(Decimal(5)).0, "V");
            assert_eq!(Roman::from(Decimal(10)).0, "X");
            assert_eq!(Roman::from(Decimal(50)).0, "L");
            assert_eq!(Roman::from(Decimal(100)).0, "C");
            assert_eq!(Roman::from(Decimal(500)).0, "D");
            assert_eq!(Roman::from(Decimal(1000)).0, "M");
        }

        // 加法的合成: 同じ記号を繰り返して大きい数を表す (例: III = 1+1+1)。
        #[test]
        fn converts_additive_combinations() {
            assert_eq!(Roman::from(Decimal(2)).0, "II");
            assert_eq!(Roman::from(Decimal(3)).0, "III");
            assert_eq!(Roman::from(Decimal(6)).0, "VI"); // 5 + 1
            assert_eq!(Roman::from(Decimal(7)).0, "VII"); // 5 + 1 + 1
            assert_eq!(Roman::from(Decimal(8)).0, "VIII"); // 5 + 1 + 1 + 1
        }

        // 減法的記法: 小さい記号を大きい記号の前に置いて引き算を表す (例: IV = 5-1)。
        #[test]
        fn converts_subtractive_notation() {
            assert_eq!(Roman::from(Decimal(4)).0, "IV"); // 5 - 1
            assert_eq!(Roman::from(Decimal(9)).0, "IX"); // 10 - 1
            assert_eq!(Roman::from(Decimal(40)).0, "XL"); // 50 - 10
            assert_eq!(Roman::from(Decimal(90)).0, "XC"); // 100 - 10
            assert_eq!(Roman::from(Decimal(400)).0, "CD"); // 500 - 100
            assert_eq!(Roman::from(Decimal(900)).0, "CM"); // 1000 - 100
        }

        // 複合: 複数の桁が組み合わさった数 (千の位 + 百の位 + 十の位 + 一の位)。
        #[test]
        fn converts_composite_numbers() {
            assert_eq!(Roman::from(Decimal(14)).0, "XIV"); // X + IV
            assert_eq!(Roman::from(Decimal(42)).0, "XLII"); // XL + II
            assert_eq!(Roman::from(Decimal(99)).0, "XCIX"); // XC + IX
            assert_eq!(Roman::from(Decimal(1984)).0, "MCMLXXXIV"); // M + CM + LXXX + IV
        }

        // 境界値: 仕様範囲 (1..=3999) の最小値と最大値。
        #[test]
        fn converts_boundary_values() {
            assert_eq!(Roman::from(Decimal(1)).0, "I");
            assert_eq!(Roman::from(Decimal(3999)).0, "MMMCMXCIX");
        }
    }
    mod from_roman {
        use super::*;

        // 基本記号: ローマ数字の単一記号と対応する10進数。
        #[test]
        fn converts_decimal_basic_symbols() {
            assert_eq!(Decimal::from(Roman(Cow::from("I"))).0, 1);
            assert_eq!(Decimal::from(Roman(Cow::from("V"))).0, 5);
            assert_eq!(Decimal::from(Roman(Cow::from("X"))).0, 10);
            assert_eq!(Decimal::from(Roman(Cow::from("L"))).0, 50);
            assert_eq!(Decimal::from(Roman(Cow::from("C"))).0, 100);
            assert_eq!(Decimal::from(Roman(Cow::from("D"))).0, 500);
            assert_eq!(Decimal::from(Roman(Cow::from("M"))).0, 1000);
        }

        // 加法的合成: 同じ記号を繰り返して大きい数を表す (例: III = 1+1+1)。
        #[test]
        fn converts_decimal_additive_combinations() {
            assert_eq!(Decimal::from(Roman(Cow::from("II"))).0, 2);
            assert_eq!(Decimal::from(Roman(Cow::from("III"))).0, 3);
            assert_eq!(Decimal::from(Roman(Cow::from("VI"))).0, 6);
            assert_eq!(Decimal::from(Roman(Cow::from("VII"))).0, 7);
            assert_eq!(Decimal::from(Roman(Cow::from("VIII"))).0, 8);
        }

        // 減法的記法: 小さい記号を大きい記号の前に置いて引き算を表す (例: IV = 5-1)。
        #[test]
        fn converts_decimal_subtractive_notation() {
            assert_eq!(Decimal::from(Roman(Cow::from("IV"))).0, 4);
            assert_eq!(Decimal::from(Roman(Cow::from("IX"))).0, 9);
            assert_eq!(Decimal::from(Roman(Cow::from("XL"))).0, 40);
            assert_eq!(Decimal::from(Roman(Cow::from("XC"))).0, 90);
            assert_eq!(Decimal::from(Roman(Cow::from("CD"))).0, 400);
            assert_eq!(Decimal::from(Roman(Cow::from("CM"))).0, 900);
        }

        // 複合: 複数の桁が組み合わさった数。
        #[test]
        fn converts_decimal_composite_numbers() {
            assert_eq!(Decimal::from(Roman(Cow::from("XIV"))).0, 14);
            assert_eq!(Decimal::from(Roman(Cow::from("XLII"))).0, 42);
            assert_eq!(Decimal::from(Roman(Cow::from("XCIX"))).0, 99);
            assert_eq!(Decimal::from(Roman(Cow::from("MCMLXXXIV"))).0, 1984);
        }

        // 境界値: 仕様範囲 (1..=3999) の最小値と最大値。
        #[test]
        fn converts_decimal_boundary_values() {
            assert_eq!(Decimal::from(Roman(Cow::from("I"))).0, 1);
            assert_eq!(Decimal::from(Roman(Cow::from("MMMCMXCIX"))).0, 3999);
        }
    }
}
