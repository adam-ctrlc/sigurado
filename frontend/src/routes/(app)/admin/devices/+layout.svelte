<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import TabNav from '$lib/components/tab-nav.svelte';
	import { auth } from '$lib/stores/auth.svelte';

	let { children }: { children: Snippet } = $props();

	const tabs = [
		{ href: '/admin/devices/status', label: 'Status' },
		{ href: '/admin/devices/inventory', label: 'Inventory' }
	];

	onMount(() => {
		if (!auth.isAdmin) void goto('/dashboard');
	});
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Devices</h1>
		<p class="text-muted-foreground text-sm">
			The reader nodes at the door and on the cabinet, and the secrets they authenticate with.
		</p>
	</div>

	<TabNav items={tabs} />

	{@render children()}
</div>
