import { z } from 'zod';
import { api } from '$lib/api/client';

export const severitySchema = z.enum(['high', 'medium', 'low']);
export type Severity = z.infer<typeof severitySchema>;

export const flagSchema = z.object({
	kind: z.string(),
	severity: severitySchema,
	title: z.string(),
	detail: z.string(),
	at: z.string(),
	person: z.string().nullish(),
	device: z.string().nullish(),
	count: z.number()
});
export type Flag = z.infer<typeof flagSchema>;

export const flagSummarySchema = z.object({
	high: z.number(),
	medium: z.number(),
	low: z.number(),
	total: z.number(),
	days: z.number()
});
export type FlagSummary = z.infer<typeof flagSummarySchema>;

export const flagsResponseSchema = z.object({
	summary: flagSummarySchema,
	flags: z.array(flagSchema)
});
export type FlagsResponse = z.infer<typeof flagsResponseSchema>;

export interface FlagFilter {
	/** How far back to look. The server clamps this to 90 days. */
	days?: number;
	severity?: Severity;
	kind?: string;
}

/**
 * Everything worth a second look. The narrowing happens on the server, while
 * the summary always counts the whole window so the filters can show what they
 * would reveal.
 */
export function listFlags(filter: FlagFilter = {}): Promise<FlagsResponse> {
	const params = new URLSearchParams();
	if (filter.days !== undefined) params.set('days', String(filter.days));
	if (filter.severity) params.set('severity', filter.severity);
	if (filter.kind) params.set('kind', filter.kind);
	const query = params.size > 0 ? `?${params}` : '';
	return api.get(`/flags${query}`, flagsResponseSchema);
}
