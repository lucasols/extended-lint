import { noCommentedOutCode } from '../src/rules/no-commented-code'
import { createTester } from './utils/createTester'

const tests = createTester(noCommentedOutCode, {
  defaultErrorId: 'commentedOutCode',
})

tests.addValid(
  'valid code',
  `
      // This comment isn't code.
      const answer = 42;

      // ignore triple slash directives
      /// <reference lib="WebWorker" />

      // TODO: salvar nome do menu localmente pq essa informação não é persistente no servidor
    `,
)

tests.addInvalid(
  'invalid',
  `
      // This comment includes some code:
      // const answer = 54;
      const answer = 42;
    `,
  'default-error',
)

tests.addInvalid(
  'invalid block of code',
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

tests.addInvalid(
  'invalid jsx commented prop',
  `
    // getLastCoords={getLastCoords}
    // editor={editor}
    // editor="editor"
  `,
  3,
)

tests.addInvalid(
  'invalid objects',
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

tests.run()
