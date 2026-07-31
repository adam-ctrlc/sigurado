<script lang="ts">
	import '@fontsource-variable/inter';
	import '@fontsource-variable/manrope';
	import '@fontsource-variable/geist-mono';
	import './layout.css';
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import favicon from '$lib/assets/favicon.svg';
	import { Toaster } from '$lib/components/ui/sonner';
	import { auth } from '$lib/stores/auth.svelte';
	import { me } from '$lib/api/auth';

	let { children }: { children: Snippet } = $props();

	onMount(async () => {
		// Rehydrate the session: validate the stored token and load the user.
		if (auth.token !== null && auth.user === null) {
			try {
				auth.setUser(await me());
			} catch {
				auth.clear();
			}
		}
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<Toaster position="top-right" richColors />
{@render children()}
