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
//! - 行頭 / スペース / `(` / `)` / `.` のいずれかで区切られている
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
// TODO: enum RomanError (impl Display, Error)
// TODO: Decimal newtype + TryFrom<&str> / TryFrom<u16>
// TODO: Roman newtype + TryFrom<&str>
// TODO: From<Decimal> for Roman / From<Roman> for Decimal

use std::borrow::Cow;

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

/// ローマ数字型。
struct Roman<'a>(Cow<'a, str>);

/// 1..=3999 の10進数型。
struct Decimal(u16);

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

#[cfg(test)]
mod tests {
    use super::*;

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
