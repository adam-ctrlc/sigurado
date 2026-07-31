import { z } from 'zod';
import { userBriefSchema } from '$lib/schemas/user';

export const scanActionSchema = z.enum(['unlock', 'deny']);
export type ScanAction = z.infer<typeof scanActionSchema>;

export const scanResponseSchema = z.object({
	action: scanActionSchema,
	reason: z.string(),
	user: userBriefSchema.nullish(),
	session_expires_at: z.string().nullish(),
	enroll_hint: z.boolean().nullish()
});
export type ScanResponse = z.infer<typeof scanResponseSchema>;

export const enrollCodeResponseSchema = z.object({
	code: z.string(),
	code_expires_at: z.string()
});
export type EnrollCodeResponse = z.infer<typeof enrollCodeResponseSchema>;

export const enrollBindResponseSchema = z.object({
	bound: z.boolean(),
	message: z.string(),
	user: userBriefSchema.nullish()
});
export type EnrollBindResponse = z.infer<typeof enrollBindResponseSchema>;
