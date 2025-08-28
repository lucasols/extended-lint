import { dedent } from '@ls-stack/utils/dedent'
import { describe, expect, test } from 'vitest'
import {
  createNewTester,
  getErrorsWithMsgFromResult,
  getSuggestionOutput,
} from '../../tests/utils/createTester'
import { reactOptimizedCollections } from './react-optimized-collections'

const { valid, invalid } = createNewTester(reactOptimizedCollections)

describe('valid cases - no errors', () => {
  test('primitive values in map', async () => {
    await valid(
      dedent`
        function TodoList({ todos }) {
          return todos.map(todo => (
            <div key={todo.id} className="todo">
              {todo.text}
            </div>
          ));
        }
      `,
    )
  })

  test('variables as props', async () => {
    await valid(
      dedent`
        function TodoList({ todos, className }) {
          const style = { padding: 10 };
          const handleClick = (id) => console.log(id);

          return todos.map(todo => (
            <div 
              key={todo.id} 
              className={className}
              style={style}
              onClick={handleClick}
            >
              {todo.text}
            </div>
          ));
        }
      `,
    )
  })

  test('already extracted component', async () => {
    await valid(
      dedent`
        const TodoItem = ({ todo }) => <div>{todo.text}</div>;
        
        function TodoList({ todos }) {
          return todos.map(todo => <TodoItem key={todo.id} todo={todo} />);
        }
      `,
    )
  })

  test('non-JSX map return', async () => {
    await valid(
      dedent`
        function processData(items) {
          return items.map(item => ({ 
            id: item.id,
            processed: true 
          }));
        }
      `,
    )
  })

  test('without compiler directive when runOnlyWithEnableCompilerDirective is true', async () => {
    await valid({
      code: dedent`
        function TodoList({ todos }) {
          return todos.map(todo => (
            <div 
              key={todo.id}
              style={{ padding: 10 }}
              onClick={() => handleClick(todo.id)}
            >
              {todo.text}
            </div>
          ));
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: true }],
    })
  })
})

describe('invalid cases - should error and suggest', () => {
  test('inline object in map with compiler directive', async () => {
    const { result } = await invalid({
      code: dedent`
        function TodoList({ todos }) {
          return todos.map(todo => (
            <div 
              key={todo.id}
              style={{ padding: 10, margin: 5 }}
            >
              {todo.text}
            </div>
          ));
        }
      `,
      filename: 'test.tsx',
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in prop "style" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 3
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function TodoList({ todos }) {
        return todos.map(todo => (
          <Todo key={todo.id} todo={todo} />
        ));
      }

      type TodoProps = {
        todo: TodoType;
      };

      const Todo: FC<TodoProps> = ({ todo }) => {
        return (
          <div 
            key={todo.id}
            style={{ padding: 10, margin: 5 }}
          >
            {todo.text}
          </div>
        );
      };"
    `)
  })

  test('inline function in map', async () => {
    const { result } = await invalid({
      code: dedent`
        function TodoList({ todos, onDelete }) {
          return todos.map(todo => (
            <div 
              key={todo.id}
              onClick={() => onDelete(todo.id)}
            >
              {todo.text}
            </div>
          ));
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in prop "onClick" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 3
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function TodoList({ todos, onDelete }) {
        return todos.map(todo => (
          <Todo key={todo.id} todo={todo} onDelete={onDelete} />
        ));
      }

      type TodoProps = {
        todo: TodoType;
        onDelete: unknown;
      };

      const Todo: FC<TodoProps> = ({ todo, onDelete }) => {
        return (
          <div 
            key={todo.id}
            onClick={() => onDelete(todo.id)}
          >
            {todo.text}
          </div>
        );
      };"
    `)
  })

  test('complex case with multiple unstable values and closures', async () => {
    const { result } = await invalid({
      code: dedent`
        function ProductList({ products }) {
          const theme = useTheme();
          const currency = 'USD';

          return (
            <div>
              {products.map((product, index) => (
                <div 
                  key={product.id}
                  style={{ 
                    background: index % 2 ? '#f0f0f0' : 'white',
                    border: '1px solid gray' 
                  }}
                  onClick={() => handleClick(product.id)}
                  data-currency={currency}
                  className={theme.productClass}
                >
                  {product.name} - {product.price}
                </div>
              ))}
            </div>
          );
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in props "style", "onClick" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 8
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function ProductList({ products }) {
        const theme = useTheme();
        const currency = 'USD';

        return (
          <div>
            {products.map((product, index) => (
              <Product key={product.id} product={product} index={index} handleClick={handleClick} currency={currency} theme={theme} />
            ))}
          </div>
        );
      }

      type ProductProps = {
        product: ProductType;
        index: number;
        handleClick: unknown;
        currency: unknown;
        theme: unknown;
      };

      const Product: FC<ProductProps> = ({ product, index, handleClick, currency, theme }) => {
        return (
          <div 
                key={product.id}
                style={{ 
                  background: index % 2 ? '#f0f0f0' : 'white',
                  border: '1px solid gray' 
                }}
                onClick={() => handleClick(product.id)}
                data-currency={currency}
                className={theme.productClass}
              >
                {product.name} - {product.price}
              </div>
        );
      };"
    `)
  })

  test('name inference from various patterns', async () => {
    const { result } = await invalid({
      code: dedent`
        function UserList({ userAccounts }) {
          return userAccounts.map(account => (
            <div key={account.id} style={{ padding: 5 }}>
              {account.name}
            </div>
          ));
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in prop "style" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 3
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function UserList({ userAccounts }) {
        return userAccounts.map(account => (
          <Useraccount key={account.id} account={account} />
        ));
      }

      type UseraccountProps = {
        account: UseraccountType;
      };

      const Useraccount: FC<UseraccountProps> = ({ account }) => {
        return (
          <div key={account.id} style={{ padding: 5 }}>
            {account.name}
          </div>
        );
      };"
    `)
  })

  test('inline array in props', async () => {
    const { result } = await invalid({
      code: dedent`
        function List({ items }) {
          return items.map(item => (
            <div key={item.id} data-tags={['tag1', 'tag2']}>
              {item.name}
            </div>
          ));
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in prop "data-tags" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 3
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function List({ items }) {
        return items.map(item => (
          <Item key={item.id} item={item} />
        ));
      }

      type ItemProps = {
        item: ItemType;
      };

      const Item: FC<ItemProps> = ({ item }) => {
        return (
          <div key={item.id} data-tags={['tag1', 'tag2']}>
            {item.name}
          </div>
        );
      };"
    `)
  })

  test('array.push with inline object', async () => {
    const { result } = await invalid({
      code: dedent`
        function buildList(items) {
          const elements = [];
          for (const item of items) {
            elements.push(
              <div key={item.id} style={{ padding: 10 }}>
                {item.name}
              </div>
            );
          }
          return elements;
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in prop "style" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 5
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function buildList(items) {
        const elements = [];
        for (const item of items) {
          elements.push(
            <ListItem key={item.id} item={item} />
          );
        }
        return elements;
      }

      type ListItemProps = {
        item: unknown;
      };

      const ListItem: FC<ListItemProps> = ({ item }) => {
        return (
          <div key={item.id} style={{ padding: 10 }}>
              {item.name}
            </div>
        );
      };"
    `)
  })

  test('array.push with inline function', async () => {
    const { result } = await invalid({
      code: dedent`
        function buildList(items, onDelete) {
          const elements = [];
          for (const item of items) {
            elements.push(
              <div key={item.id} onClick={() => onDelete(item.id)}>
                {item.name}
              </div>
            );
          }
          return elements;
        }
      `,
      options: [{ runOnlyWithEnableCompilerDirective: false }],
    })

    expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
      "
      - messageId: 'unstableValueInMap'
        msg: 'Unstable values in prop "onClick" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
        line: 5
      "
    `)
    expect(getSuggestionOutput(result)).toMatchInlineSnapshot(`
      "function buildList(items, onDelete) {
        const elements = [];
        for (const item of items) {
          elements.push(
            <ListItem key={item.id} item={item} onDelete={onDelete} />
          );
        }
        return elements;
      }

      type ListItemProps = {
        item: unknown;
        onDelete: unknown;
      };

      const ListItem: FC<ListItemProps> = ({ item, onDelete }) => {
        return (
          <div key={item.id} onClick={() => onDelete(item.id)}>
              {item.name}
            </div>
        );
      };"
    `)
  })
})

describe('array.push cases - valid (no errors)', () => {
  test('array.push with stable values', async () => {
    await valid(
      dedent`
        function buildList(items, className) {
          const elements = [];
          for (const item of items) {
            elements.push(
              <div key={item.id} className={className}>
                {item.name}
              </div>
            );
          }
          return elements;
        }
      `,
    )
  })

  test('array.push with already extracted component', async () => {
    await valid(
      dedent`
        const ListItem = ({ item }) => <div>{item.name}</div>;
        
        function buildList(items) {
          const elements = [];
          for (const item of items) {
            elements.push(<ListItem key={item.id} item={item} />);
          }
          return elements;
        }
      `,
    )
  })
})

test('runOnlyWithEnableCompilerDirective option works', async () => {
  await valid({
    code: dedent`
      function TodoList({ todos }) {
        return todos.map(todo => (
          <div 
            key={todo.id}
            style={{ padding: 10 }}
          >
            {todo.text}
          </div>
        ));
      }
    `,
    options: [{ runOnlyWithEnableCompilerDirective: true }],
  })

  const { result } = await invalid({
    code: dedent`
      // eslint react-compiler/react-compiler: ["error"]
      function TodoList({ todos }) {
        return todos.map(todo => (
          <div 
            key={todo.id}
            style={{ padding: 10 }}
          >
            {todo.text}
          </div>
        ));
      }
    `,
    options: [{ runOnlyWithEnableCompilerDirective: true }],
  })

  expect(getErrorsWithMsgFromResult(result)).toMatchInlineSnapshot(`
    "
    - messageId: 'unstableValueInMap'
      msg: 'Unstable values in prop "style" prevent React Compiler from optimizing individual list items. Extract the problematic props to outside the loop or extract the item to a separate component.'
      line: 4
    "
  `)
})
