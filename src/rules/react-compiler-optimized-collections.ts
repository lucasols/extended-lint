import { z } from 'zod'

const optionsSchema = z.object({
  runOnlyWithEnableCompilerDirective: z.boolean().optional(),
})
