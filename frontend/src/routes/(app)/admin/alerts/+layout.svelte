<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import TabNav from '$lib/components/tab-nav.svelte';
	import { auth } from '$lib/stores/auth.svelte';

	let { children }: { children: Snippet } = $props();

	const tabs = [
		{ href: '/admin/alerts/recipients', label: 'Recipients' },
		{ href: '/admin/alerts/outbox', label: 'Outbox' }
	];

	onMount(() => {
		if (!auth.isAdmin) void goto('/dashboard');
	});
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">SMS alerts</h1>
		<p class="text-muted-foreground text-sm">
			Text messages sent by the SIM800L on the cabinet node when the readers decide something.
		</p>
	</div>

	<TabNav items={tabs} />

	{@render children()}
</div>
