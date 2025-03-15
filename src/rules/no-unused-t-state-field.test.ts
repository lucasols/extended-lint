import { createTester } from '../../tests/utils/createTester'
import { noUnusedTStateField } from './no-unused-t-state-field'

const tests = createTester(noUnusedTStateField, {
  defaultErrorId: 'unusedField',
})

tests.addValid(
  'with no unused field',
  `
    import { useForm } from 't-state-form';

    const Component = () => {
      const { formTypedCtx } = useForm({
        initialConfig: {
          name: { initialValue: 'John' },
          text: { initialValue: 'text' },
        },
      })

      const { formFields } = useFormState(formTypedCtx)

      return <div>
        <TextField value={formFields.name.value} />
        <TextField value={formFields.text.value} />
      </div>
    }
  `,
)

tests.addInvalid(
  'has unused field',
  `
    import { useForm } from 't-state-form';

    const Component = () => {
      const { formTypedCtx } = useForm({
        initialConfig: {
          name: { initialValue: 'John' },
          unused: { initialValue: 'unused' },
        },
      })

      const { formFields } = useFormState(formTypedCtx)

      return <div>
        <TextField value={formFields.name.value} />
      </div>
    }
  `,
  [{ data: { name: 'unused' } }],
)

tests.addValid(
  'files with missing import are ignored',
  `
    const Component = () => {
      const { formTypedCtx } = useForm({
        initialConfig: {
          name: { initialValue: 'John' },
          unused: { initialValue: 'unused' },
        },
      })

      const { formFields } = useFormState(formTypedCtx)

      return <div>
        <TextField value={formFields.name.value} />
      </div>
    }
  `,
)

tests.addInvalid(
  'unused fields are checked when spread is used',
  `
    import { useForm } from 't-state-form';

    const Component = ({ config }) => {
      const { formTypedCtx } = useForm({
        initialConfig: {
          name: { initialValue: 'John' },
          unused: { initialValue: 'unused' },
          ...config,
        },
      })

      const { formFields } = useFormState(formTypedCtx)

      return <div>
        <TextField value={formFields.name.value} />
      </div>
    }
  `,
  [{ data: { name: 'unused' } }],
)

tests.run()
