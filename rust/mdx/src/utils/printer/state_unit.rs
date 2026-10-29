use crate::actions::script::parse::parse_script;
use crate::models::estree::Store;
use crate::models::name::Key;
use crate::utils::printer::state::print_program;

// Each source as acorn reads it and astring 1.9.0 with estree-util-to-js's JSX handlers prints it
const CASES: &[(&str, &str)] = &[
    ("a = (b, c)", "a = (b, c);\n"),
    ("(function () {})()", "(function () {})();\n"),
    ("new (a())()", "new (a())();\n"),
    ("new a.b()", "new a.b();\n"),
    ("(a ?? b) || c", "(a ?? b) || c;\n"),
    ("a ?? (b || c)", "a ?? (b || c);\n"),
    ("-(-a)", "- -a;\n"),
    ("(-1) ** 2", "(-1) ** 2;\n"),
    ("2 ** -1", "2 ** -1;\n"),
    ("(a ** b) ** c", "(a ** b) ** c;\n"),
    ("a ** b ** c", "a ** b ** c;\n"),
    ("({}).toString()", "({}).toString();\n"),
    ("tag`x${y}z`", "tag`x${y}z`;\n"),
    ("class A extends (a, b) {}", "class A extends (a, b) {}\n"),
    (
        "class A extends B { static x = 1; #y; get z() { return 1 } static { a() } }",
        "class A extends B {\n  static x = 1;\n  #y;\n  get z() {\n    return 1;\n  }\n  static {\n    a();\n  }\n}\n",
    ),
    (
        "for (const a of b) for await (x of y) ;",
        "for (const a of b) for await (x of y) ;\n",
    ),
    (
        "for (let i = 0; i < 1; i++) {}",
        "for (let i = 0; i < 1; i++) {}\n",
    ),
    ("for (a in b) {}", "for (a in b) {}\n"),
    (
        "label: while (a) break label",
        "label: while (a) break label;\n",
    ),
    (
        "switch (a) { case 1: b(); default: c() }",
        "switch (a) {\n  case 1:\n    b();\n  default:\n    c();\n}\n",
    ),
    (
        "try { a() } catch { b() } finally { c() }",
        "try {\n  a();\n} catch {\n  b();\n} finally {\n  c();\n}\n",
    ),
    ("try {} catch (e) {}", "try {} catch (e) {}\n"),
    ("do a(); while (b)", "do a(); while (b);\n"),
    (
        "if (a) b(); else if (c) d(); else e()",
        "if (a) b(); else if (c) d(); else e();\n",
    ),
    (
        "async function* f(a = 1, ...b) { yield* c; await d }",
        "async function* f(a = 1, ...b) {\n  yield* c;\n  await d;\n}\n",
    ),
    (
        "const f = async x => ({a: 1})",
        "const f = async x => ({\n  a: 1\n});\n",
    ),
    (
        "const {a, b: [c, , d], ...e} = f",
        "const {a, b: [c, , d], ...e} = f;\n",
    ),
    ("x = [, a, , ]", "x = [, a, , ];\n"),
    ("(a, b) => c", "((a, b) => c);\n"),
    ("a?.b?.[c]?.(d)", "a?.b?.[c]?.(d);\n"),
    ("import.meta.url", "import.meta.url;\n"),
    ("export * as a from \"b\"", "export * as a from \"b\";\n"),
    ("export * from \"b\"", "export * from \"b\";\n"),
    (
        "import a, * as b from \"c\"",
        "import a, * as b from \"c\";\n",
    ),
    (
        "import a, {b as c, d} from \"e\" with {type: \"json\"}",
        "import a, {b as c, d} from \"e\" with { type: \"json\" };\n",
    ),
    ("import \"a\"", "import \"a\";\n"),
    ("export default (1, 2)", "export default (1, 2);\n"),
    ("export default class {}", "export default class {}\n"),
    (
        "export default function () {}",
        "export default function () {}\n",
    ),
    (
        "export default async () => {}",
        "export default async () => {};\n",
    ),
    (
        "x = /re/gi; y = 1n; z = 0x1F; w = \"a\\u{0041}'\"",
        "x = /re/gi;\ny = 1n;\nz = 0x1F;\nw = \"a\\u{0041}'\";\n",
    ),
    ("x = `a\\n${b}`", "x = `a\\n${b}`;\n"),
    ("a = b ? c : d ? e : f", "a = b ? c : d ? e : f;\n"),
    ("(a ? b : c) ? d : e", "(a ? b : c) ? d : e;\n"),
    ("(a = b) ? c : d", "(a = b) ? c : d;\n"),
    (
        "void 0, typeof a, delete a.b, !a, ~a, +a, -a, ++a, a--",
        "(void 0, typeof a, delete a.b, !a, ~a, +a, -a, ++a, a--);\n",
    ),
    ("- -a", "- -a;\n"),
    ("+ +a", "+ +a;\n"),
    ("- --a", "- --a;\n"),
    ("a in b", "(a in b);\n"),
    ("a instanceof b", "a instanceof b;\n"),
    ("(a || b) && c", "(a || b) && c;\n"),
    ("a && (b || c)", "a && (b || c);\n"),
    ("({a} = b)", "({a} = b);\n"),
    (
        "({a, b() {}, get c() { return 1 }, set c(v) {}, [d]: e, \"f-g\": 1, 1: 2, async *h() {}})",
        "({\n  a,\n  b() {},\n  get c() {\n    return 1;\n  },\n  set c(v) {},\n  [d]: e,\n  \"f-g\": 1,\n  1: 2,\n  async *h() {}\n});\n",
    ),
    ("a = {}", "a = {};\n"),
    ("debugger", "debugger;\n"),
    ("`${a}${b}`", "`${a}${b}`;\n"),
    ("x = function* () {}", "x = function* () {};\n"),
    ("(async function () {})", "(async function () {});\n"),
    ("new (class {})()", "new (class {})();\n"),
    ("new (function () {})()", "new (function () {})();\n"),
    ("(() => {})()", "(() => {})();\n"),
    ("a = b => c => d", "a = b => c => d;\n"),
    ("a = (b) => (c, d)", "a = b => (c, d);\n"),
    (
        "x = <a b=\"c\" d={e} {...f}>g {h} &amp;</a>",
        "x = <a b=\"c\" d={e} {...f}>g {h} &</a>;\n",
    ),
    ("x = <></>", "x = <></>;\n"),
    ("x = <a.b.c />", "x = <a.b.c />;\n"),
    ("x = <a:b c:d=\"e\" />", "x = <a:b c:d=\"e\" />;\n"),
    ("x = <a>{/* c */}</a>", "/*c*/\nx = <a>{}</a>;\n"),
    (
        "/* a */\nconst b = 1 // c\n/* d */\nconst e = {\n  // f\n  g: 1\n}",
        "/*a*/\n// c\n/*d*/\n// f\nconst b = 1;\nconst e = {\n  g: 1\n};\n",
    ),
    ("a = b in c ? d : e", "a = (b in c) ? d : e;\n"),
    ("for ((a in b);;) {}", "for ((a in b); ; ) {}\n"),
    ("x = a?.b.c", "x = a?.b.c;\n"),
    ("(a?.b).c", "a?.b.c;\n"),
    ("new (a?.b)()", "new a?.b();\n"),
    ("a = (await b)()", "a = (await b)();\n"),
    ("if (a) {} else {}", "if (a) {} else {}\n"),
    ("a = {\"b\": c}", "a = {\n  \"b\": c\n};\n"),
    (
        "class A { \"b\"() {} [c] = 1; static async *d() {} }",
        "class A {\n  \"b\"() {}\n  [c] = 1;\n  static async *d() {}\n}\n",
    ),
    ("let a = 1, b", "let a = 1, b;\n"),
    ("var [a = 1] = b", "var [a = 1] = b;\n"),
    (
        "a ||= b; c &&= d; e ??= f; g **= 2",
        "a ||= b;\nc &&= d;\ne ??= f;\ng **= 2;\n",
    ),
    (
        "a = 1e21; b = 0.1; c = .5",
        "a = 1e21;\nb = 0.1;\nc = .5;\n",
    ),
    ("\"use strict\"\na()", "\"use strict\";\na();\n"),
    (
        "x = <a b='c &quot; &#x41; &#65; &unknown; &'/>",
        "x = <a b=\"c &quot; A A &amp;unknown; &\" />;\n",
    ),
    ("x = <a>\n  b\n  c\n</a>", "x = <a>\n  b\n  c\n</a>;\n"),
    (
        "export const {a, b: [c]} = d",
        "export const {a, b: [c]} = d;\n",
    ),
    ("export function f() {}", "export function f() {}\n"),
    ("export class C {}", "export class C {}\n"),
    ("a = b ? (c, d) : e", "a = b ? (c, d) : e;\n"),
    ("x = a => {}", "x = a => {};\n"),
    ("x = async (a) => a", "x = async a => a;\n"),
    ("x = (a = 1) => a", "x = (a = 1) => a;\n"),
    ("x = ({a}) => a", "x = ({a}) => a;\n"),
    ("(a, b)", "(a, b);\n"),
    ("a = typeof (() => {})", "a = typeof (() => {});\n"),
    ("a = !(b && c)", "a = !(b && c);\n"),
    ("a = -(b + c)", "a = -(b + c);\n"),
    ("a = (b + c) * d", "a = (b + c) * d;\n"),
    ("a = b + (c * d)", "a = b + c * d;\n"),
    ("a = b - (c - d)", "a = b - (c - d);\n"),
    ("a = (b - c) - d", "a = b - c - d;\n"),
];

#[test]
fn prints_programs_as_astring() -> Result<(), textlane_core::Error> {
    for (input, expected) in CASES {
        let mut store = Store::default();
        let (program, comments) = parse_script(&mut store, input, 2)?;
        store.set(
            program,
            Key::COMMENTS,
            crate::models::estree::Value::nodes(comments),
        );
        assert_eq!(print_program(&store, program), *expected, "{input}");
    }
    Ok(())
}
