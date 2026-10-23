mod common;

use common::{invalid, valid, Case};
use indoc::indoc;
use insta::assert_snapshot;
use serde_json::json;

const RULE: &str = "use-types-directly-above-usage";

#[test]
fn type_alias_directly_above_function() {
    valid(
        RULE,
        indoc! {r#"
        type UserData = { name: string }

        function processUser(data: UserData) {
          return data.name
        }
    "#},
    );
}

#[test]
fn interface_directly_above_function() {
    valid(
        RULE,
        indoc! {r#"
        interface Config {
          debug: boolean
        }

        function setupApp(config: Config) {
          console.log(config.debug)
        }
    "#},
    );
}

#[test]
fn props_type_above_react_component() {
    valid(
        RULE,
        indoc! {r#"
        type ButtonProps = {
          onClick: () => void
          children: string
        }

        function Button(props: ButtonProps) {
          return <button onClick={props.onClick}>{props.children}</button>
        }
    "#},
    );
}

#[test]
fn props_type_above_react_fc_component() {
    valid(
        RULE,
        indoc! {r#"
        type CardProps = {
          title: string
          content: string
        }

        const Card: React.FC<CardProps> = ({ title, content }) => {
          return <div><h2>{title}</h2><p>{content}</p></div>
        }
    "#},
    );
}

#[test]
fn props_type_above_react_fc_component_with_import_alias() {
    valid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        type HeaderProps = {
          text: string
        }

        const Header: FC<HeaderProps> = ({ text }) => {
          return <h1>{text}</h1>
        }
    "#},
    );
}

#[test]
fn type_above_arrow_function() {
    valid(
        RULE,
        indoc! {r#"
        type Handler = (id: string) => void

        const onClick: Handler = (id) => {
          console.log(id)
        }
    "#},
    );
}

#[test]
fn type_correctly_above_first_usage_when_used_multiple_times() {
    valid(
        RULE,
        indoc! {r#"
        type SharedType = { value: number }

        function processA(data: SharedType) {
          return data.value
        }

        function processB(input: SharedType) {
          return input.value * 2
        }
    "#},
    );
}

#[test]
fn type_correctly_above_first_usage_in_param_and_return_type() {
    valid(
        RULE,
        indoc! {r#"
        type Result = { data: string }

        function transform(x: string): Result {
          return { data: x }
        }

        function create(): Result {
          return { data: 'default' }
        }
    "#},
    );
}

#[test]
fn inline_type_annotations_are_allowed() {
    valid(
        RULE,
        indoc! {r#"
        function process(data: { name: string, age: number }) {
          return data.name
        }
    "#},
    );
}

#[test]
fn generic_type_parameters_are_allowed() {
    valid(
        RULE,
        indoc! {r#"
        function map<T>(items: T[]): T[] {
          return items
        }
    "#},
    );
}

#[test]
fn imported_types_are_ignored() {
    valid(
        RULE,
        indoc! {r#"
        import { ApiResponse } from './types'

        type LocalData = string

        function handleResponse(response: ApiResponse) {
          return response.data
        }
    "#},
    );
}

#[test]
fn multiple_types_each_above_their_respective_functions() {
    valid(
        RULE,
        indoc! {r#"
        type UserConfig = { theme: string }

        function setupUser(config: UserConfig) {
          return config.theme
        }

        type DatabaseConfig = { host: string }

        function connectDb(config: DatabaseConfig) {
          return config.host
        }
    "#},
    );
}

#[test]
fn type_alias_below_function_that_uses_it() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function processUser(data: UserData) {
          return data.name
        }

        type UserData = { name: string }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type UserData = { name: string }

    function processUser(data: UserData) {
      return data.name
    }

    "#);
}

#[test]
fn interface_below_function_that_uses_it() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function setupApp(config: Config) {
          console.log(config.debug)
        }

        interface Config {
          debug: boolean
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    interface Config {
      debug: boolean
    }

    function setupApp(config: Config) {
      console.log(config.debug)
    }

    "#);
}

#[test]
fn props_type_below_react_component() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function Button(props: ButtonProps) {
          return <button onClick={props.onClick}>{props.children}</button>
        }

        type ButtonProps = {
          onClick: () => void
          children: string
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type ButtonProps = {
      onClick: () => void
      children: string
    }

    function Button(props: ButtonProps) {
      return <button onClick={props.onClick}>{props.children}</button>
    }

    "#);
}

#[test]
fn props_type_below_react_fc_component() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const Card: React.FC<CardProps> = ({ title, content }) => {
          return <div><h2>{title}</h2><p>{content}</p></div>
        }

        type CardProps = {
          title: string
          content: string
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type CardProps = {
      title: string
      content: string
    }

    const Card: React.FC<CardProps> = ({ title, content }) => {
      return <div><h2>{title}</h2><p>{content}</p></div>
    }

    "#);
}

#[test]
fn props_type_below_fc_component_with_import_alias() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { FC } from 'react'

        const Header: FC<HeaderProps> = ({ text }) => {
          return <h1>{text}</h1>
        }

        type HeaderProps = { text: string }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 7 }
    "#);
    assert_snapshot!(result.output, @r#"
    import { FC } from 'react'

    type HeaderProps = { text: string }

    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }

    "#);
}

#[test]
fn type_below_arrow_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const onClick: Handler = (id) => {
          console.log(id)
        }

        type Handler = (id: string) => void
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Handler = (id: string) => void

    const onClick: Handler = (id) => {
      console.log(id)
    }

    "#);
}

#[test]
fn type_below_function_with_return_type_annotation() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function createUser(): UserType {
          return { name: 'John' }
        }

        type UserType = { name: string }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type UserType = { name: string }

    function createUser(): UserType {
      return { name: 'John' }
    }

    "#);
}

#[test]
fn multiple_misplaced_types_each_used_by_single_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function processA(data: TypeA) {
          return data.valueA
        }

        function processB(data: TypeB) {
          return data.valueB
        }

        type TypeA = { valueA: string }
        type TypeB = { valueB: number }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    type TypeA = { valueA: string }

    function processA(data: TypeA) {
      return data.valueA
    }

    type TypeB = { valueB: number }

    function processB(data: TypeB) {
      return data.valueB
    }


    "#);
}

#[test]
fn type_between_functions_where_first_function_uses_it() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function processData(input: DataType) {
          return input.value
        }

        type DataType = { value: string }

        function otherFunction() {
          return 'hello'
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type DataType = { value: string }

    function processData(input: DataType) {
      return input.value
    }



    function otherFunction() {
      return 'hello'
    }
    "#);
}

#[test]
fn interface_used_in_function_parameter_and_return_type_by_same_function_only() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function transform(input: Data): Data {
          return { ...input, processed: true }
        }

        interface Data {
          processed?: boolean
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    interface Data {
      processed?: boolean
    }

    function transform(input: Data): Data {
      return { ...input, processed: true }
    }

    "#);
}

#[test]
fn type_used_in_generic_constraint() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function filter<T extends Filterable>(items: T[]): T[] {
          return items.filter(item => item.isActive)
        }

        type Filterable = { isActive: boolean }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Filterable = { isActive: boolean }

    function filter<T extends Filterable>(items: T[]): T[] {
      return items.filter(item => item.isActive)
    }

    "#);
}

#[test]
fn type_used_in_variable_declaration_with_function_expression() {
    valid(
        RULE,
        indoc! {r#"
        type Handler = () => void

        const myHandler: Handler = function() {
          console.log('handled')
        }
    "#},
    );
}

#[test]
fn variable_declaration_with_type_below_options_example() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const options: Options = {
          debug: true,
          timeout: 5000
        }

        type Options = {
          debug: boolean
          timeout: number
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 6 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Options = {
      debug: boolean
      timeout: number
    }

    const options: Options = {
      debug: true,
      timeout: 5000
    }

    "#);
}

#[test]
fn type_correctly_above_variable_declaration() {
    valid(
        RULE,
        indoc! {r#"
        type Config = {
          apiUrl: string
          retries: number
        }

        const config: Config = {
          apiUrl: 'https://api.example.com',
          retries: 3
        }
    "#},
    );
}

#[test]
fn multiple_variables_using_same_type_should_be_above_first_usage() {
    valid(
        RULE,
        indoc! {r#"
        type Settings = {
          theme: string
          fontSize: number
        }

        const userSettings: Settings = { theme: 'dark', fontSize: 14 }
        const defaultSettings: Settings = { theme: 'light', fontSize: 12 }
    "#},
    );
}

#[test]
fn type_correctly_above_first_usage_when_used_in_nested_and_direct_contexts() {
    valid(
        RULE,
        indoc! {r#"
        type UserInfo = { name: string }

        function processUser(data: { user: UserInfo }) {
          return data.user.name
        }

        function validateUser(info: UserInfo) {
          return info.name.length > 0
        }
    "#},
    );
}

#[test]
fn props_type_correctly_above_first_fc_component_when_used_multiple_times() {
    valid(
        RULE,
        indoc! {r##"
        type SharedProps = { label: string }

        const Button: React.FC<SharedProps> = ({ label }) => {
          return <button>{label}</button>
        }

        const Link: React.FC<SharedProps> = ({ label }) => {
          return <a href="#">{label}</a>
        }
    "##},
    );
}

#[test]
fn props_type_correctly_above_first_usage_in_mixed_components() {
    valid(
        RULE,
        indoc! {r#"
        type CommonProps = { text: string }

        function Button(props: CommonProps) {
          return <button>{props.text}</button>
        }

        const Card: React.FC<CommonProps> = ({ text }) => {
          return <div>{text}</div>
        }
    "#},
    );
}

#[test]
fn type_with_comments_should_preserve_formatting() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function processData(input: MyType) {
          return input.value
        }

        // This is an important type
        type MyType = {
          // The main value
          value: string
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 6 }
    "#);
    assert_snapshot!(result.output, @r#"
    // This is an important type
    type MyType = {
      // The main value
      value: string
    }

    function processData(input: MyType) {
      return input.value
    }

    "#);
}

#[test]
fn shared_type_should_move_above_first_function_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function processA(data: SharedType) {
          return data.value
        }

        function processB(input: SharedType) {
          return input.value * 2
        }

        type SharedType = { value: number }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    type SharedType = { value: number }

    function processA(data: SharedType) {
      return data.value
    }

    function processB(input: SharedType) {
      return input.value * 2
    }

    "#);
}

#[test]
fn props_type_should_move_above_first_fc_component() {
    let result = invalid(
        RULE,
        indoc! {r##"
        const Button: React.FC<SharedProps> = ({ label }) => {
          return <button>{label}</button>
        }

        const Link: React.FC<SharedProps> = ({ label }) => {
          return <a href="#">{label}</a>
        }

        type SharedProps = { label: string }
    "##},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r##"
    type SharedProps = { label: string }

    const Button: React.FC<SharedProps> = ({ label }) => {
      return <button>{label}</button>
    }

    const Link: React.FC<SharedProps> = ({ label }) => {
      return <a href="#">{label}</a>
    }

    "##);
}

#[test]
fn shared_props_should_move_above_first_component_mixed_function_and_fc() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function Button(props: CommonProps) {
          return <button>{props.text}</button>
        }

        const Card: React.FC<CommonProps> = ({ text }) => {
          return <div>{text}</div>
        }

        type CommonProps = { text: string }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    type CommonProps = { text: string }

    function Button(props: CommonProps) {
      return <button>{props.text}</button>
    }

    const Card: React.FC<CommonProps> = ({ text }) => {
      return <div>{text}</div>
    }

    "#);
}

#[test]
fn type_used_in_param_and_return_type_should_move_above_first_usage() {
    let result = invalid(
        RULE,
        indoc! {r#"
        function transform(x: string): Result {
          return { data: x }
        }

        function create(): Result {
          return { data: 'default' }
        }

        type Result = { data: string }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Result = { data: string }

    function transform(x: string): Result {
      return { data: x }
    }

    function create(): Result {
      return { data: 'default' }
    }

    "#);
}

#[test]
fn variable_declaration_with_type_below_config_example() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const config: Config = {
          apiUrl: 'https://api.example.com',
          retries: 3
        }

        type Config = {
          apiUrl: string
          retries: number
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 6 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Config = {
      apiUrl: string
      retries: number
    }

    const config: Config = {
      apiUrl: 'https://api.example.com',
      retries: 3
    }

    "#);
}

#[test]
fn multiple_variables_with_shared_type_should_move_above_first_variable() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const userSettings: Settings = { theme: 'dark', fontSize: 14 }
        const defaultSettings: Settings = { theme: 'light', fontSize: 12 }

        type Settings = {
          theme: string
          fontSize: number
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 4 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Settings = {
      theme: string
      fontSize: number
    }

    const userSettings: Settings = { theme: 'dark', fontSize: 14 }
    const defaultSettings: Settings = { theme: 'light', fontSize: 12 }

    "#);
}

#[test]
fn mixed_function_and_variable_usage_should_move_above_first_occurrence() {
    let result = invalid(
        RULE,
        indoc! {r#"
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
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 10 }
    "#);
    assert_snapshot!(result.output, @r#"
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

    "#);
}

#[test]
fn variable_comes_first_type_should_move_above_variable_not_function() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const state: State = { loading: false }

        function updateState(newState: State) {
          return { ...state, ...newState }
        }

        type State = {
          loading: boolean
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 7 }
    "#);
    assert_snapshot!(result.output, @r#"
    type State = {
      loading: boolean
    }

    const state: State = { loading: false }

    function updateState(newState: State) {
      return { ...state, ...newState }
    }

    "#);
}

#[test]
fn type_below_variable_now_always_checked() {
    let result = invalid(
        RULE,
        indoc! {r#"
        const options: Options = { debug: true }

        type Options = { debug: boolean }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Options = { debug: boolean }

    const options: Options = { debug: true }

    "#);
}

#[test]
fn type_not_directly_above_usage_with_code_in_between() {
    let result = invalid(
        RULE,
        indoc! {r#"
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
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 1 }
    "#);
    assert_snapshot!(result.output, @r#"


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
    "#);
}

#[test]
fn type_not_directly_above_usage_with_single_variable_in_between() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Options = {
          debug: boolean
          timeout: number
          retries: number
        }

        const defaultTimeout = 5000

        function test(options: Options) {
          return options.debug ? 'debug mode' : 'production mode'
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 1 }
    "#);
    assert_snapshot!(result.output, @r#"


    const defaultTimeout = 5000

    type Options = {
      debug: boolean
      timeout: number
      retries: number
    }

    function test(options: Options) {
      return options.debug ? 'debug mode' : 'production mode'
    }
    "#);
}

#[test]
fn correctly_checks_types_in_nested_functions() {
    valid(
        RULE,
        indoc! {r#"
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
    "#},
    );
}

#[test]
fn test_case() {
    let result = invalid(
        RULE,
        indoc! {r#"
        export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
          return produce(initial, recipe);
        }

        export type ProduceRecipe<T> = (draft: T) => void | undefined | T;
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }

    "#);
}

#[test]
fn test_case_2() {
    let result = invalid(
        RULE,
        indoc! {r#"
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
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 13 }
    "#);
    assert_snapshot!(result.output, @r#"
    /** Creates a reducer using immer produce function. For more details visit
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
    }
    "#);
}

#[test]
fn real_file_with_imports() {
    let result = invalid(
        RULE,
        indoc! {r#"
        import { isFunction } from '@utils/assertions';
        import { produce } from 'immer';

        export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
          return produce(initial, recipe);
        }

        export type ProduceRecipe<T> = (draft: T) => void | undefined | T;
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 8 }
    "#);
    assert_snapshot!(result.output, @r#"
    import { isFunction } from '@utils/assertions';
    import { produce } from 'immer';

    export type ProduceRecipe<T> = (draft: T) => void | undefined | T;

    export function defaultProduce<T>(initial: T, recipe: ProduceRecipe<T>): T {
      return produce(initial, recipe);
    }

    "#);
}

#[test]
fn checkonly_function_ignores_types_used_outside_function_arguments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const data: UserData = { name: 'John' }

            type UserData = { name: string }

            function processUser(user: string) {
              return user.toUpperCase()
            }
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_fc_ignores_types_used_outside_fc_props() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const config: Config = { debug: true }

            type Config = { debug: boolean }

            const Button: React.FC<{ label: string }> = ({ label }) => {
              return <button>{label}</button>
            }
        "#},
            options: Some(json!([{"checkOnly":["FC"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_function_catches_type_used_in_function_argument() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            function processUser(data: UserData) {
              return data.name
            }

            type UserData = { name: string }
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type UserData = { name: string }

    function processUser(data: UserData) {
      return data.name
    }

    "#);
}

#[test]
fn checkonly_fc_catches_type_used_in_fc_props() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const Button: React.FC<ButtonProps> = ({ label }) => {
              return <button>{label}</button>
            }

            type ButtonProps = { label: string }
        "#},
            options: Some(json!([{"checkOnly":["FC"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type ButtonProps = { label: string }

    const Button: React.FC<ButtonProps> = ({ label }) => {
      return <button>{label}</button>
    }

    "#);
}

#[test]
fn checkonly_fc_catches_type_used_in_fc_with_import_alias() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            import { FC } from 'react'

            const Header: FC<HeaderProps> = ({ text }) => {
              return <h1>{text}</h1>
            }

            type HeaderProps = { text: string }
        "#},
            options: Some(json!([{"checkOnly":["FC"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 7 }
    "#);
    assert_snapshot!(result.output, @r#"
    import { FC } from 'react'

    type HeaderProps = { text: string }

    const Header: FC<HeaderProps> = ({ text }) => {
      return <h1>{text}</h1>
    }

    "#);
}

#[test]
fn checkonly_function_ignores_fc_props() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const Button: React.FC<ButtonProps> = ({ label }) => {
              return <button>{label}</button>
            }

            type ButtonProps = { label: string }
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_fc_ignores_function_arguments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function processUser(data: UserData) {
              return data.name
            }

            type UserData = { name: string }
        "#},
            options: Some(json!([{"checkOnly":["FC"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_both_function_and_fc_catches_both_contexts() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            function processUser(data: UserData) {
              return data.name
            }

            const Button: React.FC<ButtonProps> = ({ label }) => {
              return <button>{label}</button>
            }

            type UserData = { name: string }
            type ButtonProps = { label: string }
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 9 }
    "#);
    assert_snapshot!(result.output, @r#"
    type UserData = { name: string }

    function processUser(data: UserData) {
      return data.name
    }

    type ButtonProps = { label: string }

    const Button: React.FC<ButtonProps> = ({ label }) => {
      return <button>{label}</button>
    }


    "#);
}

#[test]
fn checkonly_function_allows_arrow_functions() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const handler: Handler = (id) => {
              console.log(id)
            }

            type Handler = (id: string) => void

            const process = (data: UserData) => data.name
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_function_catches_arrow_function_arguments() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const process = (data: UserData) => data.name

            type UserData = { name: string }
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    type UserData = { name: string }

    const process = (data: UserData) => data.name

    "#);
}

#[test]
fn checkonly_function_ignores_return_types_when_not_in_arguments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function createUser(): UserData {
              return { name: 'John' }
            }

            type UserData = { name: string }
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn considers_other_usages_as_valid() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
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
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_function_args_ignores_return_types() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            type Handler = () => void

            function createHandler(): Handler {
              return () => console.log('hello')
            }

            function useHandler(h: Handler) {
              h()
            }
        "#},
            options: Some(json!([{"checkOnly":["function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn valid_case() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            type Arg2 = number
            type Arg1 = string

            function test(arg1: Arg1, arg2: Arg2) {
              return arg1 + arg2
            }
        "#},
            options: Some(json!([{"checkOnly":["FC","function-args"]}])),
            filename: None,
        },
    );
}

#[test]
fn keep_comments_on_functions() {
    let result = invalid(
        RULE,
        indoc! {r#"
        type Arg1 = number

        const test = 'ok'

        /** @type {Arg1} */
        function test(arg1: Arg1) {
          return arg1
        }
    "#},
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 1 }
    "#);
    assert_snapshot!(result.output, @r#"


    const test = 'ok'

    type Arg1 = number

    /** @type {Arg1} */
    function test(arg1: Arg1) {
      return arg1
    }
    "#);
}

#[test]
fn checkonly_generic_args_at_fn_calls_ignores_types_used_in_function_arguments() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            function processUser(data: UserData) {
              return data.name
            }

            type UserData = { name: string }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_generic_args_at_fn_calls_ignores_types_used_in_fc_props() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const Button: React.FC<ButtonProps> = ({ label }) => {
              return <button>{label}</button>
            }

            type ButtonProps = { label: string }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_generic_args_at_fn_calls_ignores_variable_declarations() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const config: Config = { debug: true }

            type Config = { debug: boolean }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
}

#[test]
fn checkonly_generic_args_at_fn_calls_catches_type_used_in_function_call_generic() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = processData<DataType>({ value: 'test' })

            type DataType = { value: string }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    type DataType = { value: string }

    const result = processData<DataType>({ value: 'test' })

    "#);
}

#[test]
fn checkonly_generic_args_at_fn_calls_catches_type_used_in_method_call_generic() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const items = collection.filter<Item>(item => item.active)

            type Item = { active: boolean }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    type Item = { active: boolean }

    const items = collection.filter<Item>(item => item.active)

    "#);
}

#[test]
fn checkonly_generic_args_at_fn_calls_catches_type_in_chained_method_call_generic() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = data
              .map<ProcessedItem>(item => ({ ...item, processed: true }))
              .filter(item => item.active)

            type ProcessedItem = { processed: boolean; active: boolean }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 5 }
    "#);
    assert_snapshot!(result.output, @r#"
    type ProcessedItem = { processed: boolean; active: boolean }

    const result = data
      .map<ProcessedItem>(item => ({ ...item, processed: true }))
      .filter(item => item.active)

    "#);
}

#[test]
fn checkonly_generic_args_at_fn_calls_catches_type_in_constructor_call_generic() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const instance = new Collection<EntityType>()

            type EntityType = { id: string }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    type EntityType = { id: string }

    const instance = new Collection<EntityType>()

    "#);
}

#[test]
fn checkonly_generic_args_at_fn_calls_multiple_generic_args() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            const result = transform<InputType, OutputType>(data, processor)

            type InputType = { raw: string }
            type OutputType = { processed: string }
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 3 }
    "#);
    assert_snapshot!(result.output, @r#"
    type InputType = { raw: string }

    type OutputType = { processed: string }

    const result = transform<InputType, OutputType>(data, processor)


    "#);
}

#[test]
fn checkonly_generic_args_at_fn_calls_type_correctly_above_generic_function_call_usage() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            type ApiResponse = { data: string }

            const result = fetchData<ApiResponse>('/api/users')
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
}

#[test]
fn reproduce_bug() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
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
        "#},
            options: Some(json!([{"checkOnly":["generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 1 }
    "#);
    assert_snapshot!(result.output, @r#"










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
    });
    "#);
}

#[test]
fn reproduce_bug_with_all_options() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
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
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC","generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 1 }
    "#);
    assert_snapshot!(result.output, @r#"










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
    });
    "#);
}

#[test]
fn reproduce_bug_when_using_type_inside_function() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            type User = { name: string }

            const Component: FC = () => {
              const user: User[] = [{ name: 'John' }] // user is declared here so there is no error
            }

            const Component2: FC<{ user: User }> = ({ user }) => {
              return user.name
            }
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC"]}])),
            filename: None,
        },
    );
}

#[test]
fn reproduce_infinite_autofix_loop() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            export type TypeA =
              | 'a'
              | 'b'
              | 'c'
              | 'd'
              | 'e'
              | 'f';

            const typeASchema: RcType<TypeA> = rc_literals(
              'a',
              'b',
              'c',
              'd',
              'e',
              'f',
            ).withFallback('a');

            export type TypeB =
              | 'x'
              | 'y'
              | 'z'
              | 'w';

            const typeBSchema: RcType<TypeB> = rc_literals(
              'x',
              'y',
              'z',
              'w',
            );

            export type TypeC = {
              prop1: string | null;
              prop2: string | null;
              prop3: undefined | TypeF;
              prop4: TypeA;
              prop5: TypeB;
            };

            export const typeCSchema =
              rc_obj_builder<TypeC>()({
                prop1: rc_string.orNull(),
              });

            export type TypeD =
              | 'm'
              | 'n'
              | 'o'
              | 'p'
              | 'q'
              | 'r';

            export const typeDSchema: RcType<TypeD> = rc_literals(
              'm',

            );

            export type TypeE = {
              propA: string;
              propB: TypeC | null;
              propC: TypeG | null;
            };

            export type TypeH = { arr: TypeE[] };

            export type TypeG = {
              propX: string | null;
              propY: string | null;
              propZ: undefined | TypeF;
              propW: TypeA;
              propV: TypeB;
              propU: boolean | undefined;
              propT: boolean | undefined;
              propS: {
                m: boolean;
                n: boolean;
                o: boolean;
                p: boolean;
                q: boolean;
              };
              propR?: TypeD[];
              propQ: TypeH | null | undefined;
            };

            export const typeESchema = rc_obj_builder<TypeE>()({
              propA: rc_string,
              propB: rc_string,
              propC: rc_array(rc_string).orNull(),
              propD: rc_array(apiFilterSchema).orNull(),
              propE: ['null_or', { field: rc_string, order: rc_literals('asc', 'desc') }],
              propF: rc_boolean,
              propG: rc_boolean,
              propB: typeCSchema.orNull(),
              propC: rc_recursive(() => typeGSchema).orNull(),
            });

            const typeHSchema: RcType<TypeH> =
              rc_obj_builder<TypeH>()({
                arr: rc_array(typeESchema),
              });

            export const typeGSchema =
              rc_obj_builder<TypeG>()({
                propX: rc_string.orNull(),
                propY: rc_string.orNull(),
                propW: typeASchema,
                propV: typeBSchema,
                propZ: typeFSchema.optional(),
                propU: rc_boolean.optional(),
                propT: rc_boolean.optional(),
                propS: {
                  m: rc_boolean,
                  n: rc_boolean,
                  o: rc_boolean,
                  p: rc_boolean,
                  q: rc_boolean,
                },
                propR: rc_loose_array(typeDSchema).optional(),
                propQ: typeHSchema.orNullish(),
              });
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC","generic-args-at-fn-calls"]}])),
            filename: None,
        },
    );
}

#[test]
fn reproduce_bug_when_using_type_inside_another_type() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            type User = { name: string }

            type Test2 = { ok: true } & User

            const Component: FC = () => {
              console.log('test')
            }

            const Component2: FC<{ user: User }> = ({ user }) => {
              return user.name
            }
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC"]}])),
            filename: None,
        },
    );
}

#[test]
fn type_used_in_contexts_other_than_checkonly_but_after_the_first_checkonly_context() {
    let result = invalid(
        RULE,
        Case {
            code: indoc! {r#"
            type User = { name: string }

            const Component: FC = () => {
              console.log('test')
            }

            const Component2: FC<{ user: User }> = ({ user }) => {
              return user.name
            }

            type Test2 = { ok: true } & User
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC"]}])),
            filename: None,
        },
    );
    assert_snapshot!(result.errors(), @r#"

    - { messageId: 'moveTypeAboveUsage', line: 1 }
    "#);
    assert_snapshot!(result.output, @r#"


    const Component: FC = () => {
      console.log('test')
    }

    type User = { name: string }

    const Component2: FC<{ user: User }> = ({ user }) => {
      return user.name
    }

    type Test2 = { ok: true } & User
    "#);
}

#[test]
fn should_consider_only_direct_usages_not_property_access_ones() {
    valid(
        RULE,
        Case {
            code: indoc! {r#"
            const Component2: FC<{ name: Props['name'] }> = ({ name }) => {
              return name
            }

            type Props = { name: string }

            const Component: FC<Props> = ({ name }) => {
              return name
            }
        "#},
            options: Some(json!([{"checkOnly":["function-args","FC"]}])),
            filename: None,
        },
    );
}
