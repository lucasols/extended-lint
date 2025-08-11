import { createTester } from '../../tests/utils/createTester'
import { noCommentedOutCode } from './no-commented-out-code'

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
  8,
)

tests.addInvalid(
  'invalid jsx commented component',
  `
    /* <Component /> */
  `,
  1,
)

tests.addValid(
  'valid jsx commented component',
  `
    /* INFO: This is a comment
      <Component />
    */
  `,
)

tests.addValid(
  'ingore jsdocs comments',
  `
    /**
     * This is a comment
     * <Component />
     */
  `,
)

tests.addInvalid(
  'invalid commented object property',
  `
    const obj = {
      teste: '2',
      // sdffsdfds: '3',
      // ['teste']: '2',
      // test_e: '2',
      // test-e:
      //   '2',
    }
  `,
  5,
)

tests.addInvalid(
  'invalid commented object with different value types',
  `
    const config = {
      name: 'test',
      // age: 25,
      // isActive: true,
      // items: [],
      // settings: {},
      // getValue: function() { return 42; }
    }
  `,
  4,
)

tests.addInvalid(
  'invalid commented object methods',
  `
    const api = {
      getData() { return data; },
      // fetchUser() { return fetch('/user'); },
      // async loadData() { return await fetch('/data'); },
      // arrow: () => console.log('test'),
    }
  `,
  3,
)

tests.addInvalid(
  'invalid commented object with computed properties',
  `
    const obj = {
      [key]: 'value',
      // [computedKey]: 'computed',
      // ['dynamic']: dynamicValue,
      // [\`template\`]: templateValue,
    }
  `,
  3,
)

tests.addInvalid(
  'invalid commented object with nested structures',
  `
    const complex = {
      user: { name: 'John' },
      // profile: { age: 30, city: 'NYC' },
      // preferences: {
      //   theme: 'dark',
      //   notifications: true
      // },
    }
  `,
  5,
)

tests.addValid(
  'valid commented object shorthand properties',
  `
    const user = { name, age };
    const data = {
      user,
      // name,
      // age,
      // isActive,
    }
  `,
)

tests.addValid(
  'valid commented object with spread',
  `
    const base = { a: 1 };
    const extended = {
      ...base,
      // ...otherProps,
      // ...userSettings,
      b: 2
    }
  `,
)

tests.addInvalid(
  'invalid commented object getters and setters',
  `
    const obj = {
      _value: 0,
      // get value() { return this._value; },
      // set value(val) { this._value = val; },
    }
  `,
  2,
)

tests.addInvalid(
  'invalid commented object with quotes and special chars',
  `
    const obj = {
      'normal-key': 'value',
      // 'quoted-key': 'quoted value',
      // "double-quoted": "double value",
      // 'key with spaces': 'spaced value',
      // 'key:with:colons': 'colon value',
    }
  `,
  4,
)

tests.addInvalid(
  'invalid commented object trailing commas',
  `
    const obj = {
      first: 1,
      // second: 2,
      // third: 3,
    }
  `,
  2,
)

tests.addValid(
  'valid object with allowed prefix comments',
  `
    const config = {
      enabled: true,
      // TODO: add timeout property
      // NOTE: cache property is deprecated
      // FIXME: fix the validation logic
      debug: false
    }
  `,
)

tests.addValid(
  'valid object with descriptive comments',
  `
    const settings = {
      // Database configuration
      host: 'localhost',
      // API endpoints configuration  
      apiUrl: '/api/v1'
    }
  `,
)

tests.addInvalid(
  'invalid condition',
  `
    if (test
      // && test_2
      // || test_3
      // ? test_4
      // : test_5
    ) {
      console.log('test')
    }
  `,
  4,
)

tests.addInvalid(
  'invalid react hooks',
  `
    // const [state, setState] = useState(0)
    // useEffect(() => {
    //   console.log('effect')
    // }, [])
    // const memoValue = useMemo(() => value * 2, [value])
  `,
  5,
)

tests.addInvalid(
  'invalid modern js patterns',
  `
    // const result = data?.map(item => item.value)
    // const fallback = value ?? 'default'
    // await fetchData()
    // items.filter(x => x.active)
  `,
  4,
)

tests.addInvalid(
  'invalid console and dom',
  `
    // console.log('debug')
    // document.getElementById('test')
    // window.location.href = '/new-page'
  `,
  2,
)

tests.addInvalid(
  'invalid try catch',
  `
    // try {
    //   doSomething()
    // } catch (error) {
    //   console.error(error)
    // }
  `,
  4,
)

tests.addValid(
  'valid with FIXME prefix',
  `
    // FIXME: const broken = getSomething()
    const answer = 42;
  `,
)

tests.addValid(
  'valid with NOTE prefix',
  `
    // NOTE: useState was causing issues here
    const answer = 42;
  `,
)

tests.addValid(
  'valid with HACK prefix',
  `
    // HACK: temporarily disable this feature
    const answer = 42;
  `,
)

tests.addValid(
  'valid typescript comments',
  `
    // @ts-ignore
    // @ts-expect-error
    const answer = 42;
  `,
)

tests.addValid(
  'valid jsdoc patterns',
  `
    /**
     * @param name - The user name
     * @returns The greeting message
     * @throws Error when name is empty
     */
    function greet(name: string) {
      return \`Hello \${name}\`;
    }
  `,
)

tests.addInvalid(
  'invalid commented array strings',
  `
    const items = [
      'first',
      // 'second',
      // 'third',
      'fourth'
    ]
  `,
  2,
)

tests.addInvalid(
  'invalid commented array numbers',
  `
    const numbers = [
      1,
      // 2,
      // 3,
      4
    ]
  `,
  2,
)

tests.addInvalid(
  'invalid commented array objects',
  `
    const users = [
      { name: 'John', age: 30 },
      // { name: 'Jane', age: 25 },
      // { name: 'Bob', age: 35 },
    ]
  `,
  2,
)

tests.addInvalid(
  'invalid commented mixed array items',
  `
    const mixed = [
      'string',
      // 42,
      // true,
      // null,
      // undefined,
      { key: 'value' }
    ]
  `,
  1,
)

tests.addInvalid(
  'invalid commented array with functions',
  `
    const handlers = [
      onClick,
      // onHover,
      // () => console.log('click'),
      // function handler() { return true; }
    ]
  `,
  2,
)

tests.addInvalid(
  'invalid commented nested arrays',
  `
    const matrix = [
      [1, 2, 3],
      // [4, 5, 6],
      // [7, 8, 9]
    ]
  `,
  1,
)

tests.addValid(
  'valid array with allowed prefix comments',
  `
    const items = [
      'first',
      // TODO: add second item later
      // NOTE: third item is not ready yet
      'fourth'
    ]
  `,
)

tests.addValid(
  'valid array with regular comments',
  `
    const config = [
      // Configuration for development
      'dev-setting',
      // Configuration for production  
      'prod-setting'
    ]
  `,
)

tests.addInvalid(
  'invalid commented array with template literals',
  `
    const templates = [
      \`hello world\`,
      // \`template \${variable}\`,
      // \`another template\`,
    ]
  `,
  2,
)

tests.addInvalid(
  'invalid commented jsx array items',
  `
    const components = [
      <div>first</div>,
      // <span>second</span>,
      // <Component prop="value" />,
    ]
  `,
  2,
)

tests.addInvalid(
  'invalid commented array methods',
  `
    const items = [
      'item1',
      // 'item2'.toUpperCase(),
      // getValue(),
      // obj.method(),
    ]
  `,
  3,
)

tests.addInvalid(
  'invalid array with descriptive comments',
  `
    const routes = [
      '/home',
      // Routes below are for admin users only
      // '/admin',
      '/profile'
    ]
  `,
  1,
)

tests.addValid(
  'valid commented array spread',
  `
    const combined = [
      ...firstArray,
      // ...secondArray,
      // ...otherItems,
    ]
  `,
)

tests.addInvalid(
  'invalid commented object with numeric keys',
  `
    const data = {
      0: 'first',
      // 1: 'second',
      // 2: 'third',
      3: 'fourth'
    }
  `,
  2,
)

tests.addInvalid(
  'invalid commented object with mixed key types',
  `
    const mixed = {
      regularKey: 'value',
      // 'string-key': 'string value',
      // 42: 'numeric key',
      // [dynamicKey]: 'dynamic value',
    }
  `,
  3,
)

tests.addInvalid(
  'invalid commented object destructuring patterns',
  `
    const obj = {
      a: 1,
      // b: 2,
      c: 3
    };
    // const { a, b, c } = obj;
    // const { x = 1, y = 2 } = props;
  `,
  3,
)

tests.addInvalid(
  'invalid commented object with function values',
  `
    const handlers = {
      onClick: handleClick,
      // onSubmit: handleSubmit,
      // validate: (data) => data.isValid,
      // async process() { return await api.call(); }
    }
  `,
  3,
)

tests.addValid(
  'valid object with complex comment patterns',
  `
    const config = {
      // TEMP: using development settings
      apiUrl: 'dev-api.com',
      // NOTE: production timeout should be higher
      timeout: 5000,
      // TODO: implement retry logic
      retry: false
    }
  `,
)

tests.addValid(
  'valid regular block comment',
  `
    /* This is a regular block comment */
    const test = 1;
    
    /* 
     * Multi-line block comment
     * with code-like patterns like function() 
     * and const x = 1 should be allowed
     */
    const test2 = 2;
  `,
)

tests.addInvalid(
  'invalid jsx in block comment',
  `
    /* <div>JSX content</div> */
    const test = 1;
  `,
  1,
)

tests.addValid(
  'valid eslint comment',
  `
    /* slint react-compiler/react-compiler: ["error"] */
    import { css } from '@linaria/core';
    import { styled } from '@linaria/react';
  `,
)

tests.addValid(
  'valid comments with inline code examples',
  `
    // The \`dataTransfer.setDragImage(element)\` method only works if element is attached
    // Use \`useState()\` hook for state management  
    // Call \`myObject.method()\` to execute the function
    // Set config option \`enabled: true\` in the settings
  `,
)

tests.addValid(
  'valid comments with markdown inline code',
  `
    // This function uses \`Array.map()\` internally
    // The \`console.log()\` is for debugging purposes
    // Set \`config.enabled = true\` to activate
  `,
)

tests.addInvalid(
  'invalid commented code should still be detected outside backticks',
  `
    // This is a comment but console.log() is actual code
    // Array.map() without backticks should be detected
  `,
  2,
)

tests.addValid(
  'invalid commented code with backticks',
  `
    // If there are droppables intersecting with the pointer, return those
    // If a container is matched and it contains items (columns 'A', 'B', 'C')  
    // https://www.google.com/maps/place?q=place_id:ChIJN1t_tDeuEmsRUsoyG83frY4
  `,
)

tests.addInvalid(
  'invalid commented code with return',
  `
    function test() {
      // return 'test'
    }
  `,
  1,
)

tests.addValid(
  'Comment with colon and text should be allowed',
  `
    // Third pass: find exported constants
    // Second: this if a fine comment too
    // Step 1: Create all tables and their fields
  `,
)

tests.run()
