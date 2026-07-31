import { redirect } from '@sveltejs/kit';
import { browser } from '$app/environment';
import { auth } from '$lib/stores/auth.svelte';
import { me } from '$lib/api/auth';
import type { LayoutLoad } from './$types';

// Authenticated area renders client-side: the bearer token lives in the browser.
export const ssr = false;

export const load: LayoutLoad = async () => {
	if (!browser) return {};

	if (!auth.isAuthed) {
		redirect(302, '/login');
	}

	if (auth.user === null) {
		try {
			auth.setUser(await me());
		} catch {
			auth.clear();
			redirect(302, '/login');
		}
	}

	return {};
};
