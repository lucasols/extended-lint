import { dedent } from '@ls-stack/utils/dedent'
import { expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsFromResult,
} from '../../tests/utils/createTester'
import { useTypesDirectlyAboveUsage } from './use-types-directly-above-usage'

const { valid, invalid } = createNewTester(useTypesDirectlyAboveUsage)

test('type alias directly above function', async () => {
  // Valid cases - no errors expected
  await valid(
    dedent`
      type UserData = { name: string }
      
      function processUser(data: UserData) {
        return data.name
      }
    `,
  )
})

test('interface directly above function', async () => {
  await valid(
    dedent`
      interface Config {
        debug: boolean
      }
      
      function setupApp(config: Config) {
        console.log(config.debug)
      }
    `,
  )
})

test('props type above React component', async () => {
  await valid(
    dedent`
      type ButtonProps = {
        onClick: () => void
        children: string
      }
      
      function Button(props: ButtonProps) {
        return <button onClick={props.onClick}>{props.children}</button>
      }
    `,
  )
})

test('props type above React FC component', async () => {
  await valid(
    dedent`
      type CardProps = {
        title: string
        content: string
      }
      
      const Card: React.FC<CardProps> = ({ title, content }) => {
        return <div><h2>{title}</h2><p>{content}</p></div>
      }
    `,
  )
})

test('props type above React FC component with import alias', async () => {
  await valid(
    dedent`
      import { FC } from 'react'
      
      type HeaderProps = {
        text: string
      }
      
      const Header: FC<HeaderProps> = ({ text }) => {
        return <h1>{text}</h1>
      }
    `,
  )
})

test('type above arrow function', async () => {
  await valid(
    dedent`
      type Handler = (id: string) => void
      
      const onClick: Handler = (id) => {
        console.log(id)
      }
    `,
  )
})

test('type correctly above first usage when used multiple times', async () => {
  await valid(
    dedent`
      type SharedType = { value: number }
      
      function processA(data: SharedType) {
        return data.value
      }
      
      function processB(input: SharedType) {
        return input.value * 2
      }
    `,
  )
})

test('type correctly above first usage in param and return type', async () => {
  await valid(
    dedent`
      type Result = { data: string }
      
      function transform(x: string): Result {
        return { data: x }
      }
      
      function create(): Result {
        return { data: 'default' }
      }
    `,
  )
})

test('inline type annotations are allowed', async () => {
  await valid(
    dedent`
      function process(data: { name: string, age: number }) {
        return data.name
      }
    `,
  )
})

test('generic type parameters are allowed', async () => {
  await valid(
    dedent`
      function map<T>(items: T[]): T[] {
        return items
      }
    `,
  )
})

test('imported types are ignored', async () => {
  await valid(
    dedent`
      import { ApiResponse } from './types'
      
      type LocalData = string
      
      function handleResponse(response: ApiResponse) {
        return response.data
      }
    `,
  )
})

test('multiple types each above their respective functions', async () => {
  await valid(
    dedent`
      type UserConfig = { theme: string }

      function setupUser(config: UserConfig) {
        return config.theme
      }

      type DatabaseConfig = { host: string }

      function connectDb(config: DatabaseConfig) {
        return config.host
      }
    `,
  )
})

// Invalid cases - should trigger errors and provide fixes
test('type alias below function that uses it', async () => {
  const { result } = await invalid(
    dedent`
      function processUser(data: UserData) {
        return data.name
      }
      
      type UserData = { name: string }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type UserData = { name: string }

    function processUser(data: UserData) {
      return data.name
    }

    "
  `)
})

test('interface below function that uses it', async () => {
  const { result } = await invalid(
    dedent`
      function setupApp(config: Config) {
        console.log(config.debug)
      }
      
      interface Config {
        debug: boolean
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "interface Config {
      debug: boolean
    }

    function setupApp(config: Config) {
      console.log(config.debug)
    }

    "
  `)
})

test('props type below React component', async () => {
  const { result } = await invalid(
    dedent`
      function Button(props: ButtonProps) {
        return <button onClick={props.onClick}>{props.children}</button>
      }
      
      type ButtonProps = {
        onClick: () => void
        children: string
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type ButtonProps = {
      onClick: () => void
      children: string
    }

    function Button(props: ButtonProps) {
      return <button onClick={props.onClick}>{props.children}</button>
    }

    "
  `)
})

test('props type below React FC component', async () => {
  const { result } = await invalid(
    dedent`
      const Card: React.FC<CardProps> = ({ title, content }) => {
        return <div><h2>{title}</h2><p>{content}</p></div>
      }
      
      type CardProps = {
        title: string
        content: string
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type CardProps = {
      title: string
      content: string
    }

    const Card: React.FC<CardProps> = ({ title, content }) => {
      return <div><h2>{title}</h2><p>{content}</p></div>
    }

    "
  `)
})

test('props type below FC component with import alias', async () => {
  const { result } = await invalid(
    dedent`
      import { FC } from 'react'
      
      const Header: FC<HeaderProps> = ({ text }) => {
        return <h1>{text}</h1>
      }
      
      type HeaderProps = { text: string }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 7
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "import { FC } from 'react'

    type HeaderProps = { text: string }

    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }

    "
  `)
})

test('type below arrow function', async () => {
  const { result } = await invalid(
    dedent`
      const onClick: Handler = (id) => {
        console.log(id)
      }
      
      type Handler = (id: string) => void
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Handler = (id: string) => void

    const onClick: Handler = (id) => {
      console.log(id)
    }

    "
  `)
})

test('type below function with return type annotation', async () => {
  const { result } = await invalid(
    dedent`
      function createUser(): UserType {
        return { name: 'John' }
      }
      
      type UserType = { name: string }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type UserType = { name: string }

    function createUser(): UserType {
      return { name: 'John' }
    }

    "
  `)
})

test('multiple misplaced types each used by single function', async () => {
  const { result } = await invalid(
    dedent`
      function processA(data: TypeA) {
        return data.valueA
      }
      
      function processB(data: TypeB) {
        return data.valueB
      }
      
      type TypeA = { valueA: string }
      type TypeB = { valueB: number }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 9
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type TypeA = { valueA: string }

    function processA(data: TypeA) {
      return data.valueA
    }

    type TypeB = { valueB: number }

    function processB(data: TypeB) {
      return data.valueB
    }


    "
  `)
})

test('type between functions where first function uses it', async () => {
  const { result } = await invalid(
    dedent`
      function processData(input: DataType) {
        return input.value
      }
      
      type DataType = { value: string }
      
      function otherFunction() {
        return 'hello'
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type DataType = { value: string }

    function processData(input: DataType) {
      return input.value
    }



    function otherFunction() {
      return 'hello'
    }"
  `)
})

test('interface used in function parameter and return type by same function only', async () => {
  const { result } = await invalid(
    dedent`
      function transform(input: Data): Data {
        return { ...input, processed: true }
      }

      interface Data {
        processed?: boolean
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "interface Data {
      processed?: boolean
    }

    function transform(input: Data): Data {
      return { ...input, processed: true }
    }

    "
  `)
})

test('type used in generic constraint', async () => {
  const { result } = await invalid(
    dedent`
      function filter<T extends Filterable>(items: T[]): T[] {
        return items.filter(item => item.isActive)
      }
      
      type Filterable = { isActive: boolean }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Filterable = { isActive: boolean }

    function filter<T extends Filterable>(items: T[]): T[] {
      return items.filter(item => item.isActive)
    }

    "
  `)
})

// Edge cases that should be handled correctly
test('type used in variable declaration with function expression', async () => {
  await valid(
    dedent`
      type Handler = () => void
      
      const myHandler: Handler = function() {
        console.log('handled')
      }
    `,
  )
})

// Variable declaration tests
test('variable declaration with type below - options example', async () => {
  const { result } = await invalid(
    dedent`
      const options: Options = {
        debug: true,
        timeout: 5000
      }
      
      type Options = {
        debug: boolean
        timeout: number
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 6
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Options = {
      debug: boolean
      timeout: number
    }

    const options: Options = {
      debug: true,
      timeout: 5000
    }

    "
  `)
})

test('type correctly above variable declaration', async () => {
  await valid(
    dedent`
      type Config = {
        apiUrl: string
        retries: number
      }
      
      const config: Config = {
        apiUrl: 'https://api.example.com',
        retries: 3
      }
    `,
  )
})

test('multiple variables using same type - should be above first usage', async () => {
  await valid(
    dedent`
      type Settings = {
        theme: string
        fontSize: number
      }
      
      const userSettings: Settings = { theme: 'dark', fontSize: 14 }
      const defaultSettings: Settings = { theme: 'light', fontSize: 12 }
    `,
  )
})

test('type correctly above first usage when used in nested and direct contexts', async () => {
  await valid(
    dedent`
      type UserInfo = { name: string }
      
      function processUser(data: { user: UserInfo }) {
        return data.user.name
      }
      
      function validateUser(info: UserInfo) {
        return info.name.length > 0
      }
    `,
  )
})

test('props type correctly above first FC component when used multiple times', async () => {
  await valid(
    dedent`
      type SharedProps = { label: string }
      
      const Button: React.FC<SharedProps> = ({ label }) => {
        return <button>{label}</button>
      }
      
      const Link: React.FC<SharedProps> = ({ label }) => {
        return <a href="#">{label}</a>
      }
    `,
  )
})

test('props type correctly above first usage in mixed components', async () => {
  await valid(
    dedent`
      type CommonProps = { text: string }
      
      function Button(props: CommonProps) {
        return <button>{props.text}</button>
      }
      
      const Card: React.FC<CommonProps> = ({ text }) => {
        return <div>{text}</div>
      }
    `,
  )
})

test('type with comments should preserve formatting', async () => {
  const { result } = await invalid(
    dedent`
      function processData(input: MyType) {
        return input.value
      }
      
      // This is an important type
      type MyType = {
        // The main value
        value: string
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 6
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "// This is an important type
    type MyType = {
      // The main value
      value: string
    }

    function processData(input: MyType) {
      return input.value
    }

    "
  `)
})

// Invalid cases for types used by multiple functions - should move above first usage
test('shared type should move above first function usage', async () => {
  const { result } = await invalid(
    dedent`
      function processA(data: SharedType) {
        return data.value
      }
      
      function processB(input: SharedType) {
        return input.value * 2
      }
      
      type SharedType = { value: number }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 9
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type SharedType = { value: number }

    function processA(data: SharedType) {
      return data.value
    }

    function processB(input: SharedType) {
      return input.value * 2
    }

    "
  `)
})

test('props type should move above first FC component', async () => {
  const { result } = await invalid(
    dedent`
      const Button: React.FC<SharedProps> = ({ label }) => {
        return <button>{label}</button>
      }
      
      const Link: React.FC<SharedProps> = ({ label }) => {
        return <a href="#">{label}</a>
      }
      
      type SharedProps = { label: string }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 9
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type SharedProps = { label: string }

    const Button: React.FC<SharedProps> = ({ label }) => {
      return <button>{label}</button>
    }

    const Link: React.FC<SharedProps> = ({ label }) => {
      return <a href="#">{label}</a>
    }

    "
  `)
})

test('shared props should move above first component (mixed function and FC)', async () => {
  const { result } = await invalid(
    dedent`
      function Button(props: CommonProps) {
        return <button>{props.text}</button>
      }
      
      const Card: React.FC<CommonProps> = ({ text }) => {
        return <div>{text}</div>
      }
      
      type CommonProps = { text: string }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 9
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type CommonProps = { text: string }

    function Button(props: CommonProps) {
      return <button>{props.text}</button>
    }

    const Card: React.FC<CommonProps> = ({ text }) => {
      return <div>{text}</div>
    }

    "
  `)
})

test('type used in param and return type should move above first usage', async () => {
  const { result } = await invalid(
    dedent`
      function transform(x: string): Result {
        return { data: x }
      }
      
      function create(): Result {
        return { data: 'default' }
      }
      
      type Result = { data: string }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 9
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Result = { data: string }

    function transform(x: string): Result {
      return { data: x }
    }

    function create(): Result {
      return { data: 'default' }
    }

    "
  `)
})

test('variable declaration with type below - config example', async () => {
  const { result } = await invalid(
    dedent`
      const config: Config = {
        apiUrl: 'https://api.example.com',
        retries: 3
      }
      
      type Config = {
        apiUrl: string
        retries: number
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 6
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Config = {
      apiUrl: string
      retries: number
    }

    const config: Config = {
      apiUrl: 'https://api.example.com',
      retries: 3
    }

    "
  `)
})

test('multiple variables with shared type - should move above first variable', async () => {
  const { result } = await invalid(
    dedent`
      const userSettings: Settings = { theme: 'dark', fontSize: 14 }
      const defaultSettings: Settings = { theme: 'light', fontSize: 12 }
      
      type Settings = {
        theme: string
        fontSize: number
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 4
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Settings = {
      theme: string
      fontSize: number
    }

    const userSettings: Settings = { theme: 'dark', fontSize: 14 }
    const defaultSettings: Settings = { theme: 'light', fontSize: 12 }

    "
  `)
})

test('mixed function and variable usage - should move above first occurrence', async () => {
  const { result } = await invalid(
    dedent`
      function processConfig(cfg: AppConfig) {
        return cfg.debug
      }
      
      const defaultConfig: AppConfig = {
        debug: false,
        version: '1.0.0'
      }
      
      type AppConfig = {
        debug: boolean
        version: string
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 10
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type AppConfig = {
      debug: boolean
      version: string
    }

    function processConfig(cfg: AppConfig) {
      return cfg.debug
    }

    const defaultConfig: AppConfig = {
      debug: false,
      version: '1.0.0'
    }

    "
  `)
})

test('variable comes first - type should move above variable not function', async () => {
  const { result } = await invalid(
    dedent`
      const state: State = { loading: false }
      
      function updateState(newState: State) {
        return { ...state, ...newState }
      }
      
      type State = {
        loading: boolean
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 7
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type State = {
      loading: boolean
    }

    const state: State = { loading: false }

    function updateState(newState: State) {
      return { ...state, ...newState }
    }

    "
  `)
})

test('type below variable - now always checked', async () => {
  const { result } = await invalid(
    dedent`
      const options: Options = { debug: true }
      
      type Options = { debug: boolean }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 3
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Options = { debug: boolean }

    const options: Options = { debug: true }

    "
  `)
})

test('type not directly above usage with code in between', async () => {
  const { result } = await invalid(
    dedent`
      type Options = {
        debug: boolean
        timeout: number
        retries: number
      }

      const defaultTimeout = 5000
      const maxRetries = 3

      function performNetworkCall(url: string) {
        return fetch(url)
      }

      function validateUrl(url: string): boolean {
        return url.startsWith('https://')
      }

      class NetworkService {
        private baseUrl = 'https://api.example.com'

        async request(endpoint: string) {
          const url = this.baseUrl + endpoint
          if (!validateUrl(url)) {
            throw new Error('Invalid URL')
          }
          return performNetworkCall(url)
        }
      }

      function test(options: Options) {
        return options.debug ? 'debug mode' : 'production mode'
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 1
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "

    const defaultTimeout = 5000
    const maxRetries = 3

    function performNetworkCall(url: string) {
      return fetch(url)
    }

    function validateUrl(url: string): boolean {
      return url.startsWith('https://')
    }

    class NetworkService {
      private baseUrl = 'https://api.example.com'

      async request(endpoint: string) {
        const url = this.baseUrl + endpoint
        if (!validateUrl(url)) {
          throw new Error('Invalid URL')
        }
        return performNetworkCall(url)
      }
    }

    type Options = {
      debug: boolean
      timeout: number
      retries: number
    }

    function test(options: Options) {
      return options.debug ? 'debug mode' : 'production mode'
    }"
  `)
})

test('type not directly above usage with single variable in between', async () => {
  const { result } = await invalid(
    dedent`
      type Options = {
        debug: boolean
        timeout: number
        retries: number
      }
      
      const defaultTimeout = 5000
      
      function test(options: Options) {
        return options.debug ? 'debug mode' : 'production mode'
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 1
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "

    const defaultTimeout = 5000

    type Options = {
      debug: boolean
      timeout: number
      retries: number
    }

    function test(options: Options) {
      return options.debug ? 'debug mode' : 'production mode'
    }"
  `)
})

test('correctly checks types in nested functions', async () => {
  await valid(
    dedent`
      export function exhaustiveMatchObjUnion<
        T extends Record<string, unknown>,
        D extends keyof T,
        K extends T[D] & string,
      >(obj: T, key: D) {
        type Pattern<R> = {
          [P in K]: ((props: Extract<T, Record<D, P>>) => R) | '_never';
        };

        function withLazy<R>(pattern: Pattern<R>): R {
          // ...
        }

        return { with: withLazy };
      }
    `,
  )
})

test('test case', async () => {
  const { result } = await invalid(dedent`
    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }
    
    export type ProduceRecipe<T> = (draft: T) => void | undefined | T;
  `)
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }

    "
  `)
})

test('test case 2', async () => {
  const { result } = await invalid(
    dedent`
      /** Creates a reducer using immer produce function. For more details visit
       * {@link https://immerjs.github.io/immer/docs/introduction} */
      export function produceReducer<T, P>(
        reducer: (state: T, payload: P) => void | undefined | T,
      ) {
        return produce(reducer) as (state: T, payload: P) => T;
      }

      export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
        return produce(initial, recipe);
      }

      export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

      export function replaceDraftArrayItem<T>(
        draftArray: T[] | null | undefined,
        getItemToReplace: (item: T) => boolean,
        replacement: T,
      ) {
        if (!draftArray) return;

        const itemIndex = draftArray.findIndex(getItemToReplace);

        if (itemIndex !== -1) draftArray[itemIndex] = replacement;
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 13
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "/** Creates a reducer using immer produce function. For more details visit
     * {@link https://immerjs.github.io/immer/docs/introduction} */
    export function produceReducer<T, P>(
      reducer: (state: T, payload: P) => void | undefined | T,
    ) {
      return produce(reducer) as (state: T, payload: P) => T;
    }

    export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }



    export function replaceDraftArrayItem<T>(
      draftArray: T[] | null | undefined,
      getItemToReplace: (item: T) => boolean,
      replacement: T,
    ) {
      if (!draftArray) return;

      const itemIndex = draftArray.findIndex(getItemToReplace);

      if (itemIndex !== -1) draftArray[itemIndex] = replacement;
    }"
  `)
})

test('real file with imports', async () => {
  const { result } = await invalid(
    dedent`
      import { isFunction } from '@utils/assertions';
      import { produce } from 'immer';
      
      export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
        return produce(initial, recipe);
      }
      
      export type ProduceRecipe<T> = (draft: T) => void | undefined | T;
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 8
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "import { isFunction } from '@utils/assertions';
    import { produce } from 'immer';

    export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }

    "
  `)
})

// Tests for checkOnly option
test('checkOnly function - ignores types used outside function arguments', async () => {
  await valid({
    code: dedent`
      const data: UserData = { name: 'John' }
      
      type UserData = { name: string }
      
      function processUser(user: string) {
        return user.toUpperCase()
      }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
})

test('checkOnly FC - ignores types used outside FC props', async () => {
  await valid({
    code: dedent`
      const config: Config = { debug: true }

      type Config = { debug: boolean }

      const Button: React.FC<{ label: string }> = ({ label }) => {
        return <button>{label}</button>
      }
    `,
    options: [{ checkOnly: ['FC'] }],
  })
})

test('checkOnly function - catches type used in function argument', async () => {
  const { result } = await invalid({
    code: dedent`
      function processUser(data: UserData) {
        return data.name
      }
      
      type UserData = { name: string }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type UserData = { name: string }

    function processUser(data: UserData) {
      return data.name
    }

    "
  `)
})

test('checkOnly FC - catches type used in FC props', async () => {
  const { result } = await invalid({
    code: dedent`
      const Button: React.FC<ButtonProps> = ({ label }) => {
        return <button>{label}</button>
      }
      
      type ButtonProps = { label: string }
    `,
    options: [{ checkOnly: ['FC'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type ButtonProps = { label: string }

    const Button: React.FC<ButtonProps> = ({ label }) => {
      return <button>{label}</button>
    }

    "
  `)
})

test('checkOnly FC - catches type used in FC with import alias', async () => {
  const { result } = await invalid({
    code: dedent`
      import { FC } from 'react'
      
      const Header: FC<HeaderProps> = ({ text }) => {
        return <h1>{text}</h1>
      }
      
      type HeaderProps = { text: string }
    `,
    options: [{ checkOnly: ['FC'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 7
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "import { FC } from 'react'

    type HeaderProps = { text: string }

    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }

    "
  `)
})

test('checkOnly function - ignores FC props', async () => {
  await valid({
    code: dedent`
      const Button: React.FC<ButtonProps> = ({ label }) => {
        return <button>{label}</button>
      }
      
      type ButtonProps = { label: string }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
})

test('checkOnly FC - ignores function arguments', async () => {
  await valid({
    code: dedent`
      function processUser(data: UserData) {
        return data.name
      }
      
      type UserData = { name: string }
    `,
    options: [{ checkOnly: ['FC'] }],
  })
})

test('checkOnly both function and FC - catches both contexts', async () => {
  const { result } = await invalid({
    code: dedent`
      function processUser(data: UserData) {
        return data.name
      }
      
      const Button: React.FC<ButtonProps> = ({ label }) => {
        return <button>{label}</button>
      }
      
      type UserData = { name: string }
      type ButtonProps = { label: string }
    `,
    options: [{ checkOnly: ['function-args', 'FC'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 9
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type UserData = { name: string }

    function processUser(data: UserData) {
      return data.name
    }

    type ButtonProps = { label: string }

    const Button: React.FC<ButtonProps> = ({ label }) => {
      return <button>{label}</button>
    }


    "
  `)
})

test('checkOnly function - allows arrow functions', async () => {
  await valid({
    code: dedent`
      const handler: Handler = (id) => {
        console.log(id)
      }
      
      type Handler = (id: string) => void
      
      const process = (data: UserData) => data.name
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
})

test('checkOnly function - catches arrow function arguments', async () => {
  const { result } = await invalid({
    code: dedent`
      const process = (data: UserData) => data.name
      
      type UserData = { name: string }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 3
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type UserData = { name: string }

    const process = (data: UserData) => data.name

    "
  `)
})

test('checkOnly function - ignores return types when not in arguments', async () => {
  await valid({
    code: dedent`
      function createUser(): UserData {
        return { name: 'John' }
      }
      
      type UserData = { name: string }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
})

test('considers other usages as valid', async () => {
  await valid({
    code: dedent`
      export interface ReactNodeViewRendererOptions extends NodeViewRendererOptions {
        as?: string;
        className?: string;
        attrs?: Record<string, string>;
      }

      class ReactNodeView extends NodeView<
        React.FunctionComponent,
        Editor,
        ReactNodeViewRendererOptions
      > {
        renderer: ReactRenderer<__LEGIT_ANY__, __LEGIT_ANY__> | null = null;

        contentDOMElement: HTMLElement | null | undefined;

        mount() {
        }

        get dom() {

        }

        get contentDOM() {
          if (this.node.isLeaf) return null;

          return this.contentDOMElement ?? null;
        }

        update(node: ProseMirrorNode, decorations: DecorationWithType[]) {

        }

        selectNode() {
          this.renderer?.updateProps({ selected: true });
        }

        deselectNode() {
          this.renderer?.updateProps({ selected: false });
        }

        destroy() {
          this.renderer?.destroy();
          this.contentDOMElement = null;
        }
      }

      export function ReactNodeViewRenderer(
        component: FC<__LEGIT_ANY__>,
        options?: Partial<ReactNodeViewRendererOptions>,
      ): NodeViewRenderer {
        return (props) => {
        };
      }

      function capitalizeFirstChar(string: string): string {
        return string.charAt(0).toUpperCase() + string.substring(1);
      }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
})

test('checkOnly function-args ignores return types', async () => {
  await valid({
    code: dedent`
      type Handler = () => void
      
      function createHandler(): Handler {
        return () => console.log('hello')
      }
      
      function useHandler(h: Handler) {
        h()
      }
    `,
    options: [{ checkOnly: ['function-args'] }],
  })
})

test('valid case', async () => {
  await valid({
    code: dedent`
      type Arg2 = number
      type Arg1 = string
      
      function test(arg1: Arg1, arg2: Arg2) {
        return arg1 + arg2
      }
    `,
    options: [{ checkOnly: ['FC', 'function-args'] }],
  })
})

test('keep comments on functions', async () => {
  const { result } = await invalid(
    dedent`
      type Arg1 = number
      
      const test = 'ok'
      
      /** @type {Arg1} */
      function test(arg1: Arg1) {
        return arg1
      }
    `,
  )
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 1
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "

    const test = 'ok'

    type Arg1 = number

    /** @type {Arg1} */
    function test(arg1: Arg1) {
      return arg1
    }"
  `)
})

// Tests for generic-args-at-fn-calls option
test('checkOnly generic-args-at-fn-calls - ignores types used in function arguments', async () => {
  await valid({
    code: dedent`
      function processUser(data: UserData) {
        return data.name
      }
      
      type UserData = { name: string }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
})

test('checkOnly generic-args-at-fn-calls - ignores types used in FC props', async () => {
  await valid({
    code: dedent`
      const Button: React.FC<ButtonProps> = ({ label }) => {
        return <button>{label}</button>
      }
      
      type ButtonProps = { label: string }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
})

test('checkOnly generic-args-at-fn-calls - ignores variable declarations', async () => {
  await valid({
    code: dedent`
      const config: Config = { debug: true }
      
      type Config = { debug: boolean }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
})

test('checkOnly generic-args-at-fn-calls - catches type used in function call generic', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = processData<DataType>({ value: 'test' })
      
      type DataType = { value: string }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 3
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type DataType = { value: string }

    const result = processData<DataType>({ value: 'test' })

    "
  `)
})

test('checkOnly generic-args-at-fn-calls - catches type used in method call generic', async () => {
  const { result } = await invalid({
    code: dedent`
      const items = collection.filter<Item>(item => item.active)
      
      type Item = { active: boolean }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 3
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type Item = { active: boolean }

    const items = collection.filter<Item>(item => item.active)

    "
  `)
})

test('checkOnly generic-args-at-fn-calls - catches type in chained method call generic', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = data
        .map<ProcessedItem>(item => ({ ...item, processed: true }))
        .filter(item => item.active)
      
      type ProcessedItem = { processed: boolean; active: boolean }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 5
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type ProcessedItem = { processed: boolean; active: boolean }

    const result = data
      .map<ProcessedItem>(item => ({ ...item, processed: true }))
      .filter(item => item.active)

    "
  `)
})

test('checkOnly generic-args-at-fn-calls - catches type in constructor call generic', async () => {
  const { result } = await invalid({
    code: dedent`
      const instance = new Collection<EntityType>()
      
      type EntityType = { id: string }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 3
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type EntityType = { id: string }

    const instance = new Collection<EntityType>()

    "
  `)
})

test('checkOnly generic-args-at-fn-calls - multiple generic args', async () => {
  const { result } = await invalid({
    code: dedent`
      const result = transform<InputType, OutputType>(data, processor)
      
      type InputType = { raw: string }
      type OutputType = { processed: string }
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 3
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "type InputType = { raw: string }

    type OutputType = { processed: string }

    const result = transform<InputType, OutputType>(data, processor)


    "
  `)
})

test('checkOnly generic-args-at-fn-calls - type correctly above generic function call usage', async () => {
  await valid({
    code: dedent`
      type ApiResponse = { data: string }
      
      const result = fetchData<ApiResponse>('/api/users')
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
})

test('reproduce bug', async () => {
  const { result } = await invalid({
    code: dedent`
      export type ConfigA = {
        id: string;
        label: string | null;
        enabled: boolean;
        value: unknown;
      };
      
      export type ConfigB = {
        title: string;
        description: string | null;
      };
      
      export type ConfigC = {
        name: string | null;
        reference_id: string;
        items: string[];
      };
      
      export type ConfigD = {
        title: string;
        visible: boolean;
        content: string;
      };
      
      export type ConfigE = {
        name: string;
        description: string | null;
        active: boolean;
      };
      
      const configASchema = rc_obj_builder<ConfigA>()({
        id: rc_string,
        label: rc_string.orNull(),
        enabled: rc_boolean,
        value: rc_unknown,
      });
      
      const configBSchema = rc_obj_builder<ConfigB>()({
        title: rc_string,
        description: rc_string.orNull(),
      });
      
      const configCSchema = rc_obj_builder<ConfigC>()({
        name: rc_string.orNull(),
        reference_id: rc_string,
        items: rc_array(rc_string),
      });
      
      const configDSchema = rc_obj_builder<ConfigD>()({
        title: rc_string,
        visible: rc_boolean,
        content: rc_string,
      });
      
      const configESchema = rc_obj_builder<ConfigE>()({
        name: rc_string,
        description: rc_string.orNull(),
        active: rc_boolean,
      });
      
      export type BlockType =
        | { id: string; type: 'typeA'; config: ConfigA }
        | { id: string; type: 'typeB'; config: ConfigB }
        | { id: string; type: 'typeC'; config: ConfigC }
        | { id: string; type: 'typeD'; config: ConfigD }
        | { id: string; type: 'typeE'; config: ConfigE };
      
      const blockTypeSchema = rc_discriminated_union_builder<BlockType, 'type'>('type')({
        typeA: { id: rc_string, config: configASchema },
        typeB: { id: rc_string, config: configBSchema },
        typeC: { id: rc_string, config: configCSchema },
        typeD: { id: rc_string, config: configDSchema },
        typeE: { id: rc_string, config: configESchema },
      });
      
      export type ApiConfig = {
        type: 'mainType';
        description: string | null;
        title: string | null;
        blocks: BlockType[];
        settings: {
          enabled: boolean;
          value: string | null;
        };
      };
      
      export const apiConfigSchema = rc_obj_builder<ApiConfig>()({
        type: rc_literals('mainType'),
        title: rc_string.orNull(),
        description: rc_string.orNull(),
        blocks: rc_array(blockTypeSchema),
        settings: {
          enabled: rc_boolean,
          value: rc_string.orNull(),
        },
      });
    `,
    options: [{ checkOnly: ['generic-args-at-fn-calls'] }],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 1
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "









    export type ConfigA = {
      id: string;
      label: string | null;
      enabled: boolean;
      value: unknown;
    };

    const configASchema = rc_obj_builder<ConfigA>()({
      id: rc_string,
      label: rc_string.orNull(),
      enabled: rc_boolean,
      value: rc_unknown,
    });

    export type ConfigB = {
      title: string;
      description: string | null;
    };

    const configBSchema = rc_obj_builder<ConfigB>()({
      title: rc_string,
      description: rc_string.orNull(),
    });

    export type ConfigC = {
      name: string | null;
      reference_id: string;
      items: string[];
    };

    const configCSchema = rc_obj_builder<ConfigC>()({
      name: rc_string.orNull(),
      reference_id: rc_string,
      items: rc_array(rc_string),
    });

    export type ConfigD = {
      title: string;
      visible: boolean;
      content: string;
    };

    const configDSchema = rc_obj_builder<ConfigD>()({
      title: rc_string,
      visible: rc_boolean,
      content: rc_string,
    });

    export type ConfigE = {
      name: string;
      description: string | null;
      active: boolean;
    };

    const configESchema = rc_obj_builder<ConfigE>()({
      name: rc_string,
      description: rc_string.orNull(),
      active: rc_boolean,
    });

    export type BlockType =
      | { id: string; type: 'typeA'; config: ConfigA }
      | { id: string; type: 'typeB'; config: ConfigB }
      | { id: string; type: 'typeC'; config: ConfigC }
      | { id: string; type: 'typeD'; config: ConfigD }
      | { id: string; type: 'typeE'; config: ConfigE };

    const blockTypeSchema = rc_discriminated_union_builder<BlockType, 'type'>('type')({
      typeA: { id: rc_string, config: configASchema },
      typeB: { id: rc_string, config: configBSchema },
      typeC: { id: rc_string, config: configCSchema },
      typeD: { id: rc_string, config: configDSchema },
      typeE: { id: rc_string, config: configESchema },
    });

    export type ApiConfig = {
      type: 'mainType';
      description: string | null;
      title: string | null;
      blocks: BlockType[];
      settings: {
        enabled: boolean;
        value: string | null;
      };
    };

    export const apiConfigSchema = rc_obj_builder<ApiConfig>()({
      type: rc_literals('mainType'),
      title: rc_string.orNull(),
      description: rc_string.orNull(),
      blocks: rc_array(blockTypeSchema),
      settings: {
        enabled: rc_boolean,
        value: rc_string.orNull(),
      },
    });"
  `)
})

test('reproduce bug with all options', async () => {
  const { result } = await invalid({
    code: dedent`
      export type ConfigA = {
        id: string;
        label: string | null;
        enabled: boolean;
        value: unknown;
      };
      
      export type ConfigB = {
        title: string;
        description: string | null;
      };
      
      export type ConfigC = {
        name: string | null;
        reference_id: string;
        items: string[];
      };
      
      export type ConfigD = {
        title: string;
        visible: boolean;
        content: string;
      };
      
      export type ConfigE = {
        name: string;
        description: string | null;
        active: boolean;
      };
      
      const configASchema = rc_obj_builder<ConfigA>()({
        id: rc_string,
        label: rc_string.orNull(),
        enabled: rc_boolean,
        value: rc_unknown,
      });
      
      const configBSchema = rc_obj_builder<ConfigB>()({
        title: rc_string,
        description: rc_string.orNull(),
      });
      
      const configCSchema = rc_obj_builder<ConfigC>()({
        name: rc_string.orNull(),
        reference_id: rc_string,
        items: rc_array(rc_string),
      });
      
      const configDSchema = rc_obj_builder<ConfigD>()({
        title: rc_string,
        visible: rc_boolean,
        content: rc_string,
      });
      
      const configESchema = rc_obj_builder<ConfigE>()({
        name: rc_string,
        description: rc_string.orNull(),
        active: rc_boolean,
      });
      
      export type BlockType =
        | { id: string; type: 'typeA'; config: ConfigA }
        | { id: string; type: 'typeB'; config: ConfigB }
        | { id: string; type: 'typeC'; config: ConfigC }
        | { id: string; type: 'typeD'; config: ConfigD }
        | { id: string; type: 'typeE'; config: ConfigE };
      
      const blockTypeSchema = rc_discriminated_union_builder<BlockType, 'type'>('type')({
        typeA: { id: rc_string, config: configASchema },
        typeB: { id: rc_string, config: configBSchema },
        typeC: { id: rc_string, config: configCSchema },
        typeD: { id: rc_string, config: configDSchema },
        typeE: { id: rc_string, config: configESchema },
      });
      
      export type ApiConfig = {
        type: 'mainType';
        description: string | null;
        title: string | null;
        blocks: BlockType[];
        settings: {
          enabled: boolean;
          value: string | null;
        };
      };
      
      export const apiConfigSchema = rc_obj_builder<ApiConfig>()({
        type: rc_literals('mainType'),
        title: rc_string.orNull(),
        description: rc_string.orNull(),
        blocks: rc_array(blockTypeSchema),
        settings: {
          enabled: rc_boolean,
          value: rc_string.orNull(),
        },
      });
    `,
    options: [
      { checkOnly: ['function-args', 'FC', 'generic-args-at-fn-calls'] },
    ],
  })
  expect(getErrorsFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'moveTypeAboveUsage'
      data: 'Type definition should be placed directly above its first usage.'
      line: 1
    "
  `)
  expect(result.output).toMatchInlineSnapshot(`
    "









    export type ConfigA = {
      id: string;
      label: string | null;
      enabled: boolean;
      value: unknown;
    };

    const configASchema = rc_obj_builder<ConfigA>()({
      id: rc_string,
      label: rc_string.orNull(),
      enabled: rc_boolean,
      value: rc_unknown,
    });

    export type ConfigB = {
      title: string;
      description: string | null;
    };

    const configBSchema = rc_obj_builder<ConfigB>()({
      title: rc_string,
      description: rc_string.orNull(),
    });

    export type ConfigC = {
      name: string | null;
      reference_id: string;
      items: string[];
    };

    const configCSchema = rc_obj_builder<ConfigC>()({
      name: rc_string.orNull(),
      reference_id: rc_string,
      items: rc_array(rc_string),
    });

    export type ConfigD = {
      title: string;
      visible: boolean;
      content: string;
    };

    const configDSchema = rc_obj_builder<ConfigD>()({
      title: rc_string,
      visible: rc_boolean,
      content: rc_string,
    });

    export type ConfigE = {
      name: string;
      description: string | null;
      active: boolean;
    };

    const configESchema = rc_obj_builder<ConfigE>()({
      name: rc_string,
      description: rc_string.orNull(),
      active: rc_boolean,
    });

    export type BlockType =
      | { id: string; type: 'typeA'; config: ConfigA }
      | { id: string; type: 'typeB'; config: ConfigB }
      | { id: string; type: 'typeC'; config: ConfigC }
      | { id: string; type: 'typeD'; config: ConfigD }
      | { id: string; type: 'typeE'; config: ConfigE };

    const blockTypeSchema = rc_discriminated_union_builder<BlockType, 'type'>('type')({
      typeA: { id: rc_string, config: configASchema },
      typeB: { id: rc_string, config: configBSchema },
      typeC: { id: rc_string, config: configCSchema },
      typeD: { id: rc_string, config: configDSchema },
      typeE: { id: rc_string, config: configESchema },
    });

    export type ApiConfig = {
      type: 'mainType';
      description: string | null;
      title: string | null;
      blocks: BlockType[];
      settings: {
        enabled: boolean;
        value: string | null;
      };
    };

    export const apiConfigSchema = rc_obj_builder<ApiConfig>()({
      type: rc_literals('mainType'),
      title: rc_string.orNull(),
      description: rc_string.orNull(),
      blocks: rc_array(blockTypeSchema),
      settings: {
        enabled: rc_boolean,
        value: rc_string.orNull(),
      },
    });"
  `)
})
