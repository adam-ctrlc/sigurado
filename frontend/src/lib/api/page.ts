import { z } from 'zod';

/**
 * The envelope every searchable list endpoint returns. Filtering, searching and
 * paging all happen in the database, so the client only ever holds one page.
 */
export function pageSchema<T extends z.ZodTypeAny>(item: T) {
	return z.object({
		items: z.array(item),
		total: z.number(),
		page: z.number(),
		per_page: z.number(),
		pages: z.number()
	});
}

export interface Paged<T> {
	items: T[];
	total: number;
	page: number;
	per_page: number;
	pages: number;
}

export interface PageRequest {
	page?: number;
	per_page?: number;
}

/** Builds a query string, dropping empty values so the URL stays readable. */
export function queryString(params: Record<string, string | number | undefined | null>): string {
	const search = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value === undefined || value === null) continue;
		const text = String(value);
		if (text.length === 0) continue;
		search.set(key, text);
	}
	const qs = search.toString();
	return qs.length > 0 ? `?${qs}` : '';
}
