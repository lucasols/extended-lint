import { noCommentedOutCode } from './no-commented-code'
import { noUnusedObjectTypeProperties } from './no-unused-type-props-in-args'

export const rules = {
  [noUnusedObjectTypeProperties.name]: noUnusedObjectTypeProperties.rule,
  [noCommentedOutCode.name]: noCommentedOutCode.rule,
}
