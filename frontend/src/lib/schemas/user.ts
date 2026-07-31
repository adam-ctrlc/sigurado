import { z } from 'zod';

export const roleSchema = z.enum(['student', 'faculty', 'admin']);
export type Role = z.infer<typeof roleSchema>;

export const userSchema = z.object({
	id: z.string(),
	email: z.string(),
	username: z.string(),
	first_name: z.string(),
	middle_name: z.string().nullish(),
	last_name: z.string(),
	suffix: z.string().nullish(),
	/** Where SMS alerts go. Only staff are ever subscribed. */
	phone_number: z.string().nullish(),
	/** Server-computed: the name parts joined for display. */
	display_name: z.string(),
	role: roleSchema,
	is_active: z.boolean(),
	last_login_at: z.string().nullish(),
	created_at: z.string()
});
export type User = z.infer<typeof userSchema>;

export const userListSchema = z.array(userSchema);

export const fingerprintSchema = z.object({
	id: z.string(),
	user_id: z.string(),
	finger_token: z.string(),
	label: z.string().nullish(),
	created_at: z.string()
});
export type Fingerprint = z.infer<typeof fingerprintSchema>;

export const fingerprintListSchema = z.array(fingerprintSchema);

export const userBriefSchema = z.object({
	id: z.string(),
	username: z.string(),
	display_name: z.string(),
	role: roleSchema
});
export type UserBrief = z.infer<typeof userBriefSchema>;

export const loginEventSchema = z.object({
	id: z.string(),
	success: z.boolean(),
	reason: z.string().nullish(),
	ip_address: z.string().nullish(),
	user_agent: z.string().nullish(),
	created_at: z.string()
});
export type LoginEvent = z.infer<typeof loginEventSchema>;

export const loginEventListSchema = z.array(loginEventSchema);
