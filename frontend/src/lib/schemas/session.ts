import { z } from 'zod';

export const sessionStatusSchema = z.enum(['active', 'expired', 'consumed', 'closed']);
export type SessionStatus = z.infer<typeof sessionStatusSchema>;

export const sessionSchema = z.object({
	id: z.string(),
	user_id: z.string(),
	user_name: z.string().nullish(),
	door_device_id: z.string(),
	door_device_name: z.string().nullish(),
	opened_at: z.string(),
	expires_at: z.string(),
	status: sessionStatusSchema,
	is_live: z.boolean()
});
export type AccessSession = z.infer<typeof sessionSchema>;

export const sessionListSchema = z.array(sessionSchema);
