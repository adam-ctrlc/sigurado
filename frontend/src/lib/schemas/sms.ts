import { z } from 'zod';
import { roleSchema } from '$lib/schemas/user';

export const smsStatusSchema = z.enum(['pending', 'sent', 'failed']);
export type SmsStatus = z.infer<typeof smsStatusSchema>;

export const smsRecipientSchema = z.object({
	id: z.string(),
	/** The staff account behind this subscription. */
	user_id: z.string(),
	display_name: z.string(),
	username: z.string(),
	role: roleSchema,
	/** From the user record; null means nothing can be sent yet. */
	phone_number: z.string().nullish(),
	user_is_active: z.boolean(),
	notify_cabinet_opened: z.boolean(),
	notify_access_denied: z.boolean(),
	notify_door_opened: z.boolean(),
	is_active: z.boolean(),
	created_at: z.string()
});
export type SmsRecipient = z.infer<typeof smsRecipientSchema>;

export const smsRecipientListSchema = z.array(smsRecipientSchema);

/** A staff account that could be subscribed but is not yet. */
export const smsCandidateSchema = z.object({
	id: z.string(),
	display_name: z.string(),
	username: z.string(),
	role: roleSchema,
	phone_number: z.string().nullish()
});
export type SmsCandidate = z.infer<typeof smsCandidateSchema>;

export const smsCandidateListSchema = z.array(smsCandidateSchema);

export const smsMessageSchema = z.object({
	id: z.string(),
	recipient_id: z.string().nullish(),
	recipient_label: z.string().nullish(),
	phone_number: z.string(),
	body: z.string(),
	status: smsStatusSchema,
	attempts: z.number(),
	error: z.string().nullish(),
	created_at: z.string(),
	sent_at: z.string().nullish()
});
export type SmsMessage = z.infer<typeof smsMessageSchema>;
