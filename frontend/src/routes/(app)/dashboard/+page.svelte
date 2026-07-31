<script lang="ts">
	import { auth } from '$lib/stores/auth.svelte';
	import StaffDashboard from './staff-dashboard.svelte';
	import MyDashboard from './my-dashboard.svelte';

	// Faculty and admins watch the room; a student only ever sees their own record,
	// so the two dashboards are genuinely different pages behind one route.
	const staff = $derived(auth.isStaff);
</script>

<svelte:head><title>Dashboard - Sigurado</title></svelte:head>

{#if staff}
	<StaffDashboard />
{:else}
	<MyDashboard />
{/if}
