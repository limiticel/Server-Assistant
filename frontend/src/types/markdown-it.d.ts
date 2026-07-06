declare module 'markdown-it' {
  type RendererRule = (
    tokens: Array<{
      attrs: string[][] | null
      content: string
      info: string
      attrIndex(name: string): number
      attrPush(attrData: [string, string]): void
    }>,
    idx: number,
    options: unknown,
    env: unknown,
    self: {
      renderToken(tokens: unknown[], idx: number, options: unknown): string
    }
  ) => string

  export default class MarkdownIt {
    constructor(options?: Record<string, unknown>)
    renderer: {
      rules: Record<string, RendererRule | undefined>
    }
    render(content: string): string
  }
}
