mod common;

use common::{lint_rule, Case};
use fast_lint::scope::{declared_names, Access, ReferenceIndex};
use indoc::indoc;
use insta::assert_snapshot;
use std::fmt::Write as _;
use tsrs_ast::{Kind, Node};
use tsrs_core::P;
use tsrslint::rule::{Ctx, Listener, Rule, RuleMessage, RuleVisitor};

/// Reports every reference to each variable whose name starts with `target`/`Target`, at the reference, with its
/// access flags.
struct ReferencesRule;

struct NoVisitor;

impl RuleVisitor for NoVisitor {
    fn listeners(&self) -> &[Listener] {
        &[]
    }
    fn visit(&mut self, _ctx: &mut Ctx, _l: Listener, _node: P<Node>) {}
}

fn is_target_name(name: &str) -> bool {
    name.starts_with("target") || name.starts_with("Target")
}

fn all_nodes(root: P<Node>) -> Vec<P<Node>> {
    let mut nodes = Vec::new();
    let mut stack = vec![root];
    while let Some(node) = stack.pop() {
        nodes.push(node);
        node.for_each_child(&mut |child| {
            stack.push(child);
            false
        });
    }
    nodes
}

impl Rule for ReferencesRule {
    fn name(&self) -> &'static str {
        "test-references"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        let index = ReferenceIndex::new(ctx.file);
        let mut bindings: Vec<P<Node>> = all_nodes(ctx.file.as_node())
            .into_iter()
            .flat_map(declared_names)
            .filter(|name| is_target_name(name.text()))
            .collect();
        bindings.sort_by_key(|node| node.pos());
        bindings.dedup();
        for binding in bindings {
            for reference in index.references_of_binding(ctx.checker, binding) {
                let access = match reference.access {
                    Access::Read => "read",
                    Access::Write => "write",
                    Access::ReadWrite => "readWrite",
                };
                let init = if reference.init { " init" } else { "" };
                ctx.report_node(
                    reference.identifier,
                    RuleMessage::new("reference", format!("{} {access}{init}", binding.text())),
                );
            }
        }
        Box::new(NoVisitor)
    }
}

/// Reports the names each declaration declares, like ESLint's `getDeclaredVariables`.
struct DeclaredRule;

const DECLARATION_KINDS: &[Kind] = &[
    Kind::VariableStatement,
    Kind::FunctionDeclaration,
    Kind::FunctionExpression,
    Kind::ArrowFunction,
    Kind::ClassDeclaration,
    Kind::ImportDeclaration,
    Kind::CatchClause,
];

impl Rule for DeclaredRule {
    fn name(&self) -> &'static str {
        "test-declared"
    }
    fn create_visitor(&'static self, ctx: &mut Ctx) -> Box<dyn RuleVisitor> {
        for node in all_nodes(ctx.file.as_node()) {
            if !DECLARATION_KINDS.contains(&node.kind()) {
                continue;
            }
            let names: Vec<&str> = declared_names(node)
                .iter()
                .map(|name| name.text())
                .collect();
            ctx.report_node(
                node,
                RuleMessage::new(
                    "declared",
                    format!("{:?}: {}", node.kind(), names.join(", ")),
                ),
            );
        }
        Box::new(NoVisitor)
    }
}

fn references(code: &str) -> String {
    let result = lint_rule(&ReferencesRule, &Case::from(code));
    let mut diagnostics = result.diagnostics.clone();
    diagnostics.sort_by_key(|d| (d.line, d.column));
    let mut out = String::new();
    for d in diagnostics {
        let _ = writeln!(out, "{}:{} {}", d.line, d.column, d.message);
    }
    out
}

fn declared(code: &str) -> String {
    let result = lint_rule(&DeclaredRule, &Case::from(code));
    let mut diagnostics = result.diagnostics.clone();
    diagnostics.sort_by_key(|d| (d.line, d.column));
    let mut out = String::new();
    for d in diagnostics {
        let _ = writeln!(out, "{}: {}", d.line, d.message);
    }
    out
}

#[test]
fn reads_writes_and_init() {
    assert_snapshot!(references(indoc! {"
        let target = 1
        target
        target = 2
        target += 1
        target++
        const o = { target }
        ;({ target } = o)
        ;[target] = [1]
        for (target of [1]) {}
        export { target }
    "}), @r"
    1:5 target write init
    2:1 target read
    3:1 target write
    4:1 target readWrite
    5:1 target readWrite
    6:13 target read
    7:5 target write
    8:3 target write
    9:6 target write
    10:10 target read
    ");
}

#[test]
fn shadowed_names_are_other_variables() {
    assert_snapshot!(references(indoc! {"
        const target = 1
        function f(target: number) {
          return target
        }
        function g() {
          const target = 2
          return target
        }
        target
    "}), @r"
    1:7 target write init
    3:10 target read
    6:9 target write init
    7:10 target read
    9:1 target read
    ");
}

#[test]
fn property_names_and_keys_are_not_references() {
    assert_snapshot!(references(indoc! {"
        declare const obj: { target: number }
        const target = 1
        obj.target
        const o = { target: 2 }
        class C {
          target = 3
          targetMethod() {}
        }
        const { target: renamed } = obj
    "}), @"2:7 target write init");
}

#[test]
fn destructuring_parameters_and_loops() {
    assert_snapshot!(references(indoc! {"
        declare const obj: { a: number; targetB?: number }
        const { a: targetA, targetB = 1 } = obj
        function f(targetC = 1, { targetD = 2 }: { targetD?: number }, targetE: number, [targetF]: number[]) {
          return [targetA, targetB, targetC, targetD, targetE, targetF]
        }
        for (const targetG of [1]) targetG
        for (const targetH in obj) {}
        let targetI: number
        targetI = 1
    "}), @r"
    2:12 targetA write init
    2:21 targetB write init
    3:12 targetC write init
    3:27 targetD write init
    4:11 targetA read
    4:20 targetB read
    4:29 targetC read
    4:38 targetD read
    4:47 targetE read
    4:56 targetF read
    6:12 targetG write init
    6:28 targetG read
    7:12 targetH write init
    9:1 targetI write
    ");
}

#[test]
fn exported_functions_classes_and_jsx() {
    assert_snapshot!(references(indoc! {"
        export const target = 1
        export function targetFn() {
          return target
        }
        export default function targetDefault() {}
        class TargetComponent {}
        targetFn()
        targetDefault()
        const element = <TargetComponent />
        const other = <TargetComponent></TargetComponent>
    "}), @r"
    1:14 target write init
    3:10 target read
    7:1 targetFn read
    8:1 targetDefault read
    9:18 TargetComponent read
    10:16 TargetComponent read
    ");
}

#[test]
fn types_and_imports() {
    assert_snapshot!(references(indoc! {"
        import { useRef as targetRef } from 'react'
        import targetDefault, * as targetNs from 'react'
        type TargetType = { a: number }
        interface TargetInterface {}
        const value: TargetType = { a: 1 }
        let typed: typeof value & TargetInterface
        targetRef()
        targetNs.useState
        targetDefault
    "}), @r"
    5:14 TargetType read
    6:27 TargetInterface read
    7:1 targetRef read
    8:1 targetNs read
    9:1 targetDefault read
    ");
}

#[test]
fn declared_names_of_declarations() {
    assert_snapshot!(declared(indoc! {"
        import def, { a, b as c } from 'react'
        import * as ns from 'react'
        const x = 1, { y, z: [w] } = { y: 1, z: [2] }
        function f(p: number, { q }: { q: number }) {}
        const g = function named(r: number) {}
        const h = (s: number) => s
        class K {}
        try {} catch ({ message }) {}
    "}), @r"
    1: ImportDeclaration: def, a, c
    2: ImportDeclaration: ns
    3: VariableStatement: x, y, w
    4: FunctionDeclaration: f, p, q
    5: VariableStatement: g
    5: FunctionExpression: named, r
    6: VariableStatement: h
    6: ArrowFunction: s
    7: ClassDeclaration: K
    8: CatchClause: message
    ");
}
