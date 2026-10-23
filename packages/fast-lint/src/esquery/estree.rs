//! An ESTree-shaped view of the TypeScript AST, enough to evaluate ESQuery selectors written for ESLint.
//!
//! A TS node maps to a stack of zero or more ESTree nodes, outermost first: zero for nodes ESTree doesn't have
//! (parentheses, `TemplateSpan`, `ImportClause`, ...), several where ESTree has nested nodes over one TS node
//! (`export function f() {}` is an `ExportNamedDeclaration` holding a `FunctionDeclaration`, `a?.b` a
//! `ChainExpression` holding a `MemberExpression`, a class a `ClassDeclaration` holding a `ClassBody`). An ESTree
//! node is a TS node plus its level in that stack.

use std::borrow::Cow;

use tsrs_ast::{self as ast, Kind, ModifierFlags, Node, NodeFlags, SourceFile};
use tsrs_core::P;
use tsrslint::utils::trim_node_text_range;

macro_rules! es_types {
    ($($name:ident),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
        pub enum EsType {
            $($name,)*
            /// A node ESTree has but this view doesn't model: it can't be selected by type, has no fields, but is
            /// still a node of the tree (a parent, an ancestor).
            Other,
        }

        pub const ALL_TYPES: &[EsType] = &[$(EsType::$name,)*];

        impl EsType {
            pub fn name(self) -> &'static str {
                match self {
                    $(EsType::$name => stringify!($name),)*
                    EsType::Other => "",
                }
            }
        }
    };
}

es_types! {
    Program,
    Identifier,
    PrivateIdentifier,
    Literal,
    TemplateLiteral,
    TemplateElement,
    TaggedTemplateExpression,
    ThisExpression,
    Super,
    ArrayExpression,
    ArrayPattern,
    ObjectExpression,
    ObjectPattern,
    Property,
    FunctionDeclaration,
    FunctionExpression,
    ArrowFunctionExpression,
    TSDeclareFunction,
    ClassDeclaration,
    ClassExpression,
    ClassBody,
    MethodDefinition,
    PropertyDefinition,
    StaticBlock,
    VariableDeclaration,
    VariableDeclarator,
    ExpressionStatement,
    BlockStatement,
    EmptyStatement,
    DebuggerStatement,
    IfStatement,
    SwitchStatement,
    SwitchCase,
    WhileStatement,
    DoWhileStatement,
    ForStatement,
    ForInStatement,
    ForOfStatement,
    ReturnStatement,
    ThrowStatement,
    BreakStatement,
    ContinueStatement,
    LabeledStatement,
    WithStatement,
    TryStatement,
    CatchClause,
    UnaryExpression,
    UpdateExpression,
    BinaryExpression,
    LogicalExpression,
    AssignmentExpression,
    AssignmentPattern,
    RestElement,
    SpreadElement,
    SequenceExpression,
    ConditionalExpression,
    CallExpression,
    NewExpression,
    ImportExpression,
    MemberExpression,
    ChainExpression,
    AwaitExpression,
    YieldExpression,
    MetaProperty,
    Decorator,
    ImportDeclaration,
    ImportSpecifier,
    ImportDefaultSpecifier,
    ImportNamespaceSpecifier,
    ExportNamedDeclaration,
    ExportDefaultDeclaration,
    ExportAllDeclaration,
    ExportSpecifier,
    JSXElement,
    JSXOpeningElement,
    JSXClosingElement,
    JSXFragment,
    JSXOpeningFragment,
    JSXClosingFragment,
    JSXAttribute,
    JSXSpreadAttribute,
    JSXExpressionContainer,
    JSXText,
    JSXIdentifier,
    JSXMemberExpression,
    JSXNamespacedName,
    TSNonNullExpression,
    TSAsExpression,
    TSSatisfiesExpression,
    TSTypeAssertion,
    TSQualifiedName,
    TSTypeReference,
    TSTypeParameter,
    TSInterfaceDeclaration,
    TSInterfaceBody,
    TSTypeAliasDeclaration,
    TSEnumDeclaration,
    TSEnumMember,
    TSParameterProperty,
}

impl EsType {
    /// ESQuery matches type names case-insensitively.
    pub fn from_name(name: &str) -> Option<EsType> {
        ALL_TYPES
            .iter()
            .copied()
            .find(|t| t.name().eq_ignore_ascii_case(name))
    }

    pub fn index(self) -> usize {
        self as usize
    }

    /// The fields this view provides for the type, besides `type` and `parent`.
    pub fn fields(self) -> &'static [&'static str] {
        use EsType as T;
        match self {
            T::Program | T::BlockStatement | T::ClassBody | T::StaticBlock | T::TSInterfaceBody => {
                &["body"]
            }
            T::Identifier | T::PrivateIdentifier | T::JSXIdentifier => &["name"],
            T::Literal => &["value", "raw", "regex", "bigint"],
            T::TemplateLiteral => &["quasis", "expressions"],
            T::TemplateElement => &["value", "tail"],
            T::TaggedTemplateExpression => &["tag", "quasi"],
            T::ThisExpression
            | T::Super
            | T::EmptyStatement
            | T::DebuggerStatement
            | T::JSXOpeningFragment
            | T::JSXClosingFragment
            | T::Other => &[],
            T::ArrayExpression | T::ArrayPattern => &["elements"],
            T::ObjectExpression | T::ObjectPattern => &["properties"],
            T::Property => &["key", "value", "kind", "method", "shorthand", "computed"],
            T::FunctionDeclaration
            | T::FunctionExpression
            | T::ArrowFunctionExpression
            | T::TSDeclareFunction => &["id", "params", "body", "async", "generator", "expression"],
            T::ClassDeclaration | T::ClassExpression => {
                &["id", "superClass", "body", "abstract", "decorators"]
            }
            T::MethodDefinition => &["key", "value", "kind", "static", "computed", "decorators"],
            T::PropertyDefinition => &[
                "key",
                "value",
                "static",
                "computed",
                "readonly",
                "decorators",
            ],
            T::VariableDeclaration => &["declarations", "kind", "declare"],
            T::VariableDeclarator => &["id", "init", "definite"],
            T::ExpressionStatement | T::ChainExpression | T::Decorator | T::TSNonNullExpression => {
                &["expression"]
            }
            T::JSXExpressionContainer => &["expression"],
            T::IfStatement | T::ConditionalExpression => &["test", "consequent", "alternate"],
            T::SwitchStatement => &["discriminant", "cases"],
            T::SwitchCase => &["test", "consequent"],
            T::WhileStatement | T::DoWhileStatement => &["test", "body"],
            T::ForStatement => &["init", "test", "update", "body"],
            T::ForInStatement => &["left", "right", "body"],
            T::ForOfStatement => &["left", "right", "body", "await"],
            T::ReturnStatement
            | T::ThrowStatement
            | T::RestElement
            | T::SpreadElement
            | T::AwaitExpression
            | T::JSXSpreadAttribute => &["argument"],
            T::BreakStatement | T::ContinueStatement => &["label"],
            T::LabeledStatement => &["label", "body"],
            T::WithStatement => &["object", "body"],
            T::TryStatement => &["block", "handler", "finalizer"],
            T::CatchClause => &["param", "body"],
            T::UnaryExpression | T::UpdateExpression => &["operator", "argument", "prefix"],
            T::BinaryExpression | T::LogicalExpression | T::AssignmentExpression => {
                &["operator", "left", "right"]
            }
            T::AssignmentPattern => &["left", "right"],
            T::SequenceExpression => &["expressions"],
            T::CallExpression => &["callee", "arguments", "optional"],
            T::NewExpression => &["callee", "arguments"],
            T::ImportExpression => &["source", "options"],
            T::MemberExpression => &["object", "property", "computed", "optional"],
            T::YieldExpression => &["argument", "delegate"],
            T::MetaProperty => &["property"],
            T::ImportDeclaration => &["specifiers", "source", "importKind"],
            T::ImportSpecifier => &["imported", "local", "importKind"],
            T::ImportDefaultSpecifier | T::ImportNamespaceSpecifier => &["local"],
            T::ExportNamedDeclaration => &["declaration", "specifiers", "source", "exportKind"],
            T::ExportDefaultDeclaration => &["declaration"],
            T::ExportAllDeclaration => &["exported", "source", "exportKind"],
            T::ExportSpecifier => &["local", "exported", "exportKind"],
            T::JSXElement => &["openingElement", "closingElement", "children"],
            T::JSXOpeningElement => &["name", "attributes", "selfClosing"],
            T::JSXClosingElement => &["name"],
            T::JSXFragment => &["openingFragment", "closingFragment", "children"],
            T::JSXAttribute => &["name", "value"],
            T::JSXText => &["value", "raw"],
            T::JSXMemberExpression => &["object", "property"],
            T::JSXNamespacedName => &["namespace", "name"],
            T::TSAsExpression | T::TSSatisfiesExpression | T::TSTypeAssertion => {
                &["expression", "typeAnnotation"]
            }
            T::TSQualifiedName => &["left", "right"],
            T::TSTypeReference => &["typeName"],
            T::TSTypeParameter => &["name", "constraint", "default"],
            T::TSInterfaceDeclaration => &["id", "body", "declare"],
            T::TSTypeAliasDeclaration => &["id", "typeAnnotation", "declare"],
            T::TSEnumDeclaration => &["id", "members", "const", "declare"],
            T::TSEnumMember => &["id", "initializer"],
            T::TSParameterProperty => &[
                "parameter",
                "accessibility",
                "readonly",
                "override",
                "static",
            ],
        }
    }

    /// The TS kinds whose stack can hold this type.
    pub fn ts_kinds(self) -> &'static [Kind] {
        use EsType as T;
        match self {
            T::Program => &[Kind::SourceFile],
            T::Identifier => &[Kind::Identifier],
            T::PrivateIdentifier => &[Kind::PrivateIdentifier],
            T::Literal => &[
                Kind::StringLiteral,
                Kind::NumericLiteral,
                Kind::BigIntLiteral,
                Kind::RegularExpressionLiteral,
                Kind::TrueKeyword,
                Kind::FalseKeyword,
                Kind::NullKeyword,
            ],
            T::TemplateLiteral => &[
                Kind::TemplateExpression,
                Kind::NoSubstitutionTemplateLiteral,
            ],
            T::TemplateElement => &[
                Kind::NoSubstitutionTemplateLiteral,
                Kind::TemplateHead,
                Kind::TemplateMiddle,
                Kind::TemplateTail,
            ],
            T::TaggedTemplateExpression => &[Kind::TaggedTemplateExpression],
            T::ThisExpression => &[Kind::ThisKeyword],
            T::Super => &[Kind::SuperKeyword],
            T::ArrayExpression => &[Kind::ArrayLiteralExpression],
            T::ArrayPattern => &[Kind::ArrayLiteralExpression, Kind::ArrayBindingPattern],
            T::ObjectExpression => &[Kind::ObjectLiteralExpression],
            T::ObjectPattern => &[Kind::ObjectLiteralExpression, Kind::ObjectBindingPattern],
            T::Property => &[
                Kind::PropertyAssignment,
                Kind::ShorthandPropertyAssignment,
                Kind::MethodDeclaration,
                Kind::GetAccessor,
                Kind::SetAccessor,
                Kind::BindingElement,
            ],
            T::FunctionDeclaration | T::TSDeclareFunction => &[Kind::FunctionDeclaration],
            T::FunctionExpression => &[
                Kind::FunctionExpression,
                Kind::MethodDeclaration,
                Kind::Constructor,
                Kind::GetAccessor,
                Kind::SetAccessor,
            ],
            T::ArrowFunctionExpression => &[Kind::ArrowFunction],
            T::ClassDeclaration => &[Kind::ClassDeclaration],
            T::ClassExpression => &[Kind::ClassExpression],
            T::ClassBody => &[Kind::ClassDeclaration, Kind::ClassExpression],
            T::MethodDefinition => &[
                Kind::MethodDeclaration,
                Kind::Constructor,
                Kind::GetAccessor,
                Kind::SetAccessor,
            ],
            T::PropertyDefinition => &[Kind::PropertyDeclaration],
            T::StaticBlock => &[Kind::ClassStaticBlockDeclaration],
            T::VariableDeclaration => &[Kind::VariableDeclarationList],
            T::VariableDeclarator => &[Kind::VariableDeclaration],
            T::ExpressionStatement => &[Kind::ExpressionStatement],
            T::BlockStatement => &[Kind::Block],
            T::EmptyStatement => &[Kind::EmptyStatement],
            T::DebuggerStatement => &[Kind::DebuggerStatement],
            T::IfStatement => &[Kind::IfStatement],
            T::SwitchStatement => &[Kind::SwitchStatement],
            T::SwitchCase => &[Kind::CaseClause, Kind::DefaultClause],
            T::WhileStatement => &[Kind::WhileStatement],
            T::DoWhileStatement => &[Kind::DoStatement],
            T::ForStatement => &[Kind::ForStatement],
            T::ForInStatement => &[Kind::ForInStatement],
            T::ForOfStatement => &[Kind::ForOfStatement],
            T::ReturnStatement => &[Kind::ReturnStatement],
            T::ThrowStatement => &[Kind::ThrowStatement],
            T::BreakStatement => &[Kind::BreakStatement],
            T::ContinueStatement => &[Kind::ContinueStatement],
            T::LabeledStatement => &[Kind::LabeledStatement],
            T::WithStatement => &[Kind::WithStatement],
            T::TryStatement => &[Kind::TryStatement],
            T::CatchClause => &[Kind::CatchClause],
            T::UnaryExpression => &[
                Kind::PrefixUnaryExpression,
                Kind::DeleteExpression,
                Kind::TypeOfExpression,
                Kind::VoidExpression,
            ],
            T::UpdateExpression => &[Kind::PrefixUnaryExpression, Kind::PostfixUnaryExpression],
            T::BinaryExpression
            | T::LogicalExpression
            | T::AssignmentExpression
            | T::SequenceExpression => &[Kind::BinaryExpression],
            T::AssignmentPattern => &[
                Kind::BinaryExpression,
                Kind::ShorthandPropertyAssignment,
                Kind::BindingElement,
                Kind::Parameter,
            ],
            T::RestElement => &[
                Kind::SpreadElement,
                Kind::SpreadAssignment,
                Kind::BindingElement,
                Kind::Parameter,
            ],
            T::SpreadElement => &[Kind::SpreadElement, Kind::SpreadAssignment],
            T::ConditionalExpression => &[Kind::ConditionalExpression],
            T::CallExpression | T::ImportExpression => &[Kind::CallExpression],
            T::NewExpression => &[Kind::NewExpression],
            T::MemberExpression => &[
                Kind::PropertyAccessExpression,
                Kind::ElementAccessExpression,
            ],
            T::ChainExpression => &[
                Kind::PropertyAccessExpression,
                Kind::ElementAccessExpression,
                Kind::CallExpression,
                Kind::NonNullExpression,
            ],
            T::AwaitExpression => &[Kind::AwaitExpression],
            T::YieldExpression => &[Kind::YieldExpression],
            T::MetaProperty => &[Kind::MetaProperty],
            T::Decorator => &[Kind::Decorator],
            T::ImportDeclaration => &[Kind::ImportDeclaration],
            T::ImportSpecifier => &[Kind::ImportSpecifier],
            T::ImportDefaultSpecifier => &[Kind::Identifier],
            T::ImportNamespaceSpecifier => &[Kind::NamespaceImport],
            T::ExportNamedDeclaration => &[
                Kind::ExportDeclaration,
                Kind::VariableStatement,
                Kind::FunctionDeclaration,
                Kind::ClassDeclaration,
                Kind::InterfaceDeclaration,
                Kind::TypeAliasDeclaration,
                Kind::EnumDeclaration,
                Kind::ModuleDeclaration,
            ],
            T::ExportDefaultDeclaration => &[
                Kind::ExportAssignment,
                Kind::FunctionDeclaration,
                Kind::ClassDeclaration,
                Kind::InterfaceDeclaration,
            ],
            T::ExportAllDeclaration => &[Kind::ExportDeclaration],
            T::ExportSpecifier => &[Kind::ExportSpecifier],
            T::JSXElement => &[Kind::JsxElement, Kind::JsxSelfClosingElement],
            T::JSXOpeningElement => &[Kind::JsxOpeningElement, Kind::JsxSelfClosingElement],
            T::JSXClosingElement => &[Kind::JsxClosingElement],
            T::JSXFragment => &[Kind::JsxFragment],
            T::JSXOpeningFragment => &[Kind::JsxOpeningFragment],
            T::JSXClosingFragment => &[Kind::JsxClosingFragment],
            T::JSXAttribute => &[Kind::JsxAttribute],
            T::JSXSpreadAttribute => &[Kind::JsxSpreadAttribute],
            T::JSXExpressionContainer => &[Kind::JsxExpression],
            T::JSXText => &[Kind::JsxText, Kind::JsxTextAllWhiteSpaces],
            T::JSXIdentifier => &[Kind::Identifier, Kind::ThisKeyword],
            T::JSXMemberExpression => &[Kind::PropertyAccessExpression],
            T::JSXNamespacedName => &[Kind::JsxNamespacedName],
            T::TSNonNullExpression => &[Kind::NonNullExpression],
            T::TSAsExpression => &[Kind::AsExpression],
            T::TSSatisfiesExpression => &[Kind::SatisfiesExpression],
            T::TSTypeAssertion => &[Kind::TypeAssertionExpression],
            T::TSQualifiedName => &[Kind::QualifiedName],
            T::TSTypeReference => &[Kind::TypeReference],
            T::TSTypeParameter => &[Kind::TypeParameter],
            T::TSInterfaceDeclaration | T::TSInterfaceBody => &[Kind::InterfaceDeclaration],
            T::TSTypeAliasDeclaration => &[Kind::TypeAliasDeclaration],
            T::TSEnumDeclaration => &[Kind::EnumDeclaration],
            T::TSEnumMember => &[Kind::EnumMember],
            T::TSParameterProperty => &[Kind::Parameter],
            T::Other => &[],
        }
    }
}

/// Every field name of every type, plus the keys of the object values (`TemplateElement.value.raw`,
/// `Literal.regex.flags`): the names a path segment after the first may use.
pub fn is_known_field(name: &str) -> bool {
    matches!(
        name,
        "type" | "parent" | "length" | "raw" | "cooked" | "pattern" | "flags"
    ) || name.bytes().all(|b| b.is_ascii_digit())
        || ALL_TYPES.iter().any(|t| t.fields().contains(&name))
}

/// The ESTree nodes of one TS node, outermost first.
#[derive(Clone, Copy)]
pub struct Stack {
    len: u8,
    items: [EsType; 3],
}

impl Stack {
    const EMPTY: Stack = Stack {
        len: 0,
        items: [EsType::Other; 3],
    };

    fn of(types: &[EsType]) -> Stack {
        let mut stack = Stack::EMPTY;
        for &t in types.iter().take(3) {
            stack.push(t);
        }
        stack
    }

    fn push(&mut self, t: EsType) {
        if let Some(slot) = self.items.get_mut(usize::from(self.len)) {
            *slot = t;
            self.len += 1;
        }
    }

    fn prepend(self, t: EsType) -> Stack {
        let mut stack = Stack::of(&[t]);
        for &item in self.types() {
            stack.push(item);
        }
        stack
    }

    pub fn types(&self) -> &[EsType] {
        self.items.get(..usize::from(self.len)).unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        usize::from(self.len)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn get(&self, level: usize) -> EsType {
        self.types().get(level).copied().unwrap_or(EsType::Other)
    }
}

/// An ESTree node: level `level` of `node`'s stack.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EsNode {
    pub node: P<Node>,
    pub level: u8,
}

impl EsNode {
    fn at(node: P<Node>, level: usize) -> EsNode {
        EsNode {
            node,
            level: u8::try_from(level).unwrap_or(0),
        }
    }

    pub fn es_type(self) -> EsType {
        stack(self.node).get(usize::from(self.level))
    }

    fn is_last_level(self) -> bool {
        usize::from(self.level) + 1 >= stack(self.node).len()
    }
}

fn one(t: EsType) -> Stack {
    Stack::of(&[t])
}

pub fn stack(node: P<Node>) -> Stack {
    use EsType as T;
    match node.kind() {
        Kind::SourceFile => one(T::Program),
        Kind::Identifier => identifier_stack(node),
        Kind::PrivateIdentifier => one(T::PrivateIdentifier),
        Kind::StringLiteral
        | Kind::NumericLiteral
        | Kind::BigIntLiteral
        | Kind::RegularExpressionLiteral
        | Kind::TrueKeyword
        | Kind::FalseKeyword => one(T::Literal),
        Kind::NullKeyword => {
            if node.parent().is_some_and(|p| p.kind() == Kind::LiteralType) {
                one(T::Other)
            } else {
                one(T::Literal)
            }
        }
        Kind::ThisKeyword => {
            if is_jsx_name(node) {
                one(T::JSXIdentifier)
            } else {
                one(T::ThisExpression)
            }
        }
        Kind::SuperKeyword => one(T::Super),
        Kind::NoSubstitutionTemplateLiteral => Stack::of(&[T::TemplateLiteral, T::TemplateElement]),
        Kind::TemplateHead | Kind::TemplateMiddle | Kind::TemplateTail => one(T::TemplateElement),
        Kind::TemplateExpression => one(T::TemplateLiteral),
        Kind::TaggedTemplateExpression => one(T::TaggedTemplateExpression),
        Kind::ArrayLiteralExpression => one(if is_pattern(node) {
            T::ArrayPattern
        } else {
            T::ArrayExpression
        }),
        Kind::ObjectLiteralExpression => one(if is_pattern(node) {
            T::ObjectPattern
        } else {
            T::ObjectExpression
        }),
        Kind::ArrayBindingPattern => one(T::ArrayPattern),
        Kind::ObjectBindingPattern => one(T::ObjectPattern),
        Kind::PropertyAssignment => one(T::Property),
        Kind::ShorthandPropertyAssignment => {
            if node
                .as_shorthand_property_assignment()
                .object_assignment_initializer()
                .is_some()
            {
                Stack::of(&[T::Property, T::AssignmentPattern])
            } else {
                one(T::Property)
            }
        }
        Kind::SpreadElement | Kind::SpreadAssignment => {
            if node.parent().is_some_and(is_pattern) {
                one(T::RestElement)
            } else {
                one(T::SpreadElement)
            }
        }
        Kind::PropertyAccessExpression => {
            if is_jsx_name(node) {
                one(T::JSXMemberExpression)
            } else {
                chain_wrapped(node, T::MemberExpression)
            }
        }
        Kind::ElementAccessExpression => chain_wrapped(node, T::MemberExpression),
        Kind::CallExpression => {
            if node.as_call_expression().expression().kind() == Kind::ImportKeyword {
                one(T::ImportExpression)
            } else {
                chain_wrapped(node, T::CallExpression)
            }
        }
        Kind::NonNullExpression => chain_wrapped(node, T::TSNonNullExpression),
        Kind::NewExpression => one(T::NewExpression),
        Kind::FunctionExpression => one(T::FunctionExpression),
        Kind::ArrowFunction => one(T::ArrowFunctionExpression),
        Kind::DeleteExpression | Kind::TypeOfExpression | Kind::VoidExpression => {
            one(T::UnaryExpression)
        }
        Kind::PrefixUnaryExpression => match node.as_prefix_unary_expression().operator() {
            Kind::PlusPlusToken | Kind::MinusMinusToken => one(T::UpdateExpression),
            _ => one(T::UnaryExpression),
        },
        Kind::PostfixUnaryExpression => one(T::UpdateExpression),
        Kind::AwaitExpression => one(T::AwaitExpression),
        Kind::YieldExpression => one(T::YieldExpression),
        Kind::BinaryExpression => binary_stack(node),
        Kind::ConditionalExpression => one(T::ConditionalExpression),
        Kind::ClassExpression => Stack::of(&[T::ClassExpression, T::ClassBody]),
        Kind::AsExpression => one(T::TSAsExpression),
        Kind::SatisfiesExpression => one(T::TSSatisfiesExpression),
        Kind::TypeAssertionExpression => one(T::TSTypeAssertion),
        Kind::MetaProperty => one(T::MetaProperty),
        Kind::Decorator => one(T::Decorator),
        Kind::ExpressionWithTypeArguments => {
            if is_class_extends(node) {
                Stack::EMPTY
            } else {
                one(T::Other)
            }
        }
        Kind::Block => {
            if node
                .parent()
                .is_some_and(|p| p.kind() == Kind::ClassStaticBlockDeclaration)
            {
                Stack::EMPTY
            } else {
                one(T::BlockStatement)
            }
        }
        Kind::EmptyStatement => one(T::EmptyStatement),
        Kind::VariableStatement => {
            if ast::has_syntactic_modifier(node, ModifierFlags::Export) {
                one(T::ExportNamedDeclaration)
            } else {
                Stack::EMPTY
            }
        }
        Kind::ExpressionStatement => one(T::ExpressionStatement),
        Kind::IfStatement => one(T::IfStatement),
        Kind::DoStatement => one(T::DoWhileStatement),
        Kind::WhileStatement => one(T::WhileStatement),
        Kind::ForStatement => one(T::ForStatement),
        Kind::ForInStatement => one(T::ForInStatement),
        Kind::ForOfStatement => one(T::ForOfStatement),
        Kind::ContinueStatement => one(T::ContinueStatement),
        Kind::BreakStatement => one(T::BreakStatement),
        Kind::ReturnStatement => one(T::ReturnStatement),
        Kind::WithStatement => one(T::WithStatement),
        Kind::SwitchStatement => one(T::SwitchStatement),
        Kind::LabeledStatement => one(T::LabeledStatement),
        Kind::ThrowStatement => one(T::ThrowStatement),
        Kind::TryStatement => one(T::TryStatement),
        Kind::DebuggerStatement => one(T::DebuggerStatement),
        Kind::CaseClause | Kind::DefaultClause => one(T::SwitchCase),
        Kind::CatchClause => one(T::CatchClause),
        Kind::VariableDeclaration => {
            if node.parent().is_some_and(|p| p.kind() == Kind::CatchClause) {
                Stack::EMPTY
            } else {
                one(T::VariableDeclarator)
            }
        }
        Kind::VariableDeclarationList => one(T::VariableDeclaration),
        Kind::FunctionDeclaration => export_wrapped(
            node,
            one(if node.body().is_some() {
                T::FunctionDeclaration
            } else {
                T::TSDeclareFunction
            }),
        ),
        Kind::ClassDeclaration => {
            export_wrapped(node, Stack::of(&[T::ClassDeclaration, T::ClassBody]))
        }
        Kind::InterfaceDeclaration => export_wrapped(
            node,
            Stack::of(&[T::TSInterfaceDeclaration, T::TSInterfaceBody]),
        ),
        Kind::TypeAliasDeclaration => export_wrapped(node, one(T::TSTypeAliasDeclaration)),
        Kind::EnumDeclaration => export_wrapped(node, one(T::TSEnumDeclaration)),
        Kind::ModuleDeclaration => export_wrapped(node, one(T::Other)),
        Kind::EnumMember => one(T::TSEnumMember),
        Kind::ImportDeclaration => one(T::ImportDeclaration),
        Kind::NamespaceImport => one(T::ImportNamespaceSpecifier),
        Kind::ImportSpecifier => one(T::ImportSpecifier),
        Kind::ExportDeclaration => {
            let is_named = node
                .as_export_declaration()
                .export_clause()
                .is_some_and(|clause| clause.kind() == Kind::NamedExports);
            one(if is_named {
                T::ExportNamedDeclaration
            } else {
                T::ExportAllDeclaration
            })
        }
        Kind::ExportSpecifier => one(T::ExportSpecifier),
        Kind::ExportAssignment => {
            if node.as_export_assignment().is_export_equals {
                one(T::Other)
            } else {
                one(T::ExportDefaultDeclaration)
            }
        }
        Kind::Parameter => parameter_stack(node),
        Kind::BindingElement => binding_element_stack(node),
        Kind::PropertyDeclaration => {
            let flags = node.modifier_flags();
            if flags.intersects(ModifierFlags::Abstract | ModifierFlags::Accessor) {
                one(T::Other)
            } else {
                one(T::PropertyDefinition)
            }
        }
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor | Kind::Constructor => {
            method_stack(node)
        }
        Kind::ClassStaticBlockDeclaration => one(T::StaticBlock),
        Kind::JsxElement => one(T::JSXElement),
        Kind::JsxSelfClosingElement => Stack::of(&[T::JSXElement, T::JSXOpeningElement]),
        Kind::JsxOpeningElement => one(T::JSXOpeningElement),
        Kind::JsxClosingElement => one(T::JSXClosingElement),
        Kind::JsxFragment => one(T::JSXFragment),
        Kind::JsxOpeningFragment => one(T::JSXOpeningFragment),
        Kind::JsxClosingFragment => one(T::JSXClosingFragment),
        Kind::JsxAttribute => one(T::JSXAttribute),
        Kind::JsxSpreadAttribute => one(T::JSXSpreadAttribute),
        Kind::JsxExpression => {
            if node.as_jsx_expression().dot_dot_dot_token().is_some() {
                one(T::Other)
            } else {
                one(T::JSXExpressionContainer)
            }
        }
        Kind::JsxText | Kind::JsxTextAllWhiteSpaces => one(T::JSXText),
        Kind::JsxNamespacedName => one(T::JSXNamespacedName),
        Kind::QualifiedName => one(T::TSQualifiedName),
        Kind::TypeReference => one(T::TSTypeReference),
        Kind::TypeParameter => {
            if node.parent().is_some_and(|p| p.kind() == Kind::MappedType) {
                one(T::Other)
            } else {
                one(T::TSTypeParameter)
            }
        }
        Kind::ParenthesizedExpression
        | Kind::ParenthesizedType
        | Kind::TemplateSpan
        | Kind::ImportClause
        | Kind::NamedImports
        | Kind::NamedExports
        | Kind::NamespaceExport
        | Kind::JsxAttributes
        | Kind::CaseBlock
        | Kind::HeritageClause
        | Kind::ComputedPropertyName
        | Kind::OmittedExpression
        | Kind::SemicolonClassElement
        | Kind::EndOfFile => Stack::EMPTY,
        _ => one(T::Other),
    }
}

fn identifier_stack(node: P<Node>) -> Stack {
    let Some(parent) = node.parent() else {
        return one(EsType::Identifier);
    };
    match parent.kind() {
        Kind::ImportClause => Stack::of(&[EsType::ImportDefaultSpecifier, EsType::Identifier]),
        Kind::JsxAttribute | Kind::JsxNamespacedName => one(EsType::JSXIdentifier),
        Kind::JsxOpeningElement
        | Kind::JsxClosingElement
        | Kind::JsxSelfClosingElement
        | Kind::PropertyAccessExpression => {
            if is_jsx_name(node) {
                one(EsType::JSXIdentifier)
            } else {
                one(EsType::Identifier)
            }
        }
        _ => one(EsType::Identifier),
    }
}

/// Whether `node` is (part of) a JSX tag name: `<a.b.c>`'s `a`, `a.b`, `b` and `c`.
fn is_jsx_name(node: P<Node>) -> bool {
    let mut current = node;
    loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            Kind::PropertyAccessExpression => current = parent,
            Kind::JsxOpeningElement | Kind::JsxClosingElement | Kind::JsxSelfClosingElement => {
                return parent.tag_name() == current;
            }
            _ => return false,
        }
    }
}

fn is_class_extends(node: P<Node>) -> bool {
    node.parent().is_some_and(|clause| {
        clause.kind() == Kind::HeritageClause
            && clause.as_heritage_clause().token() == Kind::ExtendsKeyword
            && clause
                .parent()
                .is_some_and(|c| matches!(c.kind(), Kind::ClassDeclaration | Kind::ClassExpression))
    })
}

/// Whether an object/array literal (or a `=` default inside one) is part of a destructuring assignment target:
/// `[a = 1] = x`, `({ a: { b } } = x)`, `for ([a] of x)`.
fn is_pattern(node: P<Node>) -> bool {
    let mut current = node;
    loop {
        let Some(parent) = current.parent() else {
            return false;
        };
        match parent.kind() {
            Kind::ArrayLiteralExpression
            | Kind::ObjectLiteralExpression
            | Kind::SpreadElement
            | Kind::SpreadAssignment
            | Kind::ParenthesizedExpression => {}
            Kind::PropertyAssignment => {
                if parent.as_property_assignment().initializer() != current {
                    return false;
                }
            }
            Kind::BinaryExpression => {
                let binary = parent.as_binary_expression();
                return binary.operator_token.kind() == Kind::EqualsToken && binary.left == current;
            }
            Kind::ForInStatement | Kind::ForOfStatement => {
                return parent.as_for_in_or_of_statement().initializer == current;
            }
            _ => return false,
        }
        current = parent;
    }
}

fn chain_wrapped(node: P<Node>, t: EsType) -> Stack {
    if crate::utils::is_estree_chain_expression(node) {
        Stack::of(&[EsType::ChainExpression, t])
    } else {
        one(t)
    }
}

fn export_wrapped(node: P<Node>, inner: Stack) -> Stack {
    if !ast::has_syntactic_modifier(node, ModifierFlags::Export) {
        return inner;
    }
    if ast::has_syntactic_modifier(node, ModifierFlags::Default) {
        inner.prepend(EsType::ExportDefaultDeclaration)
    } else {
        inner.prepend(EsType::ExportNamedDeclaration)
    }
}

fn is_comma(node: P<Node>) -> bool {
    node.kind() == Kind::BinaryExpression
        && node.as_binary_expression().operator_token.kind() == Kind::CommaToken
}

fn binary_stack(node: P<Node>) -> Stack {
    let operator = node.as_binary_expression().operator_token.kind();
    match operator {
        Kind::CommaToken => {
            let flattened = node.parent().is_some_and(|parent| {
                is_comma(parent) && parent.as_binary_expression().left == node
            });
            if flattened {
                Stack::EMPTY
            } else {
                one(EsType::SequenceExpression)
            }
        }
        Kind::EqualsToken if is_pattern(node) => one(EsType::AssignmentPattern),
        Kind::AmpersandAmpersandToken | Kind::BarBarToken | Kind::QuestionQuestionToken => {
            one(EsType::LogicalExpression)
        }
        _ if ast::is_assignment_operator(operator) => one(EsType::AssignmentExpression),
        _ => one(EsType::BinaryExpression),
    }
}

fn parameter_stack(node: P<Node>) -> Stack {
    let parameter = node.as_parameter_declaration();
    let mut stack = Stack::EMPTY;
    if ast::has_syntactic_modifier(node, ModifierFlags::ParameterPropertyModifier) {
        stack.push(EsType::TSParameterProperty);
    }
    if parameter.dot_dot_dot_token().is_some() {
        stack.push(EsType::RestElement);
    } else if parameter.initializer().is_some() {
        stack.push(EsType::AssignmentPattern);
    }
    stack
}

fn binding_element_stack(node: P<Node>) -> Stack {
    let element = node.as_binding_element();
    let in_object = node
        .parent()
        .is_some_and(|p| p.kind() == Kind::ObjectBindingPattern);
    if element.dot_dot_dot_token().is_some() {
        return one(EsType::RestElement);
    }
    let has_default = element.initializer().is_some();
    match (in_object, has_default) {
        (true, true) => Stack::of(&[EsType::Property, EsType::AssignmentPattern]),
        (true, false) => one(EsType::Property),
        (false, true) => one(EsType::AssignmentPattern),
        (false, false) => Stack::EMPTY,
    }
}

fn is_in_object_literal(node: P<Node>) -> bool {
    node.parent()
        .is_some_and(|p| p.kind() == Kind::ObjectLiteralExpression)
}

fn method_stack(node: P<Node>) -> Stack {
    if is_in_object_literal(node) {
        return Stack::of(&[EsType::Property, EsType::FunctionExpression]);
    }
    if ast::has_syntactic_modifier(node, ModifierFlags::Abstract) {
        return Stack::of(&[EsType::Other, EsType::Other]);
    }
    if node.body().is_none() {
        return Stack::of(&[EsType::MethodDefinition, EsType::Other]);
    }
    Stack::of(&[EsType::MethodDefinition, EsType::FunctionExpression])
}

fn is_class_member(node: P<Node>) -> bool {
    ast::is_class_element(node)
}

fn is_interface_member(node: P<Node>) -> bool {
    ast::is_type_element(node)
}

/// The level of `parent`'s stack that its TS child `child` hangs from.
fn attach_level(parent: P<Node>, child: P<Node>, len: usize) -> usize {
    let last = len.saturating_sub(1);
    match parent.kind() {
        Kind::ClassDeclaration | Kind::ClassExpression => {
            if is_class_member(child) {
                last
            } else {
                last.saturating_sub(1)
            }
        }
        Kind::InterfaceDeclaration => {
            if is_interface_member(child) {
                last
            } else {
                last.saturating_sub(1)
            }
        }
        Kind::MethodDeclaration | Kind::GetAccessor | Kind::SetAccessor | Kind::Constructor => {
            if parent.name() == Some(child) || child.kind() == Kind::Decorator {
                0
            } else {
                last
            }
        }
        Kind::BindingElement => {
            if parent.as_binding_element().property_name() == Some(child) {
                0
            } else {
                last
            }
        }
        Kind::Parameter => {
            if child.kind() == Kind::Decorator {
                0
            } else {
                last
            }
        }
        _ => last,
    }
}

/// The ESTree parent of an ESTree node.
pub fn parent(es: EsNode) -> Option<EsNode> {
    if es.level > 0 {
        return Some(EsNode {
            node: es.node,
            level: es.level - 1,
        });
    }
    let mut child = es.node;
    loop {
        let parent = child.parent()?;
        let parent_stack = stack(parent);
        if !parent_stack.is_empty() {
            return Some(EsNode::at(
                parent,
                attach_level(parent, child, parent_stack.len()),
            ));
        }
        child = parent;
    }
}

/// Calls `f` with each ESTree ancestor of `es`, innermost first, until it returns `true`. Ancestors whose TS kind
/// fails `kind_filter` are skipped without computing their stack.
pub fn any_ancestor(
    es: EsNode,
    kind_filter: impl Fn(Kind) -> bool,
    mut f: impl FnMut(EsNode) -> bool,
) -> bool {
    if kind_filter(es.node.kind()) {
        for level in (0..usize::from(es.level)).rev() {
            if f(EsNode::at(es.node, level)) {
                return true;
            }
        }
    }
    let mut child = es.node;
    while let Some(parent) = child.parent() {
        if kind_filter(parent.kind()) {
            let parent_stack = stack(parent);
            if !parent_stack.is_empty() {
                let attach = attach_level(parent, child, parent_stack.len());
                for level in (0..=attach).rev() {
                    if f(EsNode::at(parent, level)) {
                        return true;
                    }
                }
            }
        }
        child = parent;
    }
    false
}

/// The ESTree node of a TS node used as a field value, looking through the nodes ESTree doesn't have.
pub fn node_of(node: P<Node>) -> Option<EsNode> {
    let mut current = node;
    loop {
        if !stack(current).is_empty() {
            return Some(EsNode::at(current, 0));
        }
        current = match current.kind() {
            Kind::ParenthesizedExpression => current.as_parenthesized_expression().expression(),
            Kind::ParenthesizedType => current.type_node()?,
            Kind::ComputedPropertyName => current.as_computed_property_name().expression,
            Kind::Parameter | Kind::BindingElement | Kind::VariableDeclaration => current.name()?,
            Kind::VariableStatement => current.as_variable_statement().declaration_list,
            Kind::ExpressionWithTypeArguments => {
                current.as_expression_with_type_arguments().expression
            }
            _ => return None,
        };
    }
}

#[derive(Clone, Debug)]
pub enum EsValue {
    Node(EsNode),
    List(Vec<EsValue>),
    Str(Cow<'static, str>),
    Num(f64),
    Bool(bool),
    /// A `bigint` literal's value, as its digits.
    BigInt(&'static str),
    /// A regex literal's value (a `RegExp` object, `/a/g` as a string).
    Regex(&'static str),
    /// `TemplateElement.value`: `{ raw, cooked }`.
    TemplateValue(P<Node>),
    /// `Literal.regex`: `{ pattern, flags }`.
    RegexInfo(P<Node>),
    Null,
    Undefined,
}

fn child(node: Option<P<Node>>) -> EsValue {
    node.and_then(node_of).map_or(EsValue::Null, EsValue::Node)
}

fn list(nodes: &[P<Node>]) -> EsValue {
    EsValue::List(nodes.iter().map(|&n| child(Some(n))).collect())
}

fn list_skipping_absent(nodes: &[P<Node>]) -> EsValue {
    EsValue::List(
        nodes
            .iter()
            .filter_map(|&n| node_of(n))
            .map(EsValue::Node)
            .collect(),
    )
}

fn str(s: &'static str) -> EsValue {
    EsValue::Str(Cow::Borrowed(s))
}

fn has_modifier(node: P<Node>, flags: ModifierFlags) -> bool {
    ast::has_syntactic_modifier(node, flags)
}

fn same_level(es: EsNode, level: usize) -> EsValue {
    EsValue::Node(EsNode::at(es.node, level))
}

/// The source text of `node` without its leading trivia.
pub fn source_text(file: P<SourceFile>, node: P<Node>) -> &'static str {
    let (pos, end) = trim_node_text_range(file, node);
    source_text_range(file, pos, end)
}

pub fn source_text_range(file: P<SourceFile>, pos: i32, end: i32) -> &'static str {
    let (Ok(pos), Ok(end)) = (usize::try_from(pos), usize::try_from(end)) else {
        return "";
    };
    file.text().get(pos..end).unwrap_or_default()
}

/// A template part's raw text: between the backtick or `}` and the backtick or `${`.
pub fn template_raw(file: P<SourceFile>, node: P<Node>) -> &'static str {
    let text = source_text(file, node);
    let tail_len = match node.kind() {
        Kind::TemplateHead | Kind::TemplateMiddle => 2,
        _ => 1,
    };
    text.get(1..text.len().saturating_sub(tail_len))
        .unwrap_or_default()
}

/// A literal's `value`.
pub fn literal_value(node: P<Node>) -> EsValue {
    match node.kind() {
        Kind::StringLiteral => str(node.text()),
        Kind::NumericLiteral => EsValue::Num(parse_number(node.text())),
        Kind::BigIntLiteral => EsValue::BigInt(node.text().trim_end_matches('n')),
        Kind::RegularExpressionLiteral => EsValue::Regex(node.text()),
        Kind::TrueKeyword => EsValue::Bool(true),
        Kind::FalseKeyword => EsValue::Bool(false),
        _ => EsValue::Null,
    }
}

pub fn parse_number(text: &str) -> f64 {
    let text = text.replace('_', "");
    let radix = match text.get(..2) {
        Some("0x" | "0X") => 16,
        Some("0o" | "0O") => 8,
        Some("0b" | "0B") => 2,
        _ => return text.parse().unwrap_or(f64::NAN),
    };
    let digits = text.get(2..).unwrap_or_default();
    u64::from_str_radix(digits, radix).map_or(f64::NAN, |n| n as f64)
}

fn variable_kind(list: P<Node>) -> &'static str {
    let flags = list.flags() & NodeFlags::BlockScoped;
    if flags == NodeFlags::AwaitUsing {
        "await using"
    } else if flags == NodeFlags::Using {
        "using"
    } else if flags == NodeFlags::Const {
        "const"
    } else if flags == NodeFlags::Let {
        "let"
    } else {
        "var"
    }
}

fn is_async(node: P<Node>) -> bool {
    has_modifier(node, ModifierFlags::Async)
}

fn is_generator(node: P<Node>) -> bool {
    match node.kind() {
        Kind::FunctionDeclaration => node.as_function_declaration().asterisk_token().is_some(),
        Kind::FunctionExpression => node.as_function_expression().asterisk_token().is_some(),
        Kind::MethodDeclaration => node.as_method_declaration().asterisk_token().is_some(),
        _ => false,
    }
}

fn function_field(es: EsNode, name: &str) -> EsValue {
    let node = es.node;
    let is_method = !matches!(
        node.kind(),
        Kind::FunctionDeclaration | Kind::FunctionExpression | Kind::ArrowFunction
    );
    match name {
        "id" if is_method || node.kind() == Kind::ArrowFunction => EsValue::Null,
        "id" => child(node.name()),
        "params" => list(node.parameters()),
        "body" => child(node.body()),
        "async" => EsValue::Bool(is_async(node)),
        "generator" => EsValue::Bool(is_generator(node)),
        "expression" => EsValue::Bool(
            node.kind() == Kind::ArrowFunction
                && node.body().is_some_and(|b| b.kind() != Kind::Block),
        ),
        _ => EsValue::Undefined,
    }
}

fn is_computed(name: Option<P<Node>>) -> bool {
    name.is_some_and(|n| n.kind() == Kind::ComputedPropertyName)
}

fn property_field(es: EsNode, name: &str) -> EsValue {
    let node = es.node;
    match node.kind() {
        Kind::PropertyAssignment => match name {
            "key" => child(node.name()),
            "value" => child(Some(node.as_property_assignment().initializer())),
            "kind" => str("init"),
            "method" | "shorthand" => EsValue::Bool(false),
            "computed" => EsValue::Bool(is_computed(node.name())),
            _ => EsValue::Undefined,
        },
        Kind::ShorthandPropertyAssignment => match name {
            "key" => child(node.name()),
            "value" => {
                if stack(node).len() > 1 {
                    same_level(es, 1)
                } else {
                    child(node.name())
                }
            }
            "kind" => str("init"),
            "method" | "computed" => EsValue::Bool(false),
            "shorthand" => EsValue::Bool(true),
            _ => EsValue::Undefined,
        },
        Kind::BindingElement => {
            let element = node.as_binding_element();
            match name {
                "key" => child(element.property_name().or(element.name())),
                "value" => {
                    if stack(node).len() > 1 {
                        same_level(es, 1)
                    } else {
                        child(element.name())
                    }
                }
                "kind" => str("init"),
                "method" => EsValue::Bool(false),
                "shorthand" => EsValue::Bool(element.property_name().is_none()),
                "computed" => EsValue::Bool(is_computed(element.property_name())),
                _ => EsValue::Undefined,
            }
        }
        _ => match name {
            "key" => child(node.name()),
            "value" => same_level(es, 1),
            "kind" => str(match node.kind() {
                Kind::GetAccessor => "get",
                Kind::SetAccessor => "set",
                _ => "init",
            }),
            "method" => EsValue::Bool(node.kind() == Kind::MethodDeclaration),
            "shorthand" => EsValue::Bool(false),
            "computed" => EsValue::Bool(is_computed(node.name())),
            _ => EsValue::Undefined,
        },
    }
}

fn assignment_pattern_field(es: EsNode, name: &str) -> EsValue {
    let node = es.node;
    let (left, right) = match node.kind() {
        Kind::BinaryExpression => {
            let binary = node.as_binary_expression();
            (Some(binary.left), Some(binary.right()))
        }
        Kind::ShorthandPropertyAssignment => (
            node.name(),
            node.as_shorthand_property_assignment()
                .object_assignment_initializer(),
        ),
        Kind::BindingElement => (node.name(), node.as_binding_element().initializer()),
        Kind::Parameter => (node.name(), node.as_parameter_declaration().initializer()),
        _ => (None, None),
    };
    match name {
        "left" => child(left),
        "right" => child(right),
        _ => EsValue::Undefined,
    }
}

fn rest_argument(node: P<Node>) -> Option<P<Node>> {
    match node.kind() {
        Kind::SpreadElement => Some(node.as_spread_element().expression),
        Kind::SpreadAssignment => Some(node.as_spread_assignment().expression),
        Kind::BindingElement | Kind::Parameter => node.name(),
        _ => None,
    }
}

fn sequence_expressions(node: P<Node>) -> Vec<EsValue> {
    let mut expressions = Vec::new();
    let mut current = node;
    let mut rights = Vec::new();
    while is_comma(current) {
        let binary = current.as_binary_expression();
        rights.push(binary.right());
        current = binary.left;
    }
    expressions.push(child(Some(current)));
    for &right in rights.iter().rev() {
        expressions.push(child(Some(right)));
    }
    expressions
}

fn operator_text(node: P<Node>) -> &'static str {
    match node.kind() {
        Kind::BinaryExpression => {
            tsrs_scanner::token_to_string(node.as_binary_expression().operator_token.kind())
        }
        Kind::PrefixUnaryExpression => {
            tsrs_scanner::token_to_string(node.as_prefix_unary_expression().operator())
        }
        Kind::PostfixUnaryExpression => {
            tsrs_scanner::token_to_string(node.as_postfix_unary_expression().operator())
        }
        Kind::DeleteExpression => "delete",
        Kind::TypeOfExpression => "typeof",
        Kind::VoidExpression => "void",
        _ => "",
    }
}

fn unary_argument(node: P<Node>) -> Option<P<Node>> {
    match node.kind() {
        Kind::PrefixUnaryExpression => Some(node.as_prefix_unary_expression().operand()),
        Kind::PostfixUnaryExpression => Some(node.as_postfix_unary_expression().operand()),
        Kind::DeleteExpression => Some(node.as_delete_expression().expression),
        Kind::TypeOfExpression => Some(node.as_type_of_expression().expression),
        Kind::VoidExpression => Some(node.as_void_expression().expression),
        _ => None,
    }
}

fn class_super(node: P<Node>) -> Option<P<Node>> {
    let clauses = match node.kind() {
        Kind::ClassDeclaration => node.as_class_declaration().heritage_clauses(),
        Kind::ClassExpression => node.as_class_expression().heritage_clauses(),
        _ => None,
    }?;
    let extends = clauses
        .nodes()
        .iter()
        .find(|c| c.as_heritage_clause().token() == Kind::ExtendsKeyword)?;
    extends
        .as_heritage_clause()
        .types()
        .nodes()
        .first()
        .copied()
}

fn import_specifiers(node: P<Node>) -> EsValue {
    let mut specifiers = Vec::new();
    if let Some(clause) = node.as_import_declaration().import_clause {
        let clause = clause.as_import_clause();
        if let Some(name) = clause.name() {
            specifiers.push(child(Some(name)));
        }
        if let Some(bindings) = clause.named_bindings() {
            if bindings.kind() == Kind::NamedImports {
                for &element in bindings.elements() {
                    specifiers.push(child(Some(element)));
                }
            } else {
                specifiers.push(child(Some(bindings)));
            }
        }
    }
    EsValue::List(specifiers)
}

fn import_kind(is_type: bool) -> EsValue {
    str(if is_type { "type" } else { "value" })
}

fn export_specifiers(node: P<Node>) -> EsValue {
    match node.as_export_declaration().export_clause() {
        Some(clause) if clause.kind() == Kind::NamedExports => list(clause.elements()),
        _ => EsValue::List(Vec::new()),
    }
}

fn jsx_attributes(node: P<Node>) -> EsValue {
    node.attributes()
        .map_or(EsValue::List(Vec::new()), |attributes| {
            list(attributes.properties())
        })
}

fn jsx_children(node: P<Node>) -> EsValue {
    list(node.children().nodes())
}

/// `es`'s field `name`, `Undefined` when it has none.
pub fn field(es: EsNode, name: &str, file: P<SourceFile>) -> EsValue {
    use EsType as T;
    let t = es.es_type();
    if name == "type" {
        return if t == T::Other {
            EsValue::Undefined
        } else {
            str(t.name())
        };
    }
    if name == "parent" {
        return parent(es).map_or(EsValue::Null, EsValue::Node);
    }
    let node = es.node;
    let inner = || same_level(es, usize::from(es.level) + 1);
    match t {
        T::Program => match name {
            "body" => list_skipping_absent(node.statements()),
            _ => EsValue::Undefined,
        },
        T::Identifier | T::JSXIdentifier => match name {
            "name" if node.kind() == Kind::ThisKeyword => str("this"),
            "name" => str(node.text()),
            _ => EsValue::Undefined,
        },
        T::PrivateIdentifier => match name {
            "name" => str(node.text().trim_start_matches('#')),
            _ => EsValue::Undefined,
        },
        T::Literal => match name {
            "value" => literal_value(node),
            "raw" => str(source_text(file, node)),
            "regex" if node.kind() == Kind::RegularExpressionLiteral => EsValue::RegexInfo(node),
            "bigint" if node.kind() == Kind::BigIntLiteral => {
                str(node.text().trim_end_matches('n'))
            }
            _ => EsValue::Undefined,
        },
        T::TemplateLiteral => match (node.kind(), name) {
            (Kind::NoSubstitutionTemplateLiteral, "quasis") => {
                EsValue::List(vec![same_level(es, 1)])
            }
            (Kind::NoSubstitutionTemplateLiteral, "expressions") => EsValue::List(Vec::new()),
            (_, "quasis") => {
                let template = node.as_template_expression();
                let mut quasis = vec![child(Some(template.head()))];
                for span in template.template_spans().nodes() {
                    quasis.push(child(Some(span.as_template_span().literal)));
                }
                EsValue::List(quasis)
            }
            (_, "expressions") => EsValue::List(
                node.as_template_expression()
                    .template_spans()
                    .nodes()
                    .iter()
                    .map(|span| child(Some(span.as_template_span().expression)))
                    .collect(),
            ),
            _ => EsValue::Undefined,
        },
        T::TemplateElement => match name {
            "value" => EsValue::TemplateValue(node),
            "tail" => EsValue::Bool(matches!(
                node.kind(),
                Kind::TemplateTail | Kind::NoSubstitutionTemplateLiteral
            )),
            _ => EsValue::Undefined,
        },
        T::TaggedTemplateExpression => {
            let tagged = node.as_tagged_template_expression();
            match name {
                "tag" => child(Some(tagged.tag())),
                "quasi" => child(Some(tagged.template())),
                _ => EsValue::Undefined,
            }
        }
        T::ArrayExpression | T::ArrayPattern => match name {
            "elements" => list(node.elements()),
            _ => EsValue::Undefined,
        },
        T::ObjectExpression | T::ObjectPattern => match name {
            "properties" if node.kind() == Kind::ObjectBindingPattern => list(node.elements()),
            "properties" => list(node.properties()),
            _ => EsValue::Undefined,
        },
        T::Property => property_field(es, name),
        T::FunctionDeclaration
        | T::FunctionExpression
        | T::ArrowFunctionExpression
        | T::TSDeclareFunction => function_field(es, name),
        T::ClassDeclaration | T::ClassExpression => match name {
            "id" => child(node.name()),
            "superClass" => child(class_super(node)),
            "body" => inner(),
            "abstract" => EsValue::Bool(has_modifier(node, ModifierFlags::Abstract)),
            "decorators" => list(&node.decorators()),
            _ => EsValue::Undefined,
        },
        T::ClassBody => match name {
            "body" => list_skipping_absent(node.members()),
            _ => EsValue::Undefined,
        },
        T::MethodDefinition => match name {
            "key" => child(node.name()),
            "value" => inner(),
            "kind" => str(match node.kind() {
                Kind::Constructor => "constructor",
                Kind::GetAccessor => "get",
                Kind::SetAccessor => "set",
                _ => "method",
            }),
            "static" => EsValue::Bool(has_modifier(node, ModifierFlags::Static)),
            "computed" => EsValue::Bool(is_computed(node.name())),
            "decorators" => list(&node.decorators()),
            _ => EsValue::Undefined,
        },
        T::PropertyDefinition => match name {
            "key" => child(node.name()),
            "value" => child(node.initializer()),
            "static" => EsValue::Bool(has_modifier(node, ModifierFlags::Static)),
            "computed" => EsValue::Bool(is_computed(node.name())),
            "readonly" => EsValue::Bool(has_modifier(node, ModifierFlags::Readonly)),
            "decorators" => list(&node.decorators()),
            _ => EsValue::Undefined,
        },
        T::StaticBlock => match name {
            "body" => {
                list_skipping_absent(node.as_class_static_block_declaration().body.statements())
            }
            _ => EsValue::Undefined,
        },
        T::VariableDeclaration => match name {
            "declarations" => list(node.as_variable_declaration_list().declarations().nodes()),
            "kind" => str(variable_kind(node)),
            "declare" => EsValue::Bool(node.parent().is_some_and(|p| {
                p.kind() == Kind::VariableStatement && has_modifier(p, ModifierFlags::Ambient)
            })),
            _ => EsValue::Undefined,
        },
        T::VariableDeclarator => match name {
            "id" => child(node.name()),
            "init" => child(node.initializer()),
            "definite" => {
                EsValue::Bool(node.as_variable_declaration().exclamation_token().is_some())
            }
            _ => EsValue::Undefined,
        },
        T::ExpressionStatement => match name {
            "expression" => child(Some(node.as_expression_statement().expression)),
            _ => EsValue::Undefined,
        },
        T::BlockStatement => match name {
            "body" => list_skipping_absent(node.statements()),
            _ => EsValue::Undefined,
        },
        T::IfStatement => {
            let statement = node.as_if_statement();
            match name {
                "test" => child(Some(statement.expression())),
                "consequent" => child(Some(statement.then_statement())),
                "alternate" => child(statement.else_statement()),
                _ => EsValue::Undefined,
            }
        }
        T::SwitchStatement => {
            let statement = node.as_switch_statement();
            match name {
                "discriminant" => child(Some(statement.expression())),
                "cases" => list(statement.case_block().as_case_block().clauses().nodes()),
                _ => EsValue::Undefined,
            }
        }
        T::SwitchCase => {
            let clause = node.as_case_or_default_clause();
            match name {
                "test" => child(clause.expression()),
                "consequent" => list_skipping_absent(clause.statements().nodes()),
                _ => EsValue::Undefined,
            }
        }
        T::WhileStatement | T::DoWhileStatement => match name {
            "test" if node.kind() == Kind::DoStatement => {
                child(Some(node.as_do_statement().expression()))
            }
            "test" => child(Some(node.as_while_statement().expression())),
            "body" => child(Some(node.statement())),
            _ => EsValue::Undefined,
        },
        T::ForStatement => {
            let statement = node.as_for_statement();
            match name {
                "init" => child(statement.initializer()),
                "test" => child(statement.condition()),
                "update" => child(statement.incrementor()),
                "body" => child(Some(statement.statement())),
                _ => EsValue::Undefined,
            }
        }
        T::ForInStatement | T::ForOfStatement => {
            let statement = node.as_for_in_or_of_statement();
            match name {
                "left" => child(Some(statement.initializer)),
                "right" => child(Some(statement.expression)),
                "body" => child(Some(statement.statement)),
                "await" if t == T::ForOfStatement => {
                    EsValue::Bool(statement.await_modifier().is_some())
                }
                _ => EsValue::Undefined,
            }
        }
        T::ReturnStatement => match name {
            "argument" => child(node.as_return_statement().expression()),
            _ => EsValue::Undefined,
        },
        T::ThrowStatement => match name {
            "argument" => child(Some(node.as_throw_statement().expression())),
            _ => EsValue::Undefined,
        },
        T::BreakStatement | T::ContinueStatement => match name {
            "label" => child(node.label()),
            _ => EsValue::Undefined,
        },
        T::LabeledStatement => match name {
            "label" => child(node.label()),
            "body" => child(Some(node.statement())),
            _ => EsValue::Undefined,
        },
        T::WithStatement => match name {
            "object" => child(Some(node.as_with_statement().expression())),
            "body" => child(Some(node.statement())),
            _ => EsValue::Undefined,
        },
        T::TryStatement => {
            let statement = node.as_try_statement();
            match name {
                "block" => child(Some(statement.try_block())),
                "handler" => child(statement.catch_clause()),
                "finalizer" => child(statement.finally_block()),
                _ => EsValue::Undefined,
            }
        }
        T::CatchClause => {
            let clause = node.as_catch_clause();
            match name {
                "param" => child(clause.variable_declaration().and_then(|d| d.name())),
                "body" => child(Some(clause.block)),
                _ => EsValue::Undefined,
            }
        }
        T::UnaryExpression | T::UpdateExpression => match name {
            "operator" => str(operator_text(node)),
            "argument" => child(unary_argument(node)),
            "prefix" => EsValue::Bool(node.kind() != Kind::PostfixUnaryExpression),
            _ => EsValue::Undefined,
        },
        T::BinaryExpression | T::LogicalExpression | T::AssignmentExpression => {
            let binary = node.as_binary_expression();
            match name {
                "operator" => str(operator_text(node)),
                "left" => child(Some(binary.left)),
                "right" => child(Some(binary.right())),
                _ => EsValue::Undefined,
            }
        }
        T::AssignmentPattern => assignment_pattern_field(es, name),
        T::RestElement | T::SpreadElement => match name {
            "argument" => child(rest_argument(node)),
            _ => EsValue::Undefined,
        },
        T::SequenceExpression => match name {
            "expressions" => EsValue::List(sequence_expressions(node)),
            _ => EsValue::Undefined,
        },
        T::ConditionalExpression => {
            let conditional = node.as_conditional_expression();
            match name {
                "test" => child(Some(conditional.condition())),
                "consequent" => child(Some(conditional.when_true())),
                "alternate" => child(Some(conditional.when_false())),
                _ => EsValue::Undefined,
            }
        }
        T::CallExpression | T::NewExpression => match name {
            "callee" if node.kind() == Kind::NewExpression => {
                child(Some(node.as_new_expression().expression()))
            }
            "callee" => child(Some(node.as_call_expression().expression())),
            "arguments" => list(node.arguments()),
            "optional" if t == T::CallExpression => {
                EsValue::Bool(node.question_dot_token().is_some())
            }
            _ => EsValue::Undefined,
        },
        T::ImportExpression => match name {
            "source" => child(node.arguments().first().copied()),
            "options" => child(node.arguments().get(1).copied()),
            _ => EsValue::Undefined,
        },
        T::MemberExpression => {
            let computed = node.kind() == Kind::ElementAccessExpression;
            match name {
                "object" => child(node.expression()),
                "property" if computed => child(Some(
                    node.as_element_access_expression().argument_expression(),
                )),
                "property" => child(node.name()),
                "computed" => EsValue::Bool(computed),
                "optional" => EsValue::Bool(node.question_dot_token().is_some()),
                _ => EsValue::Undefined,
            }
        }
        T::ChainExpression => match name {
            "expression" => inner(),
            _ => EsValue::Undefined,
        },
        T::AwaitExpression => match name {
            "argument" => child(Some(node.as_await_expression().expression)),
            _ => EsValue::Undefined,
        },
        T::YieldExpression => {
            let expression = node.as_yield_expression();
            match name {
                "argument" => child(expression.expression()),
                "delegate" => EsValue::Bool(expression.asterisk_token().is_some()),
                _ => EsValue::Undefined,
            }
        }
        T::MetaProperty => match name {
            "property" => child(node.name()),
            _ => EsValue::Undefined,
        },
        T::Decorator => match name {
            "expression" => child(Some(node.as_decorator().expression())),
            _ => EsValue::Undefined,
        },
        T::ImportDeclaration => match name {
            "specifiers" => import_specifiers(node),
            "source" => child(node.module_specifier()),
            "importKind" => import_kind(
                node.as_import_declaration()
                    .import_clause
                    .is_some_and(|clause| clause.is_type_only()),
            ),
            _ => EsValue::Undefined,
        },
        T::ImportSpecifier => match name {
            "imported" => child(node.property_name().or(node.name())),
            "local" => child(node.name()),
            "importKind" => import_kind(node.is_type_only()),
            _ => EsValue::Undefined,
        },
        T::ImportDefaultSpecifier => match name {
            "local" => inner(),
            _ => EsValue::Undefined,
        },
        T::ImportNamespaceSpecifier => match name {
            "local" => child(node.name()),
            _ => EsValue::Undefined,
        },
        T::ExportNamedDeclaration => match (node.kind(), name) {
            (Kind::ExportDeclaration, "declaration") => EsValue::Null,
            (Kind::ExportDeclaration, "specifiers") => export_specifiers(node),
            (Kind::ExportDeclaration, "source") => child(node.module_specifier()),
            (Kind::ExportDeclaration, "exportKind") => import_kind(node.is_type_only()),
            (Kind::VariableStatement, "declaration") => {
                child(Some(node.as_variable_statement().declaration_list))
            }
            (_, "declaration") => inner(),
            (_, "specifiers") => EsValue::List(Vec::new()),
            (_, "source") => EsValue::Null,
            (_, "exportKind") => import_kind(matches!(
                node.kind(),
                Kind::InterfaceDeclaration | Kind::TypeAliasDeclaration
            )),
            _ => EsValue::Undefined,
        },
        T::ExportDefaultDeclaration => match name {
            "declaration" if node.kind() == Kind::ExportAssignment => {
                child(Some(node.as_export_assignment().expression()))
            }
            "declaration" => inner(),
            _ => EsValue::Undefined,
        },
        T::ExportAllDeclaration => match name {
            "exported" => child(
                node.as_export_declaration()
                    .export_clause()
                    .and_then(|clause| clause.name()),
            ),
            "source" => child(node.module_specifier()),
            "exportKind" => import_kind(node.is_type_only()),
            _ => EsValue::Undefined,
        },
        T::ExportSpecifier => match name {
            "local" => child(node.property_name().or(node.name())),
            "exported" => child(node.name()),
            "exportKind" => import_kind(node.is_type_only()),
            _ => EsValue::Undefined,
        },
        T::JSXElement => match (node.kind(), name) {
            (Kind::JsxSelfClosingElement, "openingElement") => same_level(es, 1),
            (Kind::JsxSelfClosingElement, "closingElement") => EsValue::Null,
            (Kind::JsxSelfClosingElement, "children") => EsValue::List(Vec::new()),
            (_, "openingElement") => child(Some(node.as_jsx_element().opening_element())),
            (_, "closingElement") => child(Some(node.as_jsx_element().closing_element())),
            (_, "children") => jsx_children(node),
            _ => EsValue::Undefined,
        },
        T::JSXOpeningElement => match name {
            "name" => child(Some(node.tag_name())),
            "attributes" => jsx_attributes(node),
            "selfClosing" => EsValue::Bool(node.kind() == Kind::JsxSelfClosingElement),
            _ => EsValue::Undefined,
        },
        T::JSXClosingElement => match name {
            "name" => child(Some(node.tag_name())),
            _ => EsValue::Undefined,
        },
        T::JSXFragment => {
            let fragment = node.as_jsx_fragment();
            match name {
                "openingFragment" => child(Some(fragment.opening_fragment())),
                "closingFragment" => child(Some(fragment.closing_fragment())),
                "children" => jsx_children(node),
                _ => EsValue::Undefined,
            }
        }
        T::JSXAttribute => {
            let attribute = node.as_jsx_attribute();
            match name {
                "name" => child(Some(attribute.name())),
                "value" => child(attribute.initializer()),
                _ => EsValue::Undefined,
            }
        }
        T::JSXSpreadAttribute => match name {
            "argument" => child(Some(node.as_jsx_spread_attribute().expression())),
            _ => EsValue::Undefined,
        },
        T::JSXExpressionContainer => match name {
            "expression" => child(node.as_jsx_expression().expression()),
            _ => EsValue::Undefined,
        },
        T::JSXText => match name {
            "value" | "raw" => str(source_text(file, node)),
            _ => EsValue::Undefined,
        },
        T::JSXMemberExpression => match name {
            "object" => child(node.expression()),
            "property" => child(node.name()),
            _ => EsValue::Undefined,
        },
        T::JSXNamespacedName => {
            let namespaced = node.as_jsx_namespaced_name();
            match name {
                "namespace" => child(Some(namespaced.namespace())),
                "name" => child(Some(namespaced.name())),
                _ => EsValue::Undefined,
            }
        }
        T::TSNonNullExpression => match name {
            "expression" => child(Some(node.as_non_null_expression().expression)),
            _ => EsValue::Undefined,
        },
        T::TSAsExpression | T::TSSatisfiesExpression | T::TSTypeAssertion => match name {
            "expression" => child(node.expression()),
            "typeAnnotation" => child(node.type_node()),
            _ => EsValue::Undefined,
        },
        T::TSQualifiedName => {
            let qualified = node.as_qualified_name();
            match name {
                "left" => child(Some(qualified.left)),
                "right" => child(Some(qualified.right)),
                _ => EsValue::Undefined,
            }
        }
        T::TSTypeReference => match name {
            "typeName" => child(Some(node.as_type_reference_node().type_name())),
            _ => EsValue::Undefined,
        },
        T::TSTypeParameter => {
            let parameter = node.as_type_parameter_declaration();
            match name {
                "name" => child(Some(parameter.name())),
                "constraint" => child(parameter.constraint()),
                "default" => child(parameter.default_type()),
                _ => EsValue::Undefined,
            }
        }
        T::TSInterfaceDeclaration => match name {
            "id" => child(node.name()),
            "body" => inner(),
            "declare" => EsValue::Bool(has_modifier(node, ModifierFlags::Ambient)),
            _ => EsValue::Undefined,
        },
        T::TSInterfaceBody => match name {
            "body" => list(node.members()),
            _ => EsValue::Undefined,
        },
        T::TSTypeAliasDeclaration => match name {
            "id" => child(node.name()),
            "typeAnnotation" => child(node.type_node()),
            "declare" => EsValue::Bool(has_modifier(node, ModifierFlags::Ambient)),
            _ => EsValue::Undefined,
        },
        T::TSEnumDeclaration => match name {
            "id" => child(node.name()),
            "members" => list(node.members()),
            "const" => EsValue::Bool(has_modifier(node, ModifierFlags::Const)),
            "declare" => EsValue::Bool(has_modifier(node, ModifierFlags::Ambient)),
            _ => EsValue::Undefined,
        },
        T::TSEnumMember => match name {
            "id" => child(node.name()),
            "initializer" => child(node.initializer()),
            _ => EsValue::Undefined,
        },
        T::TSParameterProperty => match name {
            "parameter" if !es.is_last_level() => inner(),
            "parameter" => child(node.name()),
            "accessibility" => {
                let flags = node.modifier_flags();
                if flags.contains(ModifierFlags::Public) {
                    str("public")
                } else if flags.contains(ModifierFlags::Private) {
                    str("private")
                } else if flags.contains(ModifierFlags::Protected) {
                    str("protected")
                } else {
                    EsValue::Undefined
                }
            }
            "readonly" => EsValue::Bool(has_modifier(node, ModifierFlags::Readonly)),
            "override" => EsValue::Bool(has_modifier(node, ModifierFlags::Override)),
            "static" => EsValue::Bool(false),
            _ => EsValue::Undefined,
        },
        T::ThisExpression
        | T::Super
        | T::EmptyStatement
        | T::DebuggerStatement
        | T::JSXOpeningFragment
        | T::JSXClosingFragment
        | T::Other => EsValue::Undefined,
    }
}

/// The source range of an ESTree node, like ESLint's `node.range`.
pub fn range(es: EsNode, file: P<SourceFile>) -> (i32, i32) {
    let node = es.node;
    let (pos, end) = trim_node_text_range(file, node);
    if es.level == 0 {
        return (pos, end);
    }
    let start_of = |n: P<Node>| trim_node_text_range(file, n).0;
    let start = match es.es_type() {
        EsType::FunctionDeclaration
        | EsType::TSDeclareFunction
        | EsType::ClassDeclaration
        | EsType::TSInterfaceDeclaration
        | EsType::TSTypeAliasDeclaration
        | EsType::TSEnumDeclaration
        | EsType::Other
            if es.level == 1 && is_export_level(node) =>
        {
            after_export_keywords(file, node).unwrap_or(pos)
        }
        EsType::ClassBody | EsType::TSInterfaceBody => {
            members_open_brace(file, node).unwrap_or(pos)
        }
        EsType::AssignmentPattern | EsType::RestElement
            if node.kind() != Kind::ShorthandPropertyAssignment =>
        {
            match node.kind() {
                Kind::Parameter => node
                    .as_parameter_declaration()
                    .dot_dot_dot_token()
                    .or(node.name())
                    .map_or(pos, start_of),
                _ => node.name().map_or(pos, start_of),
            }
        }
        EsType::FunctionExpression => node
            .name()
            .map_or(pos, |name| after_method_name(file, name)),
        _ => pos,
    };
    (start, end)
}

fn is_export_level(node: P<Node>) -> bool {
    ast::has_syntactic_modifier(node, ModifierFlags::Export)
}

fn after_export_keywords(file: P<SourceFile>, node: P<Node>) -> Option<i32> {
    let last = node
        .modifier_nodes()
        .iter()
        .rev()
        .find(|m| matches!(m.kind(), Kind::ExportKeyword | Kind::DefaultKeyword))?;
    let end = last.end().source_offset()?;
    i32::try_from(tsrs_scanner::skip_trivia(file.text(), end)).ok()
}

fn members_open_brace(file: P<SourceFile>, node: P<Node>) -> Option<i32> {
    let list = node.member_list()?;
    let after_brace = list.pos().source_offset()?;
    let brace = after_brace.checked_sub(1)?;
    let text = file.text();
    (text.as_bytes().get(usize::try_from(brace).ok()?) == Some(&b'{'))
        .then(|| i32::try_from(brace).ok())
        .flatten()
}

fn after_method_name(file: P<SourceFile>, name: P<Node>) -> i32 {
    let text = file.text();
    let mut pos = name.end().source_offset().unwrap_or(0);
    pos = tsrs_scanner::skip_trivia(text, pos);
    if matches!(text.as_bytes().get(pos as usize), Some(b'?' | b'!')) {
        pos = tsrs_scanner::skip_trivia(text, pos + 1);
    }
    i32::try_from(pos).unwrap_or(0)
}
