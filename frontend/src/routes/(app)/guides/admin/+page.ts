import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import { auth } from '$lib/stores/auth.svelte';
import type { PageLoad } from './$types';

/** Administrator operations only, so anyone else lands on their own guide. */
export const load: PageLoad = async ({ parent }) => {
	if (!browser) return;
	await parent();
	if (!auth.isAdmin) redirect(307, auth.isStaff ? '/guides/faculty' : '/guides/students');
};
