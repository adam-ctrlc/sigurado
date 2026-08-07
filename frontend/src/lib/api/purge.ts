import { z } from 'zod';
import { api } from '$lib/api/client';

export const countsSchema = z.object({
	events: z.number(),
	checkouts: z.number(),
	sessions: z.number(),
	logins: z.number(),
	messages: z.number(),
	enrollments: z.number()
});
export type Counts = z.infer<typeof countsSchema>;

export const purgeRecordSchema = z.object({
	id: z.string(),
	purged_at: z.string(),
	performed_by_name: z.string().nullish(),
	note: z.string().nullish(),
	hidden: countsSchema,
	undone_at: z.string().nullish()
});
export type PurgeRecord = z.infer<typeof purgeRecordSchema>;

export const purgeStatusSchema = z.object({
	/** The phrase the server will accept, so the page never invents its own. */
	confirmation: z.string(),
	visible: countsSchema,
	visible_total: z.number(),
	active: purgeRecordSchema.nullish(),
	history: z.array(purgeRecordSchema)
});
export type PurgeStatus = z.infer<typeof purgeStatusSchema>;

export function purgeStatus(): Promise<PurgeStatus> {
	return api.get('/admin/purge', purgeStatusSchema);
}

/** The phrase is checked again on the server, so this cannot be bypassed. */
export function clearData(confirm: string, note?: string): Promise<PurgeRecord> {
	return api.post('/admin/purge', purgeRecordSchema, { confirm, note });
}

export function undoClear(): Promise<PurgeRecord> {
	return api.post('/admin/purge/undo', purgeRecordSchema);
}

/** What each counter means, in the order the page shows them. */
export const COUNT_LABELS: { key: keyof Counts; label: string; detail: string }[] = [
	{ key: 'events', label: 'Reader events', detail: 'Scans, unlocks, refusals and enrollments' },
	{ key: 'checkouts', label: 'Checkouts', detail: 'What people recorded taking' },
	{ key: 'sessions', label: 'Door windows', detail: 'Each door scan that opened a window' },
	{ key: 'logins', label: 'Sign-ins', detail: 'Website sign-ins, successful and failed' },
	{ key: 'messages', label: 'Text messages', detail: 'The alert outbox' },
	{ key: 'enrollments', label: 'Enrollment codes', detail: 'Codes issued at a reader' }
];
