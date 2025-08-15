import { createTester } from '../../tests/utils/createTester'
import { useTypesDirectlyAboveUsage } from './use-types-directly-above-usage'

const tests = createTester(useTypesDirectlyAboveUsage, {
  defaultErrorId: 'moveTypeAboveUsage',
})

// Valid cases - no errors expected
tests.addValid(
  'type alias directly above function',
  `
    type UserData = { name: string }
    
    function processUser(data: UserData) {
      return data.name
    }
  `,
)

tests.addValid(
  'interface directly above function',
  `
    interface Config {
      debug: boolean
    }
    
    function setupApp(config: Config) {
      console.log(config.debug)
    }
  `,
)

tests.addValid(
  'props type above React component',
  `
    type ButtonProps = {
      onClick: () => void
      children: string
    }
    
    function Button(props: ButtonProps) {
      return <button onClick={props.onClick}>{props.children}</button>
    }
  `,
)

tests.addValid(
  'props type above React FC component',
  `
    type CardProps = {
      title: string
      content: string
    }
    
    const Card: React.FC<CardProps> = ({ title, content }) => {
      return <div><h2>{title}</h2><p>{content}</p></div>
    }
  `,
)

tests.addValid(
  'props type above React FC component with import alias',
  `
    import { FC } from 'react'
    
    type HeaderProps = {
      text: string
    }
    
    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }
  `,
)

tests.addValid(
  'type above arrow function',
  `
    type Handler = (id: string) => void
    
    const onClick: Handler = (id) => {
      console.log(id)
    }
  `,
)

tests.addValid(
  'type correctly above first usage when used multiple times',
  `
    type SharedType = { value: number }
    
    function processA(data: SharedType) {
      return data.value
    }
    
    function processB(input: SharedType) {
      return input.value * 2
    }
  `,
)

tests.addValid(
  'type correctly above first usage in param and return type',
  `
    type Result = { data: string }
    
    function transform(x: string): Result {
      return { data: x }
    }
    
    function create(): Result {
      return { data: 'default' }
    }
  `,
)

tests.addValid(
  'inline type annotations are allowed',
  `
    function process(data: { name: string, age: number }) {
      return data.name
    }
  `,
)

tests.addValid(
  'generic type parameters are allowed',
  `
    function map<T>(items: T[]): T[] {
      return items
    }
  `,
)

tests.addValid(
  'imported types are ignored',
  `
    import { ApiResponse } from './types'
    
    type LocalData = string
    
    function handleResponse(response: ApiResponse) {
      return response.data
    }
  `,
)

tests.addValid(
  'multiple types each above their respective functions',
  `
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

// Invalid cases - should trigger errors and provide fixes
tests.addInvalid(
  'type alias below function that uses it',
  `
    function processUser(data: UserData) {
      return data.name
    }

    type UserData = { name: string }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
      type UserData = { name: string }

      function processUser(data: UserData) {
        return data.name
      }
    `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'interface below function that uses it',
  `
    function setupApp(config: Config) {
      console.log(config.debug)
    }
    
    interface Config {
      debug: boolean
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    interface Config {
      debug: boolean
    }
    
    function setupApp(config: Config) {
      console.log(config.debug)
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'props type below React component',
  `
    function Button(props: ButtonProps) {
      return <button onClick={props.onClick}>{props.children}</button>
    }
    
    type ButtonProps = {
      onClick: () => void
      children: string
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type ButtonProps = {
      onClick: () => void
      children: string
    }
    
    function Button(props: ButtonProps) {
      return <button onClick={props.onClick}>{props.children}</button>
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'props type below React FC component',
  `
    const Card: React.FC<CardProps> = ({ title, content }) => {
      return <div><h2>{title}</h2><p>{content}</p></div>
    }
    
    type CardProps = {
      title: string
      content: string
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type CardProps = {
      title: string
      content: string
    }
    
    const Card: React.FC<CardProps> = ({ title, content }) => {
      return <div><h2>{title}</h2><p>{content}</p></div>
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'props type below FC component with import alias',
  `
    import { FC } from 'react'
    
    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }
    
    type HeaderProps = {
      text: string
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    import { FC } from 'react'
    
    type HeaderProps = {
      text: string
    }
    
    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'type below arrow function',
  `
    const onClick: Handler = (id) => {
      console.log(id)
    }
    
    type Handler = (id: string) => void
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Handler = (id: string) => void
    
    const onClick: Handler = (id) => {
      console.log(id)
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'type below function with return type annotation',
  `
    function createUser(): UserType {
      return { name: 'John' }
    }
    
    type UserType = { name: string }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type UserType = { name: string }
    
    function createUser(): UserType {
      return { name: 'John' }
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'multiple misplaced types each used by single function',
  `
    function processA(data: TypeA) {
      return data.valueA
    }
    
    function processB(data: TypeB) {
      return data.valueB
    }
    
    type TypeA = { valueA: string }
    type TypeB = { valueB: number }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type TypeA = { valueA: string }
    
    function processA(data: TypeA) {
      return data.valueA
    }
    
    function processB(data: TypeB) {
      return data.valueB
    }
    
    type TypeB = { valueB: number }
  `,
  },
)

tests.addInvalid(
  'type between functions where first function uses it',
  `
    function processData(input: DataType) {
      return input.value
    }
    
    type DataType = { value: string }
    
    function otherFunction() {
      return 'hello'
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type DataType = { value: string }
    
    function processData(input: DataType) {
      return input.value
    }
    

    function otherFunction() {
      return 'hello'
    }
  `,
  },
)

tests.addInvalid(
  'interface used in function parameter and return type by same function only',
  `
    function transform(input: Data): Data {
      return { ...input, processed: true }
    }
    
    interface Data {
      processed?: boolean
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    interface Data {
      processed?: boolean
    }
    
    function transform(input: Data): Data {
      return { ...input, processed: true }
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'type used in generic constraint',
  `
    function filter<T extends Filterable>(items: T[]): T[] {
      return items.filter(item => item.isActive)
    }
    
    type Filterable = { isActive: boolean }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Filterable = { isActive: boolean }
    
    function filter<T extends Filterable>(items: T[]): T[] {
      return items.filter(item => item.isActive)
    }
  `,
    appendToOutput: '\n\n',
  },
)

// Edge cases that should be handled correctly
tests.addValid(
  'type used in variable declaration with function expression',
  `
    type Handler = () => void
    
    const myHandler: Handler = function() {
      console.log('handled')
    }
  `,
)

// Variable declaration tests
tests.addInvalid(
  'variable declaration with type below - options example',
  `
    const options: Options = {
      debug: true,
      timeout: 5000
    }
    
    type Options = {
      debug: boolean
      timeout: number
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Options = {
      debug: boolean
      timeout: number
    }
    
    const options: Options = {
      debug: true,
      timeout: 5000
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addValid(
  'type correctly above variable declaration',
  `
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

tests.addValid(
  'multiple variables using same type - should be above first usage',
  `
    type Settings = {
      theme: string
      fontSize: number
    }
    
    const userSettings: Settings = { theme: 'dark', fontSize: 14 }
    const defaultSettings: Settings = { theme: 'light', fontSize: 12 }
  `,
)

tests.addValid(
  'type correctly above first usage when used in nested and direct contexts',
  `
    type UserInfo = { name: string }
    
    function processUser(data: { user: UserInfo }) {
      return data.user.name
    }
    
    function validateUser(info: UserInfo) {
      return info.name.length > 0
    }
  `,
)

tests.addValid(
  'props type correctly above first FC component when used multiple times',
  `
    type SharedProps = { label: string }
    
    const Button: React.FC<SharedProps> = ({ label }) => {
      return <button>{label}</button>
    }
    
    const Link: React.FC<SharedProps> = ({ label }) => {
      return <a href="#">{label}</a>
    }
  `,
)

tests.addValid(
  'props type correctly above first usage in mixed components',
  `
    type CommonProps = { text: string }
    
    function Button(props: CommonProps) {
      return <button>{props.text}</button>
    }
    
    const Card: React.FC<CommonProps> = ({ text }) => {
      return <div>{text}</div>
    }
  `,
)

tests.addInvalid(
  'type with comments should preserve formatting',
  `
    function processData(input: MyType) {
      return input.value
    }
    
    // This is an important type
    type MyType = {
      // The main value
      value: string
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    // This is an important type
    type MyType = {
      // The main value
      value: string
    }
    
    function processData(input: MyType) {
      return input.value
    }
  `,
    appendToOutput: '\n\n',
  },
)

// Invalid cases for types used by multiple functions - should move above first usage
tests.addInvalid(
  'shared type should move above first function usage',
  `
    function processA(data: SharedType) {
      return data.value
    }
    
    function processB(input: SharedType) {
      return input.value * 2
    }
    
    type SharedType = { value: number }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type SharedType = { value: number }
    
    function processA(data: SharedType) {
      return data.value
    }
    
    function processB(input: SharedType) {
      return input.value * 2
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'props type should move above first FC component',
  `
    const Button: React.FC<SharedProps> = ({ label }) => {
      return <button>{label}</button>
    }
    
    const Link: React.FC<SharedProps> = ({ label }) => {
      return <a href="#">{label}</a>
    }
    
    type SharedProps = { label: string }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type SharedProps = { label: string }
    
    const Button: React.FC<SharedProps> = ({ label }) => {
      return <button>{label}</button>
    }
    
    const Link: React.FC<SharedProps> = ({ label }) => {
      return <a href="#">{label}</a>
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'shared props should move above first component (mixed function and FC)',
  `
    function Button(props: CommonProps) {
      return <button>{props.text}</button>
    }
    
    const Card: React.FC<CommonProps> = ({ text }) => {
      return <div>{text}</div>
    }
    
    type CommonProps = { text: string }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type CommonProps = { text: string }
    
    function Button(props: CommonProps) {
      return <button>{props.text}</button>
    }
    
    const Card: React.FC<CommonProps> = ({ text }) => {
      return <div>{text}</div>
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'type used in param and return type should move above first usage',
  `
    function transform(x: string): Result {
      return { data: x }
    }
    
    function create(): Result {
      return { data: 'default' }
    }
    
    type Result = { data: string }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Result = { data: string }
    
    function transform(x: string): Result {
      return { data: x }
    }
    
    function create(): Result {
      return { data: 'default' }
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'variable declaration with type below - config example',
  `
    const config: Config = {
      apiUrl: 'https://api.example.com',
      retries: 3
    }
    
    type Config = {
      apiUrl: string
      retries: number
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Config = {
      apiUrl: string
      retries: number
    }
    
    const config: Config = {
      apiUrl: 'https://api.example.com',
      retries: 3
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'multiple variables with shared type - should move above first variable',
  `
    const userSettings: Settings = { theme: 'dark', fontSize: 14 }
    const defaultSettings: Settings = { theme: 'light', fontSize: 12 }
    
    type Settings = {
      theme: string
      fontSize: number
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Settings = {
      theme: string
      fontSize: number
    }
    
    const userSettings: Settings = { theme: 'dark', fontSize: 14 }
    const defaultSettings: Settings = { theme: 'light', fontSize: 12 }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'mixed function and variable usage - should move above first occurrence',
  `
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
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type AppConfig = {
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
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'variable comes first - type should move above variable not function',
  `
    const state: State = { loading: false }
    
    function updateState(newState: State) {
      return { ...state, ...newState }
    }
    
    type State = {
      loading: boolean
    }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type State = {
      loading: boolean
    }
    
    const state: State = { loading: false }
    
    function updateState(newState: State) {
      return { ...state, ...newState }
    }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'type below variable - now always checked',
  `
    const options: Options = { debug: true }
    
    type Options = { debug: boolean }
  `,
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    type Options = { debug: boolean }
    
    const options: Options = { debug: true }
  `,
    appendToOutput: '\n\n',
  },
)

tests.addInvalid(
  'type not directly above usage with code in between',
  `
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
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
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
    }
  `,
    prependToOutput: `\n`,
  },
)

tests.addInvalid(
  'type not directly above usage with single variable in between',
  `
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
  [{ messageId: 'moveTypeAboveUsage' }],
  {
    output: `
    const defaultTimeout = 5000

    type Options = {
      debug: boolean
      timeout: number
      retries: number
    }
    
    function test(options: Options) {
      return options.debug ? 'debug mode' : 'production mode'
    }
  `,
    prependToOutput: `\n`,
  },
)

tests.addValid(
  'correctly checks types in nested functions',
  `
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

tests.run()
