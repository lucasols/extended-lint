import { Rule } from 'eslint'
// @ts-ignore
import rule from '../../eslint/lib/rules/no-warning-comment.cjs'

export const noWarningComment = {
  name: 'no-warning-comment',
  rule: rule as Rule.RuleModule,
}
