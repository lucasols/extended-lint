//! Parser for the subset of ESQuery selector syntax this crate evaluates. Unsupported syntax is an error.

use regex::Regex;

use super::estree::EsType;

#[derive(Debug)]
pub enum Selector {
    Compound(Vec<Atom>),
    /// `left > right`
    Child(Box<Selector>, Box<Selector>),
    /// `left right`
    Descendant(Box<Selector>, Box<Selector>),
}

#[derive(Debug)]
pub enum Atom {
    Wildcard,
    Type(EsType),
    /// `:function`
    Function,
    Attribute(Attribute),
    /// `.key`: the node is the parent's (or a further ancestor's, for `.a.b`) field.
    Field(Vec<String>),
    Not(Vec<Selector>),
    Matches(Vec<Selector>),
}

#[derive(Debug)]
pub struct Attribute {
    pub path: Vec<String>,
    pub op: AttributeOp,
}

#[derive(Debug)]
pub enum AttributeOp {
    Exists,
    /// `=`/`!=` against a literal, compared as strings like ESQuery does.
    Equals {
        value: String,
        negated: bool,
    },
    Regex {
        regex: Regex,
        negated: bool,
    },
    TypeOf {
        name: String,
        negated: bool,
    },
    Compare {
        op: CompareOp,
        value: Literal,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum CompareOp {
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

#[derive(Debug)]
pub enum Literal {
    Str(String),
    Num(f64),
}

pub fn parse(source: &str) -> Result<Vec<Selector>, String> {
    let mut parser = Parser {
        chars: source.chars().collect(),
        pos: 0,
    };
    parser.skip_whitespace();
    let selectors = parser.selectors()?;
    parser.skip_whitespace();
    if let Some(c) = parser.peek() {
        return Err(parser.error(&format!("unexpected `{c}`")));
    }
    if selectors.is_empty() {
        return Err("empty selector".to_string());
    }
    Ok(selectors)
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

fn is_name_char(c: char) -> bool {
    !c.is_whitespace() && !"[],():#!=><~+.\"'/*".contains(c)
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn error(&self, message: &str) -> String {
        format!("{message} at position {}", self.pos)
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, c: char) -> Result<(), String> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(self.error(&format!("expected `{c}`")))
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        let start = self.pos;
        while self.peek().is_some_and(char::is_whitespace) {
            self.pos += 1;
        }
        self.pos > start
    }

    fn starts_with(&self, text: &str) -> bool {
        text.chars()
            .enumerate()
            .all(|(i, c)| self.chars.get(self.pos + i) == Some(&c))
    }

    fn selectors(&mut self) -> Result<Vec<Selector>, String> {
        let mut selectors = vec![self.selector()?];
        loop {
            let start = self.pos;
            self.skip_whitespace();
            if self.eat(',') {
                self.skip_whitespace();
                selectors.push(self.selector()?);
            } else {
                self.pos = start;
                return Ok(selectors);
            }
        }
    }

    fn selector(&mut self) -> Result<Selector, String> {
        let mut selector = self.sequence()?;
        loop {
            let start = self.pos;
            let had_whitespace = self.skip_whitespace();
            match self.peek() {
                Some('>') => {
                    self.pos += 1;
                    self.skip_whitespace();
                    let right = self.sequence()?;
                    selector = Selector::Child(Box::new(selector), Box::new(right));
                }
                Some('~' | '+') => {
                    return Err(self.error("sibling combinators (`~`, `+`) are not supported"));
                }
                Some(c) if had_whitespace && c != ',' && c != ')' => {
                    let right = self.sequence()?;
                    selector = Selector::Descendant(Box::new(selector), Box::new(right));
                }
                _ => {
                    self.pos = start;
                    return Ok(selector);
                }
            }
        }
    }

    fn sequence(&mut self) -> Result<Selector, String> {
        if self.peek() == Some('!') {
            return Err(self.error("subject indicators (`!`) are not supported"));
        }
        let mut atoms = Vec::new();
        while let Some(atom) = self.atom()? {
            atoms.push(atom);
        }
        if atoms.is_empty() {
            return Err(match self.peek() {
                Some(c) => self.error(&format!("unexpected `{c}`")),
                None => self.error("unexpected end of selector"),
            });
        }
        Ok(Selector::Compound(atoms))
    }

    fn name(&mut self) -> String {
        let start = self.pos;
        while self.peek().is_some_and(is_name_char) {
            self.pos += 1;
        }
        self.chars
            .get(start..self.pos)
            .unwrap_or_default()
            .iter()
            .collect()
    }

    fn atom(&mut self) -> Result<Option<Atom>, String> {
        let Some(c) = self.peek() else {
            return Ok(None);
        };
        let atom = match c {
            '*' => {
                self.pos += 1;
                Atom::Wildcard
            }
            '#' => {
                self.pos += 1;
                self.type_atom()?
            }
            '[' => {
                self.pos += 1;
                Atom::Attribute(self.attribute()?)
            }
            '.' => {
                let mut path = Vec::new();
                while self.eat('.') {
                    let segment = self.name();
                    if segment.is_empty() {
                        return Err(self.error("expected a field name"));
                    }
                    path.push(segment);
                }
                Atom::Field(path)
            }
            ':' => {
                self.pos += 1;
                self.pseudo()?
            }
            c if is_name_char(c) => self.type_atom()?,
            _ => return Ok(None),
        };
        Ok(Some(atom))
    }

    fn type_atom(&mut self) -> Result<Atom, String> {
        let name = self.name();
        if name.is_empty() {
            return Err(self.error("expected a node type"));
        }
        EsType::from_name(&name)
            .map(Atom::Type)
            .ok_or_else(|| format!("unsupported node type `{name}`"))
    }

    fn pseudo(&mut self) -> Result<Atom, String> {
        let name = self.name();
        match name.as_str() {
            "not" | "matches" | "is" => {
                self.expect('(')?;
                self.skip_whitespace();
                let selectors = self.selectors()?;
                self.skip_whitespace();
                self.expect(')')?;
                Ok(if name == "not" {
                    Atom::Not(selectors)
                } else {
                    Atom::Matches(selectors)
                })
            }
            "function" => Ok(Atom::Function),
            "has" | "first-child" | "last-child" | "nth-child" | "nth-last-child" | "statement"
            | "expression" | "declaration" | "pattern" => {
                Err(format!("`:{name}` is not supported"))
            }
            _ => Err(format!("unknown pseudo-class `:{name}`")),
        }
    }

    fn attribute(&mut self) -> Result<Attribute, String> {
        self.skip_whitespace();
        let mut path = vec![self.name()];
        while self.eat('.') {
            path.push(self.name());
        }
        if path.iter().any(String::is_empty) {
            return Err(self.error("expected an attribute name"));
        }
        self.skip_whitespace();
        let op = if self.eat(']') {
            return Ok(Attribute {
                path,
                op: AttributeOp::Exists,
            });
        } else if self.eat('=') {
            self.equality_value(false)?
        } else if self.starts_with("!=") {
            self.pos += 2;
            self.equality_value(true)?
        } else if self.starts_with("<=") || self.starts_with(">=") {
            let op = if self.eat('<') {
                CompareOp::LessOrEqual
            } else {
                self.pos += 1;
                CompareOp::GreaterOrEqual
            };
            self.pos += 1;
            self.skip_whitespace();
            AttributeOp::Compare {
                op,
                value: self.literal()?,
            }
        } else if self.eat('<') || self.eat('>') {
            let op = if self.chars.get(self.pos - 1) == Some(&'<') {
                CompareOp::Less
            } else {
                CompareOp::Greater
            };
            self.skip_whitespace();
            AttributeOp::Compare {
                op,
                value: self.literal()?,
            }
        } else {
            return Err(self.error("expected an attribute operator"));
        };
        self.skip_whitespace();
        self.expect(']')?;
        Ok(Attribute { path, op })
    }

    fn equality_value(&mut self, negated: bool) -> Result<AttributeOp, String> {
        self.skip_whitespace();
        if self.starts_with("type(") {
            self.pos += 5;
            let start = self.pos;
            while self.peek().is_some_and(|c| c != ')') {
                self.pos += 1;
            }
            let name: String = self
                .chars
                .get(start..self.pos)
                .unwrap_or_default()
                .iter()
                .collect();
            self.expect(')')?;
            return Ok(AttributeOp::TypeOf {
                name: name.trim().to_string(),
                negated,
            });
        }
        if self.eat('/') {
            let start = self.pos;
            while self.peek().is_some_and(|c| c != '/') {
                self.pos += 1;
            }
            let pattern: String = self
                .chars
                .get(start..self.pos)
                .unwrap_or_default()
                .iter()
                .collect();
            self.expect('/')?;
            let mut flags = String::new();
            while let Some(c) = self.peek().filter(|c| "imsu".contains(*c)) {
                flags.push(c);
                self.pos += 1;
            }
            let regex = js_regex(&pattern, &flags)?;
            return Ok(AttributeOp::Regex { regex, negated });
        }
        let value = match self.literal()? {
            Literal::Str(s) => s,
            Literal::Num(n) => js_number_to_string(n),
        };
        Ok(AttributeOp::Equals { value, negated })
    }

    fn literal(&mut self) -> Result<Literal, String> {
        match self.peek() {
            Some(quote @ ('"' | '\'')) => {
                self.pos += 1;
                let mut value = String::new();
                loop {
                    match self.peek() {
                        None => return Err(self.error("unterminated string")),
                        Some(c) if c == quote => {
                            self.pos += 1;
                            return Ok(Literal::Str(value));
                        }
                        Some('\\') => {
                            self.pos += 1;
                            let escaped = self
                                .peek()
                                .ok_or_else(|| self.error("unterminated string"))?;
                            self.pos += 1;
                            value.push(match escaped {
                                'b' => '\u{8}',
                                'f' => '\u{c}',
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                'v' => '\u{b}',
                                c => c,
                            });
                        }
                        Some(c) => {
                            value.push(c);
                            self.pos += 1;
                        }
                    }
                }
            }
            Some(c) if c.is_ascii_digit() || c == '.' => {
                let start = self.pos;
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '.') {
                    self.pos += 1;
                }
                let text: String = self
                    .chars
                    .get(start..self.pos)
                    .unwrap_or_default()
                    .iter()
                    .collect();
                text.parse()
                    .map(Literal::Num)
                    .map_err(|_| self.error(&format!("invalid number `{text}`")))
            }
            _ => {
                let name = self.name();
                if name.is_empty() {
                    Err(self.error("expected a value"))
                } else {
                    Ok(Literal::Str(name))
                }
            }
        }
    }
}

/// A JS regex (without the slashes) as a Rust regex. JS-only syntax (lookarounds, backreferences) is an error.
pub fn js_regex(pattern: &str, flags: &str) -> Result<Regex, String> {
    let mut inline = String::new();
    for flag in flags.chars() {
        match flag {
            'i' | 'm' | 's' => inline.push(flag),
            'u' | 'g' | 'y' | 'd' | 'v' => {}
            _ => return Err(format!("unsupported regex flag `{flag}`")),
        }
    }
    let source = if inline.is_empty() {
        pattern.to_string()
    } else {
        format!("(?{inline}){pattern}")
    };
    Regex::new(&source).map_err(|e| format!("invalid regex `/{pattern}/{flags}`: {e}"))
}

/// `String(n)` for the numbers selectors and literals hold.
pub fn js_number_to_string(n: f64) -> String {
    if n.is_nan() {
        return "NaN".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if n == n.trunc() && n.abs() < 1e21 {
        return format!("{n:.0}");
    }
    format!("{n}")
}
