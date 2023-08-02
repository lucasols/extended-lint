import { test, describe } from 'vitest'
import { noCommentedOutCode } from '../src/rules/no-commented-code'
import { createTester } from './utils/createTester'

const { valid, invalid } = createTester(noCommentedOutCode, {
  defaultErrorId: 'commentedOutCode',
  ignoreError: {
    code: `
      // This comment includes some code:
      // const answer = 54;
      const answer = 42;
    `,
    errors: [{ messageId: 'commentedOutCode' }],
  },
})

describe('valid code', () => {
  valid(`
      // This comment isn't code.
      const answer = 42;

      // ignore triple slash directives
      /// <reference lib="WebWorker" />

      // TODO: salvar nome do menu localmente pq essa informação não é persistente no servidor
    `)
})

describe('invalid', () => {
  invalid(
    `
      // This comment includes some code:
      // const answer = 54;
      const answer = 42;
    `,
  )
})

describe('invalid block of code', () => {
  invalid(
    `
      // function onClickAdd() {
      //   const coords = getLastCoords();

      //   if (!coords) return;

      //   const pos = editor.view.posAtCoords(coords);

      //   if (!pos) return;

      //   const blockInfo = getBlockInfoFromPos(editor.state.doc, pos.pos);

      //   if (!blockInfo) return;

      //   const { contentNode, endPos } = blockInfo;
  `,
    8,
  )
})

describe('invalid jsx commented prop', () => {
  invalid(
    `
    // getLastCoords={getLastCoords}
    // editor={editor}
    // editor="editor"
  `,
    3,
  )
})

describe('invalid objects', () => {
  invalid(
    `
      // TODO: add to back
      // {
      //   ...normalizedBase,
      //   operator: '==',
      //   valueAlias: {
      //     teste: '2',
      //     sdffsdfds: '3',
      //   },
      // },
      // TODO: add to back
      // {
      //   ...normalizedBase,
      //   operator: 'contains',
      // },
  `,
    4,
  )
})
