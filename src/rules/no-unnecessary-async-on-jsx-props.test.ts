import { createTester } from '../../tests/utils/createTester'
import { noUnnecessaryAsyncOnJsxProps } from './no-unnecessary-async-on-jsx-props'

const tester = createTester(noUnnecessaryAsyncOnJsxProps, {
  defaultErrorId: 'unnecessaryAsyncInJsxProp',
})

tester.addValid(
  'async function with await outside JSX',
  `
    const handler = async () => {
      await fetchData()
    }
  `,
)

tester.addValid(
  'regular function in JSX prop',
  `
    const Component = () => (
      <button onClick={() => {
        doSomething()
      }}>
        Click me
      </button>
    )
  `,
)

tester.addValid(
  'async function with multiple awaits in JSX prop',
  `
    const Component = () => (
      <button onClick={async () => {
        await saveData()
        await logEvent()
      }}>
        Click me
      </button>
    )
  `,
)

tester.addValid(
  'async function with no awaits in JSX prop (handled by other rules)',
  `
    const Component = () => (
      <button onClick={async () => {
        console.log('no await here')
        doSomething()
      }}>
        Click me
      </button>
    )
  `,
)

tester.addValid(
  'async function with await in function call with object arguments',
  `
    <ButtonElement
      onClick={async () => {
        const result = await uiDialog.prompt({
          title: 'Prompt with confirm function',
          description: 'This is a prompt with confirm function',
          onConfirm: async (value) => {
            await sleep(2000);
            if (value.toLowerCase() === 'error') {
              throw new Error('Error');
            }
            return Result.ok(value);
          },
        });
      }}
    />
  `,
)

tester.addValid(
  'async function with single await but additional statements',
  `
    <button onClick={async () => {
      await saveData();
      console.log('Data saved successfully');
      showNotification();
    }}>
      Save
    </button>
  `,
)

tester.addInvalid(
  'async function with single await in JSX prop',
  `
    const Component = () => (
      <button onClick={async () => {
        await removeColumnFromKanban(appId, blockId, columnValue)
      }}>
        Click me
      </button>
    )
  `,
  1,
  {
    output: `
      const Component = () => (
        <button onClick={() => {
          removeColumnFromKanban(appId, blockId, columnValue)
        }}>
          Click me
        </button>
      )
    `,
  },
)

tester.addInvalid(
  'async arrow function in event handler',
  `
    <div onClick={async () => {
      await doSomething()
    }} />
  `,
  1,
  {
    output: `
      <div onClick={() => {
        doSomething()
      }} />
    `,
  },
)

tester.addInvalid(
  'async function expression in JSX prop',
  `
    <input onChange={async function(e) {
      await handleChange(e.target.value)
    }} />
  `,
  1,
  {
    output: `
      <input onChange={function(e) {
        handleChange(e.target.value)
      }} />
    `,
  },
)

tester.run()
