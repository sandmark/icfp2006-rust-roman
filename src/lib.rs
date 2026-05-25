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
