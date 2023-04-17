import { describe, test } from 'vitest'
import { noUnusedObjectTypeProperties } from '../src/rules/no-unused-type-props-in-args'
import { createTester } from './utils/createTester'
import { noCommentedOutCode } from '../src/rules/no-commented-code'

const { valid, invalid } = createTester(noCommentedOutCode, 'commentedOutCode')

test('valid code', () => {
  valid(`
      // This comment isn't code.
      const answer = 42;

      // ignore triple slash directives
      /// <reference lib="WebWorker" />

      // TODO: salvar nome do menu localmente pq essa informação não é persistente no servidor
    `)
})

test('invalid', () => {
  invalid(
    `
      // This comment includes some code:
      // const answer = 54;
      const answer = 42;
    `,
  )
})

test('invalid block of code', () => {
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

test('invalid jsx commented prop', () => {
  invalid(
    `
    // getLastCoords={getLastCoords}
    // editor={editor}
    // editor="editor"
  `,
    3,
  )
})

test('invalid objects', () => {
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
