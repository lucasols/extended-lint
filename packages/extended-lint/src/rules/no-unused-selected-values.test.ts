import { createTester } from '../../tests/utils/createTester'
import { noUnusedSelectedValues } from './no-unused-selected-values'

const tests = createTester(noUnusedSelectedValues, {
  defaultErrorId: 'unusedSelectedValue',
})

// Valid test cases
tests.addValid(
  'no selector usage',
  `
    const x = 5;
  `,
)

tests.addValid(
  'all selected values are used',
  `
    const { currency } = store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
    }));
  `,
)

tests.addValid(
  'all selected values are used with direct selector',
  `
    const { currency, locale } = store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      locale: state.data?.locale_settings?.locale,
    }));
  `,
  { selectors: [{ name: 'useSelector' }] },
)

tests.addValid(
  'all selected values are used with nested property',
  `
    const {
      data: { deleteOrgAt, isTest, orgId },
    } = configDoc.useDocument({
      selector: (data) => ({
        deleteOrgAt: data?.delete_at,
        isTest: data?.is_test,
        orgId: data?.id,
      }),
    });
  `,
  { selectors: [{ name: 'useDocument.selector', returnProp: 'data' }] },
)

tests.addValid(
  'using rest operator',
  `
    const { currency, ...rest } = configDoc.store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));
  `,
  { selectors: [{ name: 'useSelector' }] },
)

tests.addValid(
  'non-object selector return values',
  `
    const value = useSimpleSelector(state => state.value);
  `,
  { selectors: [{ name: 'useSimpleSelector' }] },
)

// Invalid test cases
tests.addInvalidWithOptions(
  'unused selected value',
  `
    const { currency } = useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));
  `,
  { selectors: [{ name: 'useSelector' }] },
  [{ messageId: 'unusedSelectedValue', data: { name: 'unusedValue' } }],
)

tests.addInvalidWithOptions(
  'unused selected value 2',
  `
    const { currency } = useSelector((state) => {
      return {
        currency: state.data?.locale_settings?.currency_symbol,
        unusedValue: state.data.id,
      }
    });
  `,
  { selectors: [{ name: 'useSelector' }] },
  [{ messageId: 'unusedSelectedValue', data: { name: 'unusedValue' } }],
)

tests.addInvalidWithOptions(
  'unused selected value 3',
  `
    const { currency } = store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));
  `,
  { selectors: [{ name: 'useSelector' }] },
  [{ messageId: 'unusedSelectedValue', data: { name: 'unusedValue' } }],
)

tests.addInvalidWithOptions(
  'multiple unused selected values',
  `
    const { currency } = test.store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue1: state.data.id,
      unusedValue2: state.data.name,
    }));
  `,
  [{ selectors: [{ name: 'useSelector' }] }],
  [
    { messageId: 'unusedSelectedValue', data: { name: 'unusedValue1' } },
    { messageId: 'unusedSelectedValue', data: { name: 'unusedValue2' } },
  ],
)

tests.addInvalidWithOptions(
  'unused selected value with selectorProp',
  `
    const { deleteOrgAt, isTest } = configDoc.useDocument({
      selector: (data) => ({
        deleteOrgAt: data?.delete_at,
        isTest: data?.is_test,
        orgId: data?.id,
      }),
    });
  `,
  { selectors: [{ name: 'useDocument', selectorProp: 'selector' }] },
  [{ messageId: 'unusedSelectedValue', data: { name: 'orgId' } }],
)

tests.addInvalidWithOptions(
  'unused selected value with selectorProp and selectorArgPos',
  `
    const { deleteOrgAt, isTest } = configDoc.useItem('test', {
      selector: (data) => ({
        deleteOrgAt: data?.delete_at,
        isTest: data?.is_test,
        orgId: data?.id,
      }),
    });
  `,
  {
    selectors: [
      { name: 'useItem', selectorProp: 'selector', selectorArgPos: 1 },
    ],
  },
  [{ messageId: 'unusedSelectedValue', data: { name: 'orgId' } }],
)

tests.addInvalidWithOptions(
  'unused selected value with selectorProp 2',
  `
    const { deleteOrgAt, isTest } = configDoc.useDocument({
      selector(data) {
        return {
          deleteOrgAt: data?.delete_at,
          isTest: data?.is_test,
          orgId: data?.id,
        }
      },
    });
  `,
  { selectors: [{ name: 'useDocument', selectorProp: 'selector' }] },
  [{ messageId: 'unusedSelectedValue', data: { name: 'orgId' } }],
)

tests.addInvalidWithOptions(
  'unused selected value with returnProp',
  `
    const {
      data: { deleteOrgAt, isTest },
    } = configDoc.useDocument({
      selector: (data) => ({
        deleteOrgAt: data?.delete_at,
        isTest: data?.is_test,
        orgId: data?.id,
      }),
    });
  `,
  {
    selectors: [
      { name: 'useDocument', returnProp: 'data', selectorProp: 'selector' },
    ],
  },
  [{ messageId: 'unusedSelectedValue', data: { name: 'orgId' } }],
)

tests.addInvalidWithOptions(
  'multiple unused nested selected values',
  `
    const {
      data: { deleteOrgAt },
    } = configDoc.useDocument({
      selector: (data) => ({
        deleteOrgAt: data?.delete_at,
        isTest: data?.is_test,
        orgId: data?.id,
      }),
    });
  `,
  {
    selectors: [
      { name: 'useDocument', returnProp: 'data', selectorProp: 'selector' },
    ],
  },
  [
    { messageId: 'unusedSelectedValue', data: { name: 'isTest' } },
    { messageId: 'unusedSelectedValue', data: { name: 'orgId' } },
  ],
)

tests.addInvalidWithOptions(
  'complex member expression match',
  `
    const { currency } = myApp.features.user.settings.useUserPreferences((state) => ({
      currency: state.settings.currency,
      language: state.settings.language,
    }));
  `,
  { selectors: [{ name: 'useUserPreferences' }] },
  [{ messageId: 'unusedSelectedValue', data: { name: 'language' } }],
)

tests.describe('unused property access', () => {
  tests.addValid(
    'not using destructuring',
    `
    const result = configDoc.store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));
  `,
    { selectors: [{ name: 'useSelector' }] },
  )

  tests.addInvalidWithOptions(
    'with property access',
    `
    const result = configDoc.store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));

    console.log(result.currency);
  `,
    { selectors: [{ name: 'useSelector' }] },
    [{ messageId: 'unusedSelectedValue', data: { name: 'unusedValue' } }],
  )

  tests.addValid(
    'not using property access',
    `
    const result = configDoc.store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));

    console.log(result);
  `,
    { selectors: [{ name: 'useSelector' }] },
  )

  tests.addValid(
    'with partial property access',
    `
    const result = configDoc.store.useSelector((state) => ({
      currency: state.data?.locale_settings?.currency_symbol,
      unusedValue: state.data.id,
    }));

    console.log(result.currency);
    console.log(result);
  `,
    { selectors: [{ name: 'useSelector' }] },
  )

  tests.addValid(
    'with return prop',
    `
    const config = configStore.useDocument({
      selector: (data) => ({
        icon: data?.logo_customizado,
        name: data?.name,
      }),
    });

    console.log(config.data.icon);
    console.log(config.data.name);
  `,
    {
      selectors: [
        { name: 'useDocument', returnProp: 'data', selectorProp: 'selector' },
      ],
    },
  )
})

tests.run()
