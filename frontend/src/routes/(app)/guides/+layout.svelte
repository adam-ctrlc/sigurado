<script lang="ts">
	import type { Snippet } from 'svelte';
	import TabNav from '$lib/components/tab-nav.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { guidesForRole } from '$lib/guides';

	let { children }: { children: Snippet } = $props();

	// Staff can read the student guide as well, so tabs only appear when there is
	// more than one guide to choose between.
	const tabs = $derived(
		guidesForRole(auth.user?.role ?? 'student').map((guide) => ({
			href: `/guides/${guide.slug}`,
			label: guide.label
		}))
	);
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Guides</h1>
		<p class="text-muted-foreground text-sm">
			Everything the system does, numbered, in the order you will need it.
		</p>
	</div>

	{#if tabs.length > 1}
		<TabNav items={tabs} />
	{/if}

	{@render children()}
</div>
