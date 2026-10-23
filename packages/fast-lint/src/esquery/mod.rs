//! ESQuery selectors (the syntax of ESLint's `no-restricted-syntax`) evaluated over an ESTree-shaped view of the
//! TS AST.
//!
//! Selectors are compiled once and indexed by the ESTree type of their subject (the rightmost compound), so a TS
//! node only runs the selectors that can match it, and a rule only listens to the TS kinds that can produce those
//! types. `Identifier[name="x"]` and `Literal[value="x"]` subjects are indexed by name/value: each identifier or
//! literal costs one hash lookup however many such selectors there are. Combinators are evaluated from the subject
//! up: `A B` walks the subject's ancestors only once the subject matched, skipping the ancestors whose TS kind
//! can't produce `A`.
//!
//! Syntax, node types and fields the view doesn't support are errors when the selector is compiled, so a selector
//! never silently matches nothing because of a gap in the view.

pub mod estree;
pub mod parse;

use std::borrow::Cow;
use std::collections::BTreeSet;

use rustc_hash::FxHashMap;
use tsrs_ast::{Kind, Node, SourceFile};
use tsrs_core::P;

use estree::{EsNode, EsType, EsValue, ALL_TYPES};
use parse::{Atom, AttributeOp, CompareOp, Literal, Selector};

const KIND_WORDS: usize = (Kind::Count as usize).div_ceil(64);

/// A set of TS kinds.
#[derive(Clone)]
pub struct KindSet([u64; KIND_WORDS]);

impl KindSet {
    pub fn empty() -> KindSet {
        KindSet([0; KIND_WORDS])
    }

    pub fn all() -> KindSet {
        KindSet([u64::MAX; KIND_WORDS])
    }

    pub fn insert(&mut self, kind: Kind) {
        let index = kind as usize;
        if let Some(word) = self.0.get_mut(index / 64) {
            *word |= 1 << (index % 64);
        }
    }

    pub fn contains(&self, kind: Kind) -> bool {
        let index = kind as usize;
        self.0
            .get(index / 64)
            .is_some_and(|word| word & (1 << (index % 64)) != 0)
    }

    fn extend(&mut self, other: &KindSet) {
        for (word, other) in self.0.iter_mut().zip(other.0.iter()) {
            *word |= other;
        }
    }

    fn of_types(types: &TypeSet) -> KindSet {
        let Some(types) = types else {
            return KindSet::all();
        };
        let mut kinds = KindSet::empty();
        for t in types {
            for &kind in t.ts_kinds() {
                kinds.insert(kind);
            }
        }
        kinds
    }

    /// The kinds in the set, for registering listeners.
    pub fn kinds(&self) -> Vec<Kind> {
        (0..Kind::Count as i16)
            .map(Kind::from_i16)
            .filter(|&kind| self.contains(kind))
            .collect()
    }
}

/// `None`: any type.
type TypeSet = Option<BTreeSet<EsType>>;

fn intersect(a: TypeSet, b: TypeSet) -> TypeSet {
    match (a, b) {
        (None, other) | (other, None) => other,
        (Some(a), Some(b)) => Some(a.intersection(&b).copied().collect()),
    }
}

fn union(sets: impl IntoIterator<Item = TypeSet>) -> TypeSet {
    let mut all = BTreeSet::new();
    for set in sets {
        all.extend(set?);
    }
    Some(all)
}

const FUNCTION_TYPES: [EsType; 3] = [
    EsType::FunctionDeclaration,
    EsType::FunctionExpression,
    EsType::ArrowFunctionExpression,
];

fn types_with_field(name: &str) -> BTreeSet<EsType> {
    ALL_TYPES
        .iter()
        .copied()
        .filter(|t| t.fields().contains(&name))
        .collect()
}

/// The types a compound's type-like atoms allow (`Identifier`, `:function`, `[type="X"]`, `:matches(...)`).
fn explicit_types(atoms: &[Atom]) -> TypeSet {
    let mut set = None;
    for atom in atoms {
        let types = match atom {
            Atom::Type(t) => Some(BTreeSet::from([*t])),
            Atom::Function => Some(FUNCTION_TYPES.into_iter().collect()),
            Atom::Attribute(attribute) => match (attribute.path.as_slice(), &attribute.op) {
                (
                    [field],
                    AttributeOp::Equals {
                        value,
                        negated: false,
                    },
                ) if field == "type" => EsType::from_name(value).map(|t| BTreeSet::from([t])),
                _ => None,
            },
            Atom::Matches(selectors) => union(selectors.iter().map(subject_types)),
            Atom::Wildcard | Atom::Field(_) | Atom::Not(_) => None,
        };
        set = intersect(set, types);
    }
    set
}

/// The types a compound can match, narrowed by the fields its attributes require.
fn compound_types(atoms: &[Atom]) -> TypeSet {
    let mut set = explicit_types(atoms);
    for atom in atoms {
        let Atom::Attribute(attribute) = atom else {
            continue;
        };
        let Some(first) = attribute.path.first() else {
            continue;
        };
        if first == "type" || first == "parent" {
            continue;
        }
        let requires_field = match &attribute.op {
            AttributeOp::Exists | AttributeOp::Compare { .. } => true,
            AttributeOp::Equals { value, negated } => !negated && value != "undefined",
            AttributeOp::Regex { negated, .. } => !negated,
            AttributeOp::TypeOf { name, negated } => !negated && name != "undefined",
        };
        if requires_field {
            set = intersect(set, Some(types_with_field(first)));
        }
    }
    set
}

fn subject_types(selector: &Selector) -> TypeSet {
    match selector {
        Selector::Compound(atoms) => compound_types(atoms),
        Selector::Child(_, right) | Selector::Descendant(_, right) => subject_types(right),
    }
}

fn validate(selector: &Selector) -> Result<(), String> {
    match selector {
        Selector::Compound(atoms) => validate_compound(atoms),
        Selector::Child(left, right) | Selector::Descendant(left, right) => {
            validate(left)?;
            validate(right)
        }
    }
}

fn validate_compound(atoms: &[Atom]) -> Result<(), String> {
    let explicit = explicit_types(atoms);
    for atom in atoms {
        match atom {
            Atom::Attribute(attribute) => {
                let Some((first, rest)) = attribute.path.split_first() else {
                    continue;
                };
                if first != "type" && first != "parent" {
                    match &explicit {
                        Some(types)
                            if !types.iter().any(|t| t.fields().contains(&first.as_str())) =>
                        {
                            let names: Vec<&str> = types.iter().map(|t| t.name()).collect();
                            return Err(format!(
                                "field `{first}` is not supported on {}",
                                names.join(" | ")
                            ));
                        }
                        None if types_with_field(first).is_empty() => {
                            return Err(format!("unsupported field `{first}`"));
                        }
                        _ => {}
                    }
                }
                if let Some(unknown) = rest.iter().find(|segment| !estree::is_known_field(segment))
                {
                    return Err(format!("unsupported field `{unknown}`"));
                }
                let is_type_path = attribute.path.last().is_some_and(|last| last == "type");
                if let AttributeOp::Equals { value, .. } = &attribute.op {
                    if is_type_path && EsType::from_name(value).is_none() {
                        return Err(format!("unsupported node type `{value}`"));
                    }
                }
            }
            Atom::Field(path) => {
                if let Some(unknown) = path.iter().find(|segment| !estree::is_known_field(segment))
                {
                    return Err(format!("unsupported field `{unknown}`"));
                }
            }
            Atom::Not(selectors) | Atom::Matches(selectors) => {
                for selector in selectors {
                    validate(selector)?;
                }
            }
            Atom::Wildcard | Atom::Type(_) | Atom::Function => {}
        }
    }
    if compound_types(atoms).is_some_and(|types| types.is_empty()) {
        return Err(
            "selector can never match: its node types have none of the required fields".to_string(),
        );
    }
    Ok(())
}

/// A selector compiled for matching: the parse tree plus, for descendant combinators, the TS kinds an ancestor
/// must have to match the left side.
enum Compiled {
    Compound(Vec<CompiledAtom>),
    Child(Box<Compiled>, Box<Compiled>),
    Descendant(Box<Compiled>, KindSet, Box<Compiled>),
}

enum CompiledAtom {
    Simple(Atom),
    Not(Vec<Compiled>),
    Matches(Vec<Compiled>),
}

fn compile(selector: Selector) -> Compiled {
    match selector {
        Selector::Compound(atoms) => Compiled::Compound(
            atoms
                .into_iter()
                .map(|atom| match atom {
                    Atom::Not(selectors) => {
                        CompiledAtom::Not(selectors.into_iter().map(compile).collect())
                    }
                    Atom::Matches(selectors) => {
                        CompiledAtom::Matches(selectors.into_iter().map(compile).collect())
                    }
                    atom => CompiledAtom::Simple(atom),
                })
                .collect(),
        ),
        Selector::Child(left, right) => {
            Compiled::Child(Box::new(compile(*left)), Box::new(compile(*right)))
        }
        Selector::Descendant(left, right) => {
            let kinds = KindSet::of_types(&subject_types(&left));
            Compiled::Descendant(Box::new(compile(*left)), kinds, Box::new(compile(*right)))
        }
    }
}

fn matches(selector: &Compiled, es: EsNode, file: P<SourceFile>) -> bool {
    match selector {
        Compiled::Compound(atoms) => {
            let t = es.es_type();
            atoms.iter().all(|atom| match atom {
                CompiledAtom::Simple(atom) => atom_matches(atom, es, t, file),
                CompiledAtom::Not(selectors) => !selectors.iter().any(|s| matches(s, es, file)),
                CompiledAtom::Matches(selectors) => selectors.iter().any(|s| matches(s, es, file)),
            })
        }
        Compiled::Child(left, right) => {
            matches(right, es, file)
                && estree::parent(es).is_some_and(|parent| matches(left, parent, file))
        }
        Compiled::Descendant(left, kinds, right) => {
            matches(right, es, file)
                && estree::any_ancestor(
                    es,
                    |kind| kinds.contains(kind),
                    |ancestor| matches(left, ancestor, file),
                )
        }
    }
}

fn atom_matches(atom: &Atom, es: EsNode, t: EsType, file: P<SourceFile>) -> bool {
    match atom {
        Atom::Wildcard => true,
        Atom::Type(expected) => t == *expected,
        Atom::Function => FUNCTION_TYPES.contains(&t),
        Atom::Attribute(attribute) => {
            let value = get_path(es, &attribute.path, file);
            attribute_matches(&attribute.op, &value)
        }
        Atom::Field(path) => field_matches(es, path, file),
        Atom::Not(_) | Atom::Matches(_) => false,
    }
}

fn get_path(es: EsNode, path: &[String], file: P<SourceFile>) -> EsValue {
    let mut value = EsValue::Node(es);
    for segment in path {
        if matches!(value, EsValue::Null | EsValue::Undefined) {
            return value;
        }
        value = step(value, segment, file);
    }
    value
}

fn step(value: EsValue, segment: &str, file: P<SourceFile>) -> EsValue {
    match value {
        EsValue::Node(node) => estree::field(node, segment, file),
        EsValue::List(items) => {
            if segment == "length" {
                EsValue::Num(items.len() as f64)
            } else {
                segment
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| items.into_iter().nth(i))
                    .unwrap_or(EsValue::Undefined)
            }
        }
        EsValue::Str(s) if segment == "length" => EsValue::Num(s.encode_utf16().count() as f64),
        EsValue::TemplateValue(node) => match segment {
            "raw" => EsValue::Str(Cow::Borrowed(estree::template_raw(file, node))),
            "cooked" => EsValue::Str(Cow::Borrowed(node.text())),
            _ => EsValue::Undefined,
        },
        EsValue::RegexInfo(node) => {
            let text = node.text();
            let close = text.rfind('/').unwrap_or(text.len());
            match segment {
                "pattern" => EsValue::Str(Cow::Borrowed(text.get(1..close).unwrap_or_default())),
                "flags" => EsValue::Str(Cow::Borrowed(text.get(close + 1..).unwrap_or_default())),
                _ => EsValue::Undefined,
            }
        }
        _ => EsValue::Undefined,
    }
}

/// `${value}` in JS.
fn stringify(value: &EsValue) -> Cow<'static, str> {
    match value {
        EsValue::Str(s) => s.clone(),
        EsValue::Num(n) => Cow::Owned(parse::js_number_to_string(*n)),
        EsValue::Bool(b) => Cow::Borrowed(if *b { "true" } else { "false" }),
        EsValue::BigInt(s) | EsValue::Regex(s) => Cow::Borrowed(s),
        EsValue::Null => Cow::Borrowed("null"),
        EsValue::Undefined => Cow::Borrowed("undefined"),
        EsValue::List(items) => Cow::Owned(
            items
                .iter()
                .map(|item| match item {
                    EsValue::Null | EsValue::Undefined => Cow::Borrowed(""),
                    item => stringify(item),
                })
                .collect::<Vec<_>>()
                .join(","),
        ),
        EsValue::Node(_) | EsValue::TemplateValue(_) | EsValue::RegexInfo(_) => {
            Cow::Borrowed("[object Object]")
        }
    }
}

fn type_of(value: &EsValue) -> &'static str {
    match value {
        EsValue::Str(_) => "string",
        EsValue::Num(_) => "number",
        EsValue::Bool(_) => "boolean",
        EsValue::BigInt(_) => "bigint",
        EsValue::Undefined => "undefined",
        _ => "object",
    }
}

fn to_number(value: &EsValue) -> f64 {
    match value {
        EsValue::Num(n) => *n,
        EsValue::Bool(b) => f64::from(u8::from(*b)),
        EsValue::Null => 0.0,
        EsValue::Str(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                0.0
            } else {
                trimmed.parse().unwrap_or(f64::NAN)
            }
        }
        EsValue::List(items) => match items.as_slice() {
            [] => 0.0,
            [single] => to_number(single),
            _ => f64::NAN,
        },
        _ => f64::NAN,
    }
}

fn attribute_matches(op: &AttributeOp, value: &EsValue) -> bool {
    match op {
        AttributeOp::Exists => !matches!(value, EsValue::Null | EsValue::Undefined),
        AttributeOp::Equals {
            value: expected,
            negated,
        } => (stringify(value) == expected.as_str()) != *negated,
        AttributeOp::Regex { regex, negated } => {
            if *negated {
                !regex.is_match(&stringify(value))
            } else {
                matches!(value, EsValue::Str(s) if regex.is_match(s))
            }
        }
        AttributeOp::TypeOf { name, negated } => (type_of(value) == name) != *negated,
        AttributeOp::Compare {
            op,
            value: expected,
        } => {
            let ordering = match (expected, value) {
                (Literal::Str(expected), EsValue::Str(actual)) => {
                    Some(actual.as_ref().cmp(expected.as_str()))
                }
                (Literal::Str(expected), actual) => to_number(actual)
                    .partial_cmp(&to_number(&EsValue::Str(Cow::Owned(expected.clone())))),
                (Literal::Num(expected), actual) => to_number(actual).partial_cmp(expected),
            };
            ordering.is_some_and(|ordering| match op {
                CompareOp::Less => ordering.is_lt(),
                CompareOp::LessOrEqual => ordering.is_le(),
                CompareOp::Greater => ordering.is_gt(),
                CompareOp::GreaterOrEqual => ordering.is_ge(),
            })
        }
    }
}

/// ESQuery's field selector: `node` is `ancestor[path[0]][path[1]]...` (or in an array on the way), where the
/// ancestor is `path.len()` levels up.
fn field_matches(es: EsNode, path: &[String], file: P<SourceFile>) -> bool {
    let mut ancestor = Some(es);
    for _ in 0..path.len() {
        ancestor = ancestor.and_then(estree::parent);
    }
    let Some(ancestor) = ancestor else {
        return false;
    };
    in_path(es, EsValue::Node(ancestor), path, file)
}

fn in_path(es: EsNode, current: EsValue, path: &[String], file: P<SourceFile>) -> bool {
    let Some((segment, rest)) = path.split_first() else {
        return matches!(current, EsValue::Node(node) if node == es);
    };
    let EsValue::Node(node) = current else {
        return false;
    };
    match estree::field(node, segment, file) {
        EsValue::List(items) => items.into_iter().any(|item| in_path(es, item, rest, file)),
        value => in_path(es, value, rest, file),
    }
}

/// A compiled comma-separated selector list: the node matches when any alternative does.
pub struct CompiledSelector {
    alternatives: Vec<(Compiled, Subject)>,
}

enum Subject {
    Types(BTreeSet<EsType>),
    IdentifierName(String),
    LiteralValue(String),
}

fn indexed_subject(selector: &Selector) -> Option<Subject> {
    let atoms = match selector {
        Selector::Compound(atoms) => atoms,
        Selector::Child(_, right) | Selector::Descendant(_, right) => {
            return indexed_subject(right)
        }
    };
    let t = atoms.iter().find_map(|atom| match atom {
        Atom::Type(t) => Some(*t),
        _ => None,
    })?;
    let field = match t {
        EsType::Identifier => "name",
        EsType::Literal => "value",
        _ => return None,
    };
    atoms.iter().find_map(|atom| match atom {
        Atom::Attribute(attribute) => match (attribute.path.as_slice(), &attribute.op) {
            (
                [name],
                AttributeOp::Equals {
                    value,
                    negated: false,
                },
            ) if name == field => Some(if t == EsType::Identifier {
                Subject::IdentifierName(value.clone())
            } else {
                Subject::LiteralValue(value.clone())
            }),
            _ => None,
        },
        _ => None,
    })
}

impl CompiledSelector {
    pub fn new(source: &str) -> Result<CompiledSelector, String> {
        let selectors =
            parse::parse(source).map_err(|e| format!("invalid selector \"{source}\": {e}"))?;
        let mut alternatives = Vec::new();
        for selector in selectors {
            validate(&selector).map_err(|e| format!("invalid selector \"{source}\": {e}"))?;
            let subject = match indexed_subject(&selector) {
                Some(subject) => subject,
                None => match subject_types(&selector) {
                    Some(types) => Subject::Types(types),
                    None => {
                        return Err(format!(
                            "invalid selector \"{source}\": the matched node must be narrowed to node types \
                             (a type, `:function`, `[type=...]` or an attribute of specific node types)"
                        ));
                    }
                },
            };
            alternatives.push((compile(selector), subject));
        }
        Ok(CompiledSelector { alternatives })
    }
}

/// Selectors indexed by subject. `O` identifies what a selector belongs to (a configured entry).
pub struct SelectorIndex<O> {
    alternatives: Vec<(Compiled, O)>,
    by_type: Vec<Vec<usize>>,
    identifiers_by_name: FxHashMap<String, Vec<usize>>,
    literals_by_value: FxHashMap<String, Vec<usize>>,
    kinds: KindSet,
}

impl<O: Copy + PartialEq> Default for SelectorIndex<O> {
    fn default() -> Self {
        SelectorIndex::new()
    }
}

impl<O: Copy + PartialEq> SelectorIndex<O> {
    pub fn new() -> SelectorIndex<O> {
        SelectorIndex {
            alternatives: Vec::new(),
            by_type: vec![Vec::new(); ALL_TYPES.len()],
            identifiers_by_name: FxHashMap::default(),
            literals_by_value: FxHashMap::default(),
            kinds: KindSet::empty(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.alternatives.is_empty()
    }

    pub fn kinds(&self) -> &KindSet {
        &self.kinds
    }

    pub fn add(&mut self, selector: CompiledSelector, owner: O) {
        for (compiled, subject) in selector.alternatives {
            let index = self.alternatives.len();
            self.alternatives.push((compiled, owner));
            match subject {
                Subject::Types(types) => {
                    self.kinds.extend(&KindSet::of_types(&Some(types.clone())));
                    for t in types {
                        if let Some(entries) = self.by_type.get_mut(t.index()) {
                            entries.push(index);
                        }
                    }
                }
                Subject::IdentifierName(name) => {
                    self.identifiers_by_name
                        .entry(name)
                        .or_default()
                        .push(index);
                    self.kinds.insert(Kind::Identifier);
                }
                Subject::LiteralValue(value) => {
                    self.literals_by_value.entry(value).or_default().push(index);
                    for &kind in EsType::Literal.ts_kinds() {
                        self.kinds.insert(kind);
                    }
                }
            }
        }
    }

    /// Whether `Program` selectors are configured (run on the source file at its end).
    pub fn has_program_selectors(&self) -> bool {
        self.by_type
            .get(EsType::Program.index())
            .is_some_and(|entries| !entries.is_empty())
    }

    /// Runs the selectors that can match the ESTree nodes of `node`, calling `on_match` once per matching owner
    /// and ESTree node.
    pub fn visit(&self, node: P<Node>, file: P<SourceFile>, mut on_match: impl FnMut(O, EsNode)) {
        let kind = node.kind();
        let named = match kind {
            Kind::Identifier if !self.identifiers_by_name.is_empty() => {
                self.identifiers_by_name.get(node.text())
            }
            _ => None,
        };
        if kind == Kind::Identifier && named.is_none() && !self.has_identifier_type_selectors() {
            return;
        }
        let valued = match kind {
            _ if self.literals_by_value.is_empty() => None,
            Kind::StringLiteral
            | Kind::NumericLiteral
            | Kind::BigIntLiteral
            | Kind::RegularExpressionLiteral
            | Kind::TrueKeyword
            | Kind::FalseKeyword
            | Kind::NullKeyword => self.literals_by_value.get(literal_key(node).as_ref()),
            _ => None,
        };
        let stack = estree::stack(node);
        let mut matched: Vec<O> = Vec::new();
        for (level, &t) in stack.types().iter().enumerate() {
            let es = EsNode {
                node,
                level: u8::try_from(level).unwrap_or(0),
            };
            matched.clear();
            let by_type = self.by_type.get(t.index()).map_or(&[][..], Vec::as_slice);
            let extra = match t {
                EsType::Identifier => named.map_or(&[][..], Vec::as_slice),
                EsType::Literal => valued.map_or(&[][..], Vec::as_slice),
                _ => &[],
            };
            for &index in by_type.iter().chain(extra) {
                let Some((selector, owner)) = self.alternatives.get(index) else {
                    continue;
                };
                if matched.contains(owner) || !matches(selector, es, file) {
                    continue;
                }
                matched.push(*owner);
                on_match(*owner, es);
            }
        }
    }

    fn has_identifier_type_selectors(&self) -> bool {
        [
            EsType::Identifier,
            EsType::JSXIdentifier,
            EsType::ImportDefaultSpecifier,
        ]
        .iter()
        .any(|t| {
            self.by_type
                .get(t.index())
                .is_some_and(|entries| !entries.is_empty())
        })
    }
}

/// The key a literal is indexed by: its value as ESQuery compares it, `${value}`.
fn literal_key(node: P<Node>) -> Cow<'static, str> {
    match node.kind() {
        Kind::StringLiteral | Kind::RegularExpressionLiteral => Cow::Borrowed(node.text()),
        Kind::NumericLiteral => Cow::Owned(parse::js_number_to_string(estree::parse_number(
            node.text(),
        ))),
        Kind::BigIntLiteral => Cow::Borrowed(node.text().trim_end_matches('n')),
        Kind::TrueKeyword => Cow::Borrowed("true"),
        Kind::FalseKeyword => Cow::Borrowed("false"),
        _ => Cow::Borrowed("null"),
    }
}
