import { redirect } from '@sveltejs/kit';

/** The bare section has no view of its own; the first tab is the landing spot. */
export function load(): never {
	throw redirect(307, '/admin/alerts/recipients');
}
