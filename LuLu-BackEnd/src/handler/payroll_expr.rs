//! Payroll formula evaluator: parse and evaluate salary rule expressions.
//!
//! Usage: build a [`PayrollVars`], convert with [`PayrollVars::to_map`],
//! then call [`evaluate`]. Returns the numeric result or [`ExprError`].
//!
//! Grammar: numbers, variables, `+ - * / %`, parens,
//! comparisons `> < >= <= == !=` (yield 1 or 0),
//! functions `if(cond, a, b)` (`iif` is an alias), `or(...)`, `and(...)`,
//! `floor(x)`, `ceiling(x)` (`ceil` is an alias).
//! A leading `=` (spreadsheet style) is ignored, and `;` works
//! as an argument separator just like `,`.
//! Requires Rust 1.70+ (std only, no dependencies).

use std::collections::HashMap;

/// All variables a payroll formula may reference.
pub const KNOWN_VARS: &[&str] = &[
    "overtime",
    "absent",
    "present",
    "salary",
    "monthly_rate",
    "daily_rate",
    "hourly_rate",
    "late",
    "late_time",
    "late_day",
    "sick",
    "permission",
    "dow",
    "hols",
];

/// Numeric inputs for one payroll calculation.
#[derive(Debug, Clone, Default)]
pub struct PayrollVars {
    pub overtime: f64,
    pub absent: f64,
    pub present: f64,
    pub salary: f64,
    pub monthly_rate: f64,
    pub daily_rate: f64,
    pub hourly_rate: f64,
    pub late: f64,
    pub late_time: f64,
    pub late_day: f64,
    pub sick: f64,
    pub permission: f64,
    pub dow: f64,
    pub hols: f64,
}

impl PayrollVars {
    /// Export fields as a name to value map for [`evaluate`].
    pub fn to_map(&self) -> HashMap<String, f64> {
        let pairs = [
            ("overtime", self.overtime),
            ("absent", self.absent),
            ("present", self.present),
            ("salary", self.salary),
            ("monthly_rate", self.monthly_rate),
            ("daily_rate", self.daily_rate),
            ("hourly_rate", self.hourly_rate),
            ("late", self.late),
            ("late_time", self.late_time),
            ("late_day", self.late_day),
            ("sick", self.sick),
            ("permission", self.permission),
            ("dow", self.dow),
            ("hols", self.hols),
        ];
        pairs
            .iter()
            .map(|(name, value)| (name.to_string(), *value))
            .collect()
    }
}

/// Parse or evaluation failure.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprError(pub String);

impl std::fmt::Display for ExprError {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{}", self.0)
    }
}

impl std::error::Error for ExprError {}

fn fail(message: impl Into<String>) -> ExprError {
    ExprError(message.into())
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    Comma,
    Gt,
    Lt,
    GtEq,
    LtEq,
    EqEq,
    NotEq,
    End,
}

fn tokenize(expr: &str) -> Result<Vec<Token>, ExprError> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch.is_ascii_digit() || (ch == '.' && chars.clone().nth(1).is_some_and(|n| n.is_ascii_digit())) {
            let mut text = String::new();
            let mut dots = 0;
            while let Some(&digit) = chars.peek() {
                if digit.is_ascii_digit() {
                    text.push(digit);
                    chars.next();
                } else if digit == '.' && dots == 0 {
                    dots += 1;
                    text.push(digit);
                    chars.next();
                } else {
                    break;
                }
            }
            text.parse::<f64>()
                .map(Token::Number)
                .map_err(|_| fail(format!("bad number '{text}'")))
                .map(|token| tokens.push(token))?;
        } else if ch.is_alphabetic() || ch == '_' {
            let mut name = String::new();
            while let Some(&letter) = chars.peek() {
                if letter.is_alphanumeric() || letter == '_' {
                    name.push(letter);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(Token::Ident(name));
        } else {
            chars.next();
            let next = chars.peek().copied();
            match ch {
                '+' => tokens.push(Token::Plus),
                '-' => tokens.push(Token::Minus),
                '*' => tokens.push(Token::Star),
                '/' => tokens.push(Token::Slash),
                '%' => tokens.push(Token::Percent),
                '(' => tokens.push(Token::LParen),
                ')' => tokens.push(Token::RParen),
                ',' | ';' => tokens.push(Token::Comma),
                '>' => tokens.push(if next == Some('=') {
                    chars.next();
                    Token::GtEq
                } else {
                    Token::Gt
                }),
                '<' => tokens.push(if next == Some('=') {
                    chars.next();
                    Token::LtEq
                } else {
                    Token::Lt
                }),
                '=' => {
                    if next == Some('=') {
                        chars.next();
                        tokens.push(Token::EqEq);
                    } else {
                        return Err(fail("use '==' to compare, single '=' is not allowed"));
                    }
                }
                '!' => {
                    if next == Some('=') {
                        chars.next();
                        tokens.push(Token::NotEq);
                    } else {
                        return Err(fail("use '!=' to compare, lone '!' is not allowed"));
                    }
                }
                _ => return Err(fail(format!("unexpected character '{ch}'"))),
            }
        }
    }
    tokens.push(Token::End);
    Ok(tokens)
}

/// Nesting ceiling for the recursive-descent parser. Every '(' or unary sign
/// costs several stack frames, and exceeding the worker stack aborts the whole
/// process (Rust's stack-overflow handler is not per-thread). 64 is far above any
/// real payroll formula and far below what a 2 MiB stack can take.
const MAX_DEPTH: usize = 64;

struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    vars: &'a HashMap<String, f64>,
    depth: usize,
}

impl<'a> Parser<'a> {
    fn enter(&mut self) -> Result<(), ExprError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(fail("formula nested too deeply"));
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn parse_expr(&mut self) -> Result<f64, ExprError> {
        self.enter()?;
        let result = self.parse_expr_inner();
        self.leave();
        result
    }

    fn parse_expr_inner(&mut self) -> Result<f64, ExprError> {
        let mut left = self.parse_add()?;
        left = match self.peek() {
            Token::Gt => {
                self.advance();
                let right = self.parse_add()?;
                if left > right { 1.0 } else { 0.0 }
            }
            Token::Lt => {
                self.advance();
                let right = self.parse_add()?;
                if left < right { 1.0 } else { 0.0 }
            }
            Token::GtEq => {
                self.advance();
                let right = self.parse_add()?;
                if left >= right { 1.0 } else { 0.0 }
            }
            Token::LtEq => {
                self.advance();
                let right = self.parse_add()?;
                if left <= right { 1.0 } else { 0.0 }
            }
            Token::EqEq => {
                self.advance();
                let right = self.parse_add()?;
                if left == right { 1.0 } else { 0.0 }
            }
            Token::NotEq => {
                self.advance();
                let right = self.parse_add()?;
                if left != right { 1.0 } else { 0.0 }
            }
            _ => left,
        };
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<f64, ExprError> {
        let mut value = self.parse_mul()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    value += self.parse_mul()?;
                }
                Token::Minus => {
                    self.advance();
                    value -= self.parse_mul()?;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_mul(&mut self) -> Result<f64, ExprError> {
        let mut value = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    value *= self.parse_unary()?;
                }
                Token::Slash => {
                    self.advance();
                    let divisor = self.parse_unary()?;
                    if divisor == 0.0 {
                        return Err(fail("division by zero"));
                    }
                    value /= divisor;
                }
                Token::Percent => {
                    self.advance();
                    let divisor = self.parse_unary()?;
                    if divisor == 0.0 {
                        return Err(fail("division by zero"));
                    }
                    value %= divisor;
                }
                _ => return Ok(value),
            }
        }
    }

    fn parse_unary(&mut self) -> Result<f64, ExprError> {
        self.enter()?;
        let result = self.parse_unary_inner();
        self.leave();
        result
    }

    fn parse_unary_inner(&mut self) -> Result<f64, ExprError> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                Ok(-self.parse_unary()?)
            }
            Token::Plus => {
                self.advance();
                self.parse_unary()
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<f64, ExprError> {
        match self.peek().clone() {
            Token::Number(value) => {
                self.advance();
                Ok(value)
            }
            Token::Ident(name) => {
                self.advance();
                if *self.peek() == Token::LParen {
                    self.parse_call(&name)
                } else if let Some(value) = self.vars.get(&name) {
                    Ok(*value)
                } else {
                    Err(fail(format!("unknown variable '{name}'")))
                }
            }
            Token::LParen => {
                self.advance();
                let value = self.parse_expr()?;
                match self.peek() {
                    Token::RParen => {
                        self.advance();
                        Ok(value)
                    }
                    _ => Err(fail("missing closing ')'")),
                }
            }
            _ => Err(fail("expected a number, variable, or '('")),
        }
    }

    /// Function calls. `if` is lazy: only the taken branch is evaluated.
    /// `iif` is an alias with the same lazy behavior.
    fn parse_call(&mut self, name: &str) -> Result<f64, ExprError> {
        self.advance(); // consume '('
        if name == "if" || name == "iif" {
            return self.parse_if(name);
        }
        let mut args = Vec::new();
        if *self.peek() != Token::RParen {
            loop {
                args.push(self.parse_expr()?);
                match self.peek() {
                    Token::Comma => self.advance(),
                    _ => break,
                }
            }
        }
        match self.peek() {
            Token::RParen => self.advance(),
            _ => return Err(fail(format!("missing closing ')' in '{name}' call"))),
        }
        match name {
            "or" => {
                if args.is_empty() {
                    return Err(fail("'or' needs at least 1 argument"));
                }
                Ok(if args.iter().any(|a| *a != 0.0) {
                    1.0
                } else {
                    0.0
                })
            }
            "and" => {
                if args.is_empty() {
                    return Err(fail("'and' needs at least 1 argument"));
                }
                Ok(if args.iter().all(|a| *a != 0.0) {
                    1.0
                } else {
                    0.0
                })
            }
            "floor" => {
                if args.len() != 1 {
                    return Err(fail("'floor' needs exactly 1 argument"));
                }
                Ok(args[0].floor())
            }
            "ceiling" | "ceil" => {
                if args.len() != 1 {
                    return Err(fail("'{name}' needs exactly 1 argument"));
                }
                Ok(args[0].ceil())
            }
            _ => Err(fail(format!("unknown function '{name}'"))),
        }
    }

    /// Parse `if(cond, then, else)` with lazy branches: skip (don't eval)
    /// the untaken side so `if(x > 0, 10 / x, 0)` never divides by zero.
    /// `name` is the spelling used (`if` or `iif`) for error messages.
    fn parse_if(&mut self, name: &str) -> Result<f64, ExprError> {
        let cond = self.parse_expr()?;
        if *self.peek() != Token::Comma {
            return Err(fail(format!(
                "'{name}' needs 3 arguments: {name}(cond, then, else)"
            )));
        }
        self.advance();
        if cond != 0.0 {
            let then_value = self.parse_expr()?;
            if *self.peek() != Token::Comma {
                return Err(fail(format!(
                    "'{name}' needs 3 arguments: {name}(cond, then, else)"
                )));
            }
            self.advance();
            self.skip_expr()?;
            self.expect_rparen(name)?;
            Ok(then_value)
        } else {
            self.skip_expr()?;
            if *self.peek() != Token::Comma {
                return Err(fail(format!(
                    "'{name}' needs 3 arguments: {name}(cond, then, else)"
                )));
            }
            self.advance();
            let else_value = self.parse_expr()?;
            self.expect_rparen(name)?;
            Ok(else_value)
        }
    }

    fn expect_rparen(&mut self, name: &str) -> Result<(), ExprError> {
        match self.peek() {
            Token::RParen => {
                self.advance();
                Ok(())
            }
            _ => Err(fail(format!("missing closing ')' in '{name}' call"))),
        }
    }

    /// Skip one expression without evaluating (for the untaken `if` branch).
    fn skip_expr(&mut self) -> Result<(), ExprError> {
        self.enter()?;
        let result = self.skip_expr_inner();
        self.leave();
        result
    }

    fn skip_expr_inner(&mut self) -> Result<(), ExprError> {
        self.skip_add()?;
        match self.peek() {
            Token::Gt | Token::Lt | Token::GtEq | Token::LtEq | Token::EqEq | Token::NotEq => {
                self.advance();
                self.skip_add()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn skip_args(&mut self) -> Result<(), ExprError> {
        self.advance(); // consume '('
        if *self.peek() != Token::RParen {
            loop {
                self.skip_expr()?;
                match self.peek() {
                    Token::Comma => self.advance(),
                    _ => break,
                }
            }
        }
        match self.peek() {
            Token::RParen => {
                self.advance();
                Ok(())
            }
            _ => Err(fail("missing closing ')'")),
        }
    }

    fn skip_add(&mut self) -> Result<(), ExprError> {
        self.skip_mul()?;
        loop {
            match self.peek() {
                Token::Plus | Token::Minus => {
                    self.advance();
                    self.skip_mul()?;
                }
                _ => return Ok(()),
            }
        }
    }

    fn skip_mul(&mut self) -> Result<(), ExprError> {
        self.skip_unary()?;
        loop {
            match self.peek() {
                Token::Star | Token::Slash | Token::Percent => {
                    self.advance();
                    self.skip_unary()?;
                }
                _ => return Ok(()),
            }
        }
    }

    fn skip_unary(&mut self) -> Result<(), ExprError> {
        self.enter()?;
        let result = self.skip_unary_inner();
        self.leave();
        result
    }

    fn skip_unary_inner(&mut self) -> Result<(), ExprError> {
        match self.peek() {
            Token::Minus | Token::Plus => {
                self.advance();
                self.skip_unary()
            }
            _ => self.skip_primary(),
        }
    }

    fn skip_primary(&mut self) -> Result<(), ExprError> {
        match self.peek().clone() {
            Token::Number(_) => {
                self.advance();
                Ok(())
            }
            Token::Ident(_) => {
                self.advance();
                if *self.peek() == Token::LParen {
                    let is_if = matches!(
                        self.tokens.get(self.pos.saturating_sub(1)),
                        Some(Token::Ident(name)) if name == "if" || name == "iif"
                    );
                    if is_if {
                        self.skip_if_args()
                    } else {
                        self.skip_args()
                    }
                } else {
                    Ok(())
                }
            }
            Token::LParen => {
                self.advance();
                self.skip_expr()?;
                match self.peek() {
                    Token::RParen => {
                        self.advance();
                        Ok(())
                    }
                    _ => Err(fail("missing closing ')'")),
                }
            }
            _ => Err(fail("expected a number, variable, or '('")),
        }
    }

    fn skip_if_args(&mut self) -> Result<(), ExprError> {
        self.advance(); // consume '('
        self.skip_expr()?;
        if *self.peek() != Token::Comma {
            return Err(fail("'if/iif' needs 3 arguments: if(cond, then, else)"));
        }
        self.advance();
        self.skip_expr()?;
        if *self.peek() != Token::Comma {
            return Err(fail("'if/iif' needs 3 arguments: if(cond, then, else)"));
        }
        self.advance();
        self.skip_expr()?;
        self.expect_rparen("if")
    }
}

/// Evaluate a formula against variable values.
/// A single leading `=` (spreadsheet style) is stripped before parsing.
/// Unknown names, unknown functions, bad syntax, and division
/// by zero all return [`ExprError`].
pub fn evaluate(expr: &str, vars: &HashMap<String, f64>) -> Result<f64, ExprError> {
    let body = strip_leading_equals(expr);
    if body.trim().is_empty() {
        return Err(fail("formula is empty"));
    }
    let mut parser = Parser {
        tokens: tokenize(body)?,
        pos: 0,
        vars,
        depth: 0,
    };
    let value = parser.parse_expr()?;
    match parser.peek() {
        Token::End => Ok(value),
        _ => Err(fail("unexpected text after formula")),
    }
}

/// Strip one spreadsheet style `=` prefix. Only a `=` at the very start
/// (before any other character, whitespace aside) counts; `==` and
/// `=` later in the formula keep their normal meaning.
fn strip_leading_equals(expr: &str) -> &str {
    let trimmed = expr.trim_start();
    if let Some(body) = trimmed.strip_prefix('=') {
        // Keep `==...` (comparison) untouched; it is not a prefix.
        if body.starts_with('=') {
            return trimmed;
        }
        return body;
    }
    trimmed
}

/// Names used in a formula minus known function names.
/// Returns [`ExprError`] on bad syntax. Use it to check a rule
/// provides every variable before calling [`evaluate`].
pub fn required_vars(expr: &str) -> Result<Vec<String>, ExprError> {
    let tokens = tokenize(strip_leading_equals(expr))?;
    let funcs = ["if", "iif", "or", "and", "floor", "ceiling", "ceil"];
    let mut names = Vec::new();
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        if let Token::Ident(name) = token {
            let is_call = matches!(iter.peek(), Some(Token::LParen));
            if is_call {
                if !funcs.contains(&name.as_str()) {
                    return Err(fail(format!("unknown function '{name}'")));
                }
            } else if !names.contains(name) {
                names.push(name.clone());
            }
        }
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> HashMap<String, f64> {
        PayrollVars {
            overtime: 120.0,
            absent: 1.0,
            present: 20.0,
            salary: 4_000_000.0,
            monthly_rate: 4_000_000.0,
            daily_rate: 200_000.0,
            hourly_rate: 25_000.0,
            late: 2.0,
            late_time: 30.0,
            ..Default::default()
        }
        .to_map()
    }

    fn eval(expr: &str) -> f64 {
        evaluate(expr, &vars()).expect("formula should parse")
    }

    #[test]
    fn test_arithmetic_precedence() {
        assert_eq!(eval("2 + 3 * 4"), 14.0);
        assert_eq!(eval("(2 + 3) * 4"), 20.0);
        assert_eq!(eval("10 - 4 / 2"), 8.0);
        assert_eq!(eval("10 % 3"), 1.0);
        assert_eq!(eval("-5 + 8"), 3.0);
    }

    #[test]
    fn test_all_known_vars_resolve() {
        for name in KNOWN_VARS {
            evaluate(name, &vars()).expect("known var should resolve");
        }
        assert_eq!(KNOWN_VARS.len(), 14);
    }

    #[test]
    fn test_comparisons_yield_one_or_zero() {
        assert_eq!(eval("overtime > 1"), 1.0);
        assert_eq!(eval("overtime < 1"), 0.0);
        assert_eq!(eval("overtime >= 2"), 1.0);
        assert_eq!(eval("overtime <= 1"), 0.0);
        assert_eq!(eval("absent == 1"), 1.0);
        assert_eq!(eval("absent != 1"), 0.0);
    }

    #[test]
    fn test_if_picks_branch() {
        assert_eq!(eval("if(absent > 0, 100, 0)"), 100.0);
        assert_eq!(eval("if(absent > 5, 100, 0)"), 0.0);
        // Lazy else branch: no division by zero when cond is true.
        assert_eq!(eval("if(overtime > 0, 1200 / overtime, 0)"), 10.0);
        // Lazy then branch: no division by zero when cond is false.
        assert_eq!(eval("if(absent > 5, 10 / overtime, 7)"), 7.0);
    }

    #[test]
    fn test_or_and() {
        assert_eq!(eval("or(absent > 5, late > 0)"), 1.0);
        assert_eq!(eval("or(absent > 5, late > 5)"), 0.0);
        assert_eq!(eval("and(present > 0, late > 0)"), 1.0);
        assert_eq!(eval("and(present > 0, late > 5)"), 0.0);
        assert_eq!(eval("or(absent > 5, late > 5, overtime > 0)"), 1.0);
    }

    #[test]
    fn test_floor_ceiling() {
        assert_eq!(eval("floor(7.8)"), 7.0);
        assert_eq!(eval("ceiling(7.2)"), 8.0);
        assert_eq!(eval("floor(salary / 173)"), 23121.0);
        assert_eq!(eval("ceiling(late_time / 60)"), 1.0);
    }

    #[test]
    fn test_long_overtime_tiered_rate() {
        // Long overtime: minutes above 240 pay double, rest pays 1.5x.
        // 360 min x 25k/hr blended = (120 min x 25k/60 x 2) + (240 min x 25k/60 x 1.5) = 250k.
        let mut long = vars();
        long.insert("overtime".to_string(), 360.0);
        let pay = evaluate(
            "if(overtime > 240, (overtime - 240) * hourly_rate / 60 * 2 + 240 * hourly_rate / 60 * 1.5, overtime * hourly_rate / 60 * 1.5)",
            &long,
        )
        .unwrap();
        assert_eq!(pay, 250_000.0);
    }

    #[test]
    fn test_long_overtime_short_day_flat() {
        // Short day (180 min): plain 1.5x, no double tier.
        let mut short = vars();
        short.insert("overtime".to_string(), 180.0);
        let pay = evaluate(
            "if(overtime > 240, (overtime - 240) * hourly_rate / 60 * 2 + 240 * hourly_rate / 60 * 1.5, overtime * hourly_rate / 60 * 1.5)",
            &short,
        )
        .unwrap();
        assert_eq!(pay, 112_500.0);
    }

    #[test]
    fn test_bpjs_chain() {
        // BPJS chain on 4M salary: evaluate each contribution, then net.
        let mut chained = vars();
        let jht = evaluate("salary * 0.02", &chained).unwrap();
        let jp = evaluate("salary * 0.01", &chained).unwrap();
        let kes = evaluate("salary * 0.01", &chained).unwrap();
        assert_eq!((jht, jp, kes), (80_000.0, 40_000.0, 40_000.0));
        chained.insert("jht".to_string(), jht);
        chained.insert("jp".to_string(), jp);
        chained.insert("kes".to_string(), kes);
        let net = evaluate("salary - jht - jp - kes", &chained).unwrap();
        assert_eq!(net, 3_840_000.0);
    }

    #[test]
    fn test_monthly_minus_absence() {
        assert_eq!(eval("monthly_rate - absent * daily_rate"), 3_800_000.0);
    }

    #[test]
    fn test_daily_present_times_rate() {
        assert_eq!(eval("present * daily_rate"), 4_000_000.0);
    }

    #[test]
    fn test_late_deduction() {
        // 2 lates x 30 min = 1h x hourly rate = 25k deduction.
        assert_eq!(eval("late * (late_time / 60) * hourly_rate"), 25_000.0);
    }

    #[test]
    fn test_required_vars() {
        let mut found = required_vars("monthly_rate - absent * daily_rate").unwrap();
        found.sort();
        assert_eq!(found, vec!["absent", "daily_rate", "monthly_rate"]);
        assert!(required_vars("bogus_fn(1)").is_err());
    }

    #[test]
    fn test_errors() {
        assert!(evaluate("", &vars()).is_err());
        assert!(evaluate("=", &vars()).is_err());
        assert!(evaluate("salary + unknown_var", &vars()).is_err());
        assert!(evaluate("salary / 0", &vars()).is_err());
        assert!(evaluate("bogus(1)", &vars()).is_err());
        assert!(evaluate("salary +", &vars()).is_err());
        assert!(evaluate("salary = 5", &vars()).is_err());
        assert!(evaluate("if(absent > 0, 1)", &vars()).is_err());
        assert!(evaluate("floor(1, 2)", &vars()).is_err());
        assert!(evaluate("(salary", &vars()).is_err());
    }

    #[test]
    fn test_leading_equals_prefix() {
        // Spreadsheet style prefix is stripped before parsing.
        assert_eq!(eval("=salary"), 4_000_000.0);
        assert_eq!(eval("  = salary + 1"), 4_000_001.0);
        assert_eq!(eval("=iif(overtime>=360; daily_rate; 0)"), 0.0);
    }

    #[test]
    fn test_semicolon_separator() {
        // Semicolons work as argument separators.
        assert_eq!(eval("if(absent > 0; 100; 0)"), 100.0);
        assert_eq!(eval("or(absent > 5; late > 5; overtime > 0)"), 1.0);
    }

    #[test]
    fn test_nesting_limit_rejects_deep_formulas() {
        // A 2 MiB worker stack overflows near depth 4000, and Rust aborts the whole
        // process on stack overflow, so the parser must refuse long before that.
        let parens = format!("{}1{}", "(".repeat(10_000), ")".repeat(10_000));
        assert!(evaluate(&parens, &vars()).is_err());
        let unary = format!("{}1", "-".repeat(10_000));
        assert!(evaluate(&unary, &vars()).is_err());
        let conds = format!("{}1{}", "if(1,".repeat(1000), ",0)".repeat(1000));
        assert!(evaluate(&conds, &vars()).is_err());
        // The untaken branch is skipped, not evaluated, but it must still be
        // depth-checked: skip_expr/skip_unary are the same recursion.
        let skipped = format!("if(1, 1, {})", format!("{}1{}", "(".repeat(10_000), ")".repeat(10_000)));
        assert!(evaluate(&skipped, &vars()).is_err());
        // Each '(' costs two guarded levels (expression + unary), so 20 nested
        // parens still parse and the guard does not clip real formulas.
        let ok = format!("{}1{}", "(".repeat(20), ")".repeat(20));
        assert_eq!(evaluate(&ok, &vars()).unwrap(), 1.0);
    }

    #[test]
    fn test_iif_alias() {
        // iif behaves exactly like if, including lazy branches.
        assert_eq!(eval("iif(absent > 0, 100, 0)"), 100.0);
        assert_eq!(eval("iif(absent > 5, 100, 0)"), 0.0);
        assert_eq!(eval("iif(overtime > 0, 1200 / overtime, 0)"), 10.0);
        // User example: 120 min overtime is below the 360 min threshold, so no payout.
        assert_eq!(eval("=iif(overtime>=360; daily_rate; 0)"), 0.0);
        let mut long = vars();
        long.insert("overtime".to_string(), 420.0);
        assert_eq!(
            evaluate("=iif(overtime>=360; daily_rate; 0)", &long).unwrap(),
            200_000.0
        );
        let mut found = required_vars("=iif(overtime>=360; daily_rate; 0)").unwrap();
        found.sort();
        assert_eq!(found, vec!["daily_rate", "overtime"]);
    }
}
