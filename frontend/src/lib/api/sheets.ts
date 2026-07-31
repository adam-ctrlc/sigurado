import { z } from 'zod';
import { api, API_BASE } from '$lib/api/client';
import { pageSchema, queryString, type Paged, type PageRequest } from '$lib/api/page';

export const tabSchema = z.enum(['events', 'checkouts', 'flags', 'daily']);
export type SheetTab = z.infer<typeof tabSchema>;

export const tabInfoSchema = z.object({
	tab: tabSchema,
	title: z.string(),
	summary: z.string(),
	columns: z.number(),
	/** True for a growing log, false for a derived summary. */
	history: z.boolean()
});
export type TabInfo = z.infer<typeof tabInfoSchema>;

export const googleTabStatusSchema = z.object({
	tab: z.string(),
	rows_sent: z.number(),
	last_synced_at: z.string().nullish(),
	last_error: z.string().nullish()
});

export const googleStatusSchema = z.object({
	spreadsheet_id: z.string(),
	every_secs: z.number(),
	tabs: z.array(googleTabStatusSchema),
	events_waiting: z.number(),
	checkouts_waiting: z.number()
});
export type GoogleStatus = z.infer<typeof googleStatusSchema>;

export const sheetIndexSchema = z.object({
	tabs: z.array(tabInfoSchema),
	/** Only present when the optional Google mirror is switched on. */
	google: googleStatusSchema.nullish()
});
export type SheetIndex = z.infer<typeof sheetIndexSchema>;

/** A row is just cells, in the order the headers give. */
const rowSchema = z.array(z.string());

export const sheetPageSchema = z.object({
	tab: tabSchema,
	title: z.string(),
	summary: z.string(),
	headers: z.array(z.string()),
	rows: pageSchema(rowSchema)
});
export type SheetPage = z.infer<typeof sheetPageSchema>;
export type SheetRows = Paged<string[]>;

export interface SheetFilter extends PageRequest {
	/** Free text over every column, matched by the server. */
	q?: string;
}

export function sheetIndex(): Promise<SheetIndex> {
	return api.get('/admin/sheet', sheetIndexSchema);
}

export function sheetPage(tab: SheetTab, filter: SheetFilter = {}): Promise<SheetPage> {
	return api.get(`/admin/sheet/${tab}${queryString({ ...filter })}`, sheetPageSchema);
}

/**
 * The download link for a tab. A plain href would miss the bearer token, so the
 * caller fetches it and hands the browser a blob instead.
 */
export function csvUrl(tab: SheetTab, filter: SheetFilter = {}): string {
	return `${API_BASE}/admin/sheet/${tab}/csv${queryString({ q: filter.q })}`;
}

export const syncReportSchema = z.object({
	events: z.number(),
	checkouts: z.number(),
	flags: z.number(),
	daily: z.number()
});
export type SyncReport = z.infer<typeof syncReportSchema>;

/** Pushes the optional Google mirror now. */
export function syncGoogle(): Promise<SyncReport> {
	return api.post('/admin/sheets/sync', syncReportSchema);
}
