import { z } from 'zod';

export const deviceKindSchema = z.enum(['door', 'box']);
export type DeviceKind = z.infer<typeof deviceKindSchema>;

export const deviceSchema = z.object({
	id: z.string(),
	name: z.string(),
	kind: deviceKindSchema,
	last_seen_at: z.string().nullish(),
	/** True on the node carrying the SIM800L, which drains the SMS queue. */
	sms_capable: z.boolean(),
	/** Start of the current unbroken stretch of contact; null when offline. */
	connected_since: z.string().nullish(),
	is_active: z.boolean(),
	created_at: z.string()
});
export type Device = z.infer<typeof deviceSchema>;

export const deviceListSchema = z.array(deviceSchema);

export const deviceWithSecretSchema = deviceSchema.extend({
	secret: z.string()
});
export type DeviceWithSecret = z.infer<typeof deviceWithSecretSchema>;

export const deviceConnectionSchema = z.object({
	id: z.string(),
	connected_at: z.string(),
	last_seen_at: z.string(),
	ended_at: z.string().nullish(),
	seconds: z.number(),
	heartbeats: z.number(),
	current: z.boolean()
});
export type DeviceConnection = z.infer<typeof deviceConnectionSchema>;

export const deviceConnectionListSchema = z.array(deviceConnectionSchema);
