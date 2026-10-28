import type { AnyNode, Program } from "acorn"

/** Constants visible at a node: those declared in each enclosing block, from the module inward,
 * an inner declaration shadowing an outer one of the same name. */
export function resolveConstants(props: { program: Program; ancestors: AnyNode[] }) {
  const constants = new Map<string, AnyNode>()
  const scopes =
    props.ancestors[0] === props.program
      ? props.ancestors
      : [props.program, ...props.ancestors]
  for (const scope of scopes) {
    const statements =
      scope.type === "Program" || scope.type === "BlockStatement" ? scope.body : []
    for (const statement of statements) {
      const declaration =
        statement.type === "ExportNamedDeclaration" ? statement.declaration : statement
      if (declaration?.type !== "VariableDeclaration") continue
      for (const declarator of declaration.declarations) {
        if (declarator.id.type === "Identifier" && declarator.init)
          constants.set(declarator.id.name, declarator.init)
      }
    }
  }
  return constants
}
