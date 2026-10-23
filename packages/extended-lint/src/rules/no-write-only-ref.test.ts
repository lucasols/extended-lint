import { createTester } from '../../tests/utils/createTester'
import { noWriteOnlyRef } from './no-write-only-ref'

const tests = createTester(noWriteOnlyRef, {
  defaultErrorId: 'refNotRead',
})
// Invalid cases
tests.addInvalid(
  'ref created but never read - single ref',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      return <div ref={ref} />
    }
  `,
  [{ data: { name: 'ref' } }]
)

tests.addInvalid(
  'ref created but never read - input ref',
  `
    import { useRef } from 'react'
    
    function Component() {
      const inputRef = useRef(null)
      return <input ref={inputRef} />
    }
  `,
  [{ data: { name: 'inputRef' } }]
)

tests.addInvalid(
  'multiple refs only used as ref prop',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(0)
      const anotherRef = useRef(null)
      return (
        <>
          <Child ref={ref} />
          <Other ref={anotherRef} />
        </>
      )
    }
  `,
  [
    { data: { name: 'ref' } },
    { data: { name: 'anotherRef' } },
  ]
)

// Valid cases
tests.addValid(
  'ref used in useEffect',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      
      useEffect(() => {
        if (ref.current) {
          ref.current.focus()
        }
      }, [])
      
      return <input ref={ref} />
    }
  `
)

tests.addValid(
  'ref.current modified and read',
  `
    import { useRef } from 'react'
    
    function Component() {
      const countRef = useRef(0)
      
      const handleClick = () => {
        countRef.current++
        console.log(countRef.current)
      }
      
      return <button onClick={handleClick}>Click</button>
    }
  `
)

tests.addValid(
  'ref.current read directly',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      const value = ref.current
      return <div ref={ref}>{value}</div>
    }
  `
)

tests.addValid(
  'ref.current passed to function',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      doSomething(ref.current)
      return null
    }
  `
)

tests.addValid(
  'ref destructured',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      const { current } = ref
      return <div>{current}</div>
    }
  `
)

tests.addValid(
  'ref.current used in object property',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      const obj = { prop: ref.current }
      return null
    }
  `
)

tests.addValid(
  'ref passed to function',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef()
      doSomething(ref)
      return null
    }
  `
)

tests.addValid(
  'ref passed in JSX expression',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref = useRef(null)
      return <Child someRef={ref} />
    }
  `
)

tests.addInvalid(
  'ref only used as ref prop',
  `
    import { useRef } from 'react'
    
    function Component() {
      const myRef = useRef(null)
      const anotherRef = useRef(null)
      return (
        <>
          <div ref={myRef} />
          <span ref={anotherRef} />
        </>
      )
    }
  `,
  [
    { data: { name: 'myRef' } },
    { data: { name: 'anotherRef' } },
  ]
)

tests.addInvalid(
  'renamed useRef import',
  `
    import { useRef as useReference } from 'react'
    
    function Component() {
      const ref = useReference(null)
      return <div ref={ref} />
    }
  `,
  [{ data: { name: 'ref' } }]
)

tests.addInvalid(
  'multiple refs with only one read',
  `
    import { useRef } from 'react'
    
    function Component() {
      const ref1 = useRef(null)
      const ref2 = useRef(null)
      const ref3 = useRef(null)
      
      console.log(ref2.current)
      
      return (
        <>
          <div ref={ref1} />
          <div ref={ref3} />
        </>
      )
    }
  `,
  [
    { data: { name: 'ref1' } },
    { data: { name: 'ref3' } },
  ]
)

tests.run()