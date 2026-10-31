import type * as upstream from "mdast"
import { describe, expectTypeOf, it } from "vite-plus/test"
import type * as md from "../generated/md.ts"
import type { hast, mdast } from "../index.ts"
import type { Parser } from "../models/plugin.ts"

describe("the tree namespaces", () => {
  it("are the unified ecosystem's own types", () => {
    expectTypeOf<mdast.Root>().toEqualTypeOf<upstream.Root>()
    expectTypeOf<ReturnType<typeof md.parse>>().toExtend<Parser<mdast.Root>>()
  })

  it("hold the nodes textlane's syntax extensions make", () => {
    expectTypeOf<mdast.MdxJsxFlowElement>().toExtend<mdast.BlockContent>()
    expectTypeOf<mdast.MdxjsEsm>().toExtend<mdast.RootContent>()
    expectTypeOf<mdast.ContainerDirective>().toExtend<mdast.BlockContent>()
    expectTypeOf<mdast.TextDirective>().toExtend<mdast.PhrasingContent>()
    expectTypeOf<mdast.InlineMath>().toExtend<mdast.PhrasingContent>()
    expectTypeOf<mdast.DefListNode>().toExtend<mdast.BlockContent>()
    expectTypeOf<mdast.Toml>().toExtend<mdast.FrontmatterContent>()
    expectTypeOf<hast.MdxJsxTextElementHast>().toExtend<hast.ElementContent>()
    expectTypeOf<hast.Raw>().toExtend<hast.ElementContent>()
  })

  it("type the data mdast-util-to-hast reads", () => {
    expectTypeOf<mdast.Data["hName"]>().toEqualTypeOf<string | undefined>()
  })
})
