import { z } from 'zod';

export const eventDecisionSchema = z.enum(['granted', 'denied', 'info']);
export type EventDecision = z.infer<typeof eventDecisionSchema>;

export const accessEventTypeSchema = z.enum([
	'door_granted',
	'door_denied_unregistered',
	'door_denied_inactive',
	'box_granted',
	'box_denied_no_session',
	'box_denied_unregistered',
	'box_denied_session_expired',
	'enroll_code_issued',
	'enroll_bound',
	'enroll_failed',
	'checkout_code_rejected'
]);
export type AccessEventType = z.infer<typeof accessEventTypeSchema>;

export const eventSchema = z.object({
	id: z.string(),
	event_type: accessEventTypeSchema,
	decision: eventDecisionSchema,
	device_id: z.string().nullish(),
	device_name: z.string().nullish(),
	user_id: z.string().nullish(),
	user_name: z.string().nullish(),
	finger_token: z.string().nullish(),
	/** Resolved from the token when the event predates the enrollment. */
	finger_owner_id: z.string().nullish(),
	finger_owner_name: z.string().nullish(),
	access_session_id: z.string().nullish(),
	message: z.string().nullish(),
	created_at: z.string()
});
export type AccessEvent = z.infer<typeof eventSchema>;

export const eventListSchema = z.array(eventSchema);
