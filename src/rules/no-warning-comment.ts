import { Rule } from 'eslint'
// @ts-ignore
import rule from '../../eslint/lib/rules/no-warning-comments'

export const noWarningComment = {
  name: 'no-warning-comments',
  rule: rule as Rule.RuleModule,
}
