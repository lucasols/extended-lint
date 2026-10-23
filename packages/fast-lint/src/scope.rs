//! ESLint scope analysis on top of the tsrs binder and checker, for rules ported from ESLint:
//!
//! - [`declared_names`] is `sourceCode.getDeclaredVariables(node)`: the binding identifiers a declaration declares;
//! - [`binding_symbol`] is the variable a binding identifier declares;
//! - [`resolve_reference`] is `reference.resolved`: the variable an identifier refers to;
//! - [`ReferenceIndex::references`] is `variable.references`: every reference to a variable in the file, in source
//!   order, with eslint-scope's read/write flags and its `init` write references for initialized bindings.
//!
//! Variables are identified by their canonical symbol: merged, and for an exported local its export symbol, so a
//! declaration and the identifiers that use it map to the same [`P<Symbol>`]. Like typescript-eslint's scope manager,
//! type references (`x: Foo`, `typeof x`) and JSX opening tag names that aren't intrinsic (lowercase) are references
//! too; closing tag names are not.

use rustc_hash::FxHashMap;
use tsrs_ast::{self as ast, Kind, Node, SourceFile, Symbol, SymbolFlags};
use tsrs_checker::Checker;
use tsrs_core::P;

/// eslint-scope's `Reference.isRead()`/`isWrite()` flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy)]
pub struct Reference {
    /// The identifier that refers to the variable. For an `init` reference, the binding identifier itself.
    pub identifier: P<Node>,
    pub access: Access,
    /// eslint-scope's `reference.init`: the write of a binding's initializer (`const x = 1`, `function f(x = 1)`,
    /// `for (const x of xs)`), whose identifier is the declaration's own name.
    pub init: bool,
}

impl Reference {
    pub fn is_read(&self) -> bool {
        self.access != Access::Write
    }

    pub fn is_write(&self) -> bool {
        self.access != Access::Read
    }

    pub fn is_read_only(&self) -> bool {
        self.access == Access::Read
    }

    pub fn is_write_only(&self) -> bool {
        self.access == Access::Write
    }
}

/// The identifiers of one file grouped by name, built once per file (a single walk) so finding the references to
/// a variable only resolves the identifiers that share its name.
pub struct ReferenceIndex {
    identifiers: FxHashMap<&'static str, Vec<P<Node>>>,
}

impl ReferenceIndex {
    pub fn new(file: P<SourceFile>) -> ReferenceIndex {
        let mut identifiers: FxHashMap<&'static str, Vec<P<Node>>> = FxHashMap::default();
        let mut stack = vec![file.as_node()];
        while let Some(node) = stack.pop() {
            if node.kind() == Kind::Identifier {
                identifiers.entry(node.text()).or_default().push(node);
                continue;
            }
            let start = stack.len();
            node.for_each_child(&mut |child| {
                stack.push(child);
                false
            });
            stack[start..].reverse();
        }
        ReferenceIndex { identifiers }
    }

    /// Every reference to `symbol` in the file, in source order, like eslint-scope's `variable.references`:
    /// identifiers that resolve to it, plus an `init` write reference for each of its initialized bindings.
    /// Declaration names (other than those `init` writes), property names and keys are not references.
    pub fn references(&self, checker: &mut Checker, symbol: P<Symbol>) -> Vec<Reference> {
        let mut references = Vec::new();
        let mut names: Vec<&'static str> = vec![symbol.name()];
        for &declaration in symbol.declarations() {
            if let Some(name) = declaration
                .name()
                .filter(|name| name.kind() == Kind::Identifier)
            {
                if !names.contains(&name.text()) {
                    names.push(name.text());
                }
            }
        }
        let mut candidates: Vec<P<Node>> = names
            .iter()
            .filter_map(|name| self.identifiers.get(name))
            .flatten()
            .copied()
            .collect();
        if names.len() > 1 {
            candidates.sort_by_key(|node| node.pos());
        }
        for identifier in candidates {
            if is_binding_name(identifier) {
                if is_initialized_binding(identifier)
                    && binding_symbol(checker, identifier) == Some(symbol)
                {
                    references.push(Reference {
                        identifier,
                        access: Access::Write,
                        init: true,
                    });
                }
                continue;
            }
            if resolve_reference(checker, identifier) == Some(symbol) {
                references.push(Reference {
                    identifier,
                    access: access_of(identifier),
                    init: false,
                });
            }
        }
        references
    }

    /// The references to the variable declared by the binding identifier `name` (see [`references`](Self::references)).
    pub fn references_of_binding(&self, checker: &mut Checker, name: P<Node>) -> Vec<Reference> {
        match binding_symbol(checker, name) {
            Some(symbol) => self.references(checker, symbol),
            None => Vec::new(),
        }
    }
}

/// The variable declared by the binding identifier `name` (a variable, parameter, function, class, import, enum, type
/// or catch binding name), as its canonical symbol.
pub fn binding_symbol(checker: &mut Checker, name: P<Node>) -> Option<P<Symbol>> {
    let symbol = checker.get_symbol_at_location_exported(name)?;
    Some(canonical(checker, symbol))
}

/// eslint-scope's `reference.resolved`: the variable the identifier `identifier` refers to, as its canonical symbol.
/// `None` for declaration names, property names and keys, labels and unresolved (global or missing) names.
pub fn resolve_reference(checker: &mut Checker, identifier: P<Node>) -> Option<P<Symbol>> {
    let parent = identifier.parent()?;
    let symbol = match parent.kind() {
        Kind::ShorthandPropertyAssignment if parent.name() == Some(identifier) => checker
            .resolve_name_exported(
                identifier.text(),
                identifier,
                SymbolFlags::Value | SymbolFlags::Alias,
                false,
            )?,
        Kind::ExportSpecifier => {
            let is_reexport = parent
                .parent()
                .and_then(|named_exports| named_exports.parent())
                .is_some_and(|declaration| declaration.module_specifier().is_some());
            if is_reexport || parent.property_name_or_name() != Some(identifier) {
                return None;
            }
            checker.resolve_name_exported(
                identifier.text(),
                identifier,
                SymbolFlags::Value
                    | SymbolFlags::Type
                    | SymbolFlags::Namespace
                    | SymbolFlags::Alias,
                false,
            )?
        }
        Kind::ImportSpecifier | Kind::JsxClosingElement => return None,
        Kind::JsxOpeningElement | Kind::JsxSelfClosingElement
            if identifier
                .text()
                .starts_with(|c: char| c.is_ascii_lowercase()) =>
        {
            return None;
        }
        _ if ast::is_declaration_name(identifier) => return None,
        _ => checker.get_symbol_at_location_exported(identifier)?,
    };
    if symbol == checker.unknown_symbol {
        return None;
    }
    Some(canonical(checker, symbol))
}

/// eslint-scope's `getDeclaredVariables(node)`: the binding identifiers `node` declares. A variable statement,
/// declaration list or declarator declares the names of its binding patterns; a function declaration or expression
/// its name and its parameters; an arrow function, method, accessor or constructor its parameters; a
/// class, enum, interface, type alias or module its name; an import declaration or specifier its local names; a
/// catch clause its variable.
pub fn declared_names(node: P<Node>) -> Vec<P<Node>> {
    let mut names = Vec::new();
    match node.kind() {
        Kind::VariableStatement => {
            push_declaration_list_names(
                node.as_variable_statement().declaration_list(),
                &mut names,
            );
        }
        Kind::VariableDeclarationList => push_declaration_list_names(node, &mut names),
        Kind::VariableDeclaration | Kind::Parameter | Kind::BindingElement => {
            if let Some(name) = node.name() {
                push_binding_names(name, &mut names);
            }
        }
        Kind::FunctionDeclaration
        | Kind::FunctionExpression
        | Kind::ArrowFunction
        | Kind::MethodDeclaration
        | Kind::Constructor
        | Kind::GetAccessor
        | Kind::SetAccessor => {
            if matches!(
                node.kind(),
                Kind::FunctionDeclaration | Kind::FunctionExpression
            ) {
                if let Some(name) = node.name() {
                    names.push(name);
                }
            }
            for &parameter in node.parameters() {
                if let Some(name) = parameter.name() {
                    push_binding_names(name, &mut names);
                }
            }
        }
        Kind::ClassDeclaration
        | Kind::ClassExpression
        | Kind::EnumDeclaration
        | Kind::InterfaceDeclaration
        | Kind::TypeAliasDeclaration
        | Kind::ModuleDeclaration => {
            if let Some(name) = node.name().filter(|name| name.kind() == Kind::Identifier) {
                names.push(name);
            }
        }
        Kind::ImportDeclaration => {
            if let Some(clause) = node.as_import_declaration().import_clause() {
                push_import_clause_names(clause, &mut names);
            }
        }
        Kind::ImportClause => push_import_clause_names(node, &mut names),
        Kind::ImportSpecifier | Kind::NamespaceImport | Kind::ImportEqualsDeclaration => {
            if let Some(name) = node.name() {
                names.push(name);
            }
        }
        Kind::CatchClause => {
            if let Some(name) = node
                .as_catch_clause()
                .variable_declaration()
                .and_then(|declaration| declaration.name())
            {
                push_binding_names(name, &mut names);
            }
        }
        _ => {}
    }
    names
}

/// The identifiers bound by a binding name: the identifier itself, or every identifier of a destructuring pattern.
pub fn push_binding_names(name: P<Node>, names: &mut Vec<P<Node>>) {
    match name.kind() {
        Kind::Identifier => names.push(name),
        Kind::ObjectBindingPattern | Kind::ArrayBindingPattern => {
            for &element in name.elements() {
                if let Some(element_name) = element.name() {
                    push_binding_names(element_name, names);
                }
            }
        }
        _ => {}
    }
}

fn push_declaration_list_names(list: P<Node>, names: &mut Vec<P<Node>>) {
    for &declaration in list.as_variable_declaration_list().declarations().nodes() {
        if let Some(name) = declaration.name() {
            push_binding_names(name, names);
        }
    }
}

fn push_import_clause_names(clause: P<Node>, names: &mut Vec<P<Node>>) {
    if let Some(name) = clause.name() {
        names.push(name);
    }
    let Some(bindings) = clause.as_import_clause().named_bindings() else {
        return;
    };
    if bindings.kind() == Kind::NamespaceImport {
        if let Some(name) = bindings.name() {
            names.push(name);
        }
        return;
    }
    for &specifier in bindings.elements() {
        if let Some(name) = specifier.name() {
            names.push(name);
        }
    }
}

fn canonical(checker: &mut Checker, symbol: P<Symbol>) -> P<Symbol> {
    checker.get_export_symbol_of_symbol(symbol)
}

/// Whether `identifier` is the name of a variable, parameter or binding-pattern element (where eslint-scope may
/// create an `init` write reference), not a use.
fn is_binding_name(identifier: P<Node>) -> bool {
    identifier.parent().is_some_and(|parent| {
        matches!(
            parent.kind(),
            Kind::VariableDeclaration | Kind::Parameter | Kind::BindingElement
        ) && parent.name() == Some(identifier)
    })
}

/// Whether eslint-scope writes the binding `name` on declaration: its variable declaration has an initializer or is
/// the left side of a `for...in`/`for...of`, or it is a parameter (or a parameter's destructured element) with a
/// default value.
fn is_initialized_binding(name: P<Node>) -> bool {
    let mut has_default = false;
    let mut node = name.parent();
    while let Some(current) = node {
        match current.kind() {
            Kind::BindingElement => {
                has_default |= current.initializer().is_some();
                node = current.parent().and_then(|pattern| pattern.parent());
            }
            Kind::VariableDeclaration => {
                if current.initializer().is_some() {
                    return true;
                }
                return current
                    .parent()
                    .and_then(|list| list.parent())
                    .is_some_and(|statement| {
                        matches!(
                            statement.kind(),
                            Kind::ForInStatement | Kind::ForOfStatement
                        )
                    });
            }
            Kind::Parameter => return has_default || current.initializer().is_some(),
            _ => return false,
        }
    }
    false
}

/// eslint-scope's read/write flags of a reference: an assignment target is written (`x = 1`, `[x] = a`,
/// `for (x of xs)`), a compound assignment or update both reads and writes (`x += 1`, `x++`), anything else reads.
fn access_of(identifier: P<Node>) -> Access {
    if ast::is_write_only_access(identifier) {
        Access::Write
    } else if ast::is_write_access(identifier) {
        Access::ReadWrite
    } else {
        Access::Read
    }
}
