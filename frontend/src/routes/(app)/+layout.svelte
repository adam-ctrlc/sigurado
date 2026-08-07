<script lang="ts">
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import { Separator } from '$lib/components/ui/separator';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import ThemeToggle from '$lib/components/theme-toggle.svelte';
	import EnrollmentGate from '$lib/components/enrollment-gate.svelte';

	let { children }: { children: Snippet } = $props();

	const titles: Record<string, string> = {
		'/dashboard': 'Dashboard',
		'/logs': 'Audit Log',
		'/enrollment': 'Enrollment',
		'/checkout': 'Checkout',
		'/admin/users': 'Users',
		'/admin/devices': 'Devices',
		'/admin/devices/status': 'Devices',
		'/admin/devices/inventory': 'Devices',
		'/admin/alerts': 'SMS Alerts',
		'/admin/alerts/recipients': 'SMS Alerts',
		'/admin/alerts/outbox': 'SMS Alerts',
		'/profile': 'Profile'
	};

	const title = $derived(titles[page.url.pathname] ?? 'Sigurado');
</script>

<!-- Held over everything until a finger is bound. Renders nothing otherwise. -->
<EnrollmentGate />

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset>
		<header class="flex h-16 shrink-0 items-center gap-2 border-b px-4">
			<Sidebar.Trigger class="-ml-1" />
			<Separator orientation="vertical" class="mr-2 h-4" />
			<h1 class="font-display text-base font-semibold">{title}</h1>
			<div class="ml-auto">
				<ThemeToggle />
			</div>
		</header>
		<div class="flex flex-1 flex-col gap-4 p-6">
			{@render children()}
		</div>
	</Sidebar.Inset>
</Sidebar.Provider>
