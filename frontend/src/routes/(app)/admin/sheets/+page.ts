import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import type { PageLoad } from './$types';

/** The section has no view of its own; the first tab is the landing spot. */
export const load: PageLoad = async ({ parent }) => {
	if (!browser) return;
	await parent();
	redirect(307, '/admin/sheets/events');
};
