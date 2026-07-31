import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import { auth } from '$lib/stores/auth.svelte';
import type { PageLoad } from './$types';

/** Written for staff, so a student is sent to the guide meant for them. */
export const load: PageLoad = async ({ parent }) => {
	if (!browser) return;
	await parent();
	if (!auth.isStaff) redirect(307, '/guides/students');
};
