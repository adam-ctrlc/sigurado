import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import { auth } from '$lib/stores/auth.svelte';
import { guideForRole } from '$lib/guides';
import type { PageLoad } from './$types';

/** No view of its own: land on the guide written for whoever is signed in. */
export const load: PageLoad = async ({ parent }) => {
	if (!browser) return;
	// Loads run in parallel, so the role is only known once the app layout has
	// finished fetching the signed-in person.
	await parent();
	redirect(307, `/guides/${guideForRole(auth.user?.role ?? 'student').slug}`);
};
