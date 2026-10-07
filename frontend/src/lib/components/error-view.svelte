<script lang="ts">
	import type { Component } from 'svelte';
	import SearchXIcon from '@lucide/svelte/icons/search-x';
	import LockKeyholeIcon from '@lucide/svelte/icons/lock-keyhole';
	import LogInIcon from '@lucide/svelte/icons/log-in';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import HouseIcon from '@lucide/svelte/icons/house';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import { Button } from '$lib/components/ui/button';
	import { auth } from '$lib/stores/auth.svelte';

	let { status, detail }: { status: number; detail?: string | null } = $props();

	type Copy = { icon: Component; title: string; text: string };

	// Keyed by status rather than by the framework's message, which is either
	// "Not Found" or a stack-shaped string a person cannot act on.
	const COPY: Record<number, Copy> = {
		401: {
			icon: LogInIcon,
			title: 'You are signed out',
			text: 'The session ended, or it was never started on this device. Sign in and you can pick up where you left off.'
		},
		403: {
			icon: LockKeyholeIcon,
			title: 'Not yours to open',
			text: 'Your account does not have access to this page. Roles are set by an administrator, so ask one if you think that is wrong.'
		},
		404: {
			icon: SearchXIcon,
			title: 'Nothing at this address',
			text: 'The page you asked for is not here. Check the link, or head back and find it from the menu.'
		}
	};

	const fallback: Copy = {
		icon: TriangleAlertIcon,
		title: 'Something went wrong',
		text: 'The request could not be finished. Try it again, and if it keeps happening the audit log will still hold every reader event from while it was down.'
	};

	const copy = $derived(COPY[status] ?? fallback);

	// "Not Found" repeats the heading; anything more specific is worth showing.
	const GENERIC = ['not found', 'forbidden', 'unauthorized', 'internal error', 'error'];
	const note = $derived(
		detail && !GENERIC.includes(detail.trim().toLowerCase()) ? detail.trim() : null
	);

	const home = $derived(auth.isAuthed ? '/dashboard' : '/');
	const homeLabel = $derived(auth.isAuthed ? 'Back to the dashboard' : 'Back to the start');
</script>

<div class="relative flex flex-col items-center gap-6 px-6 py-20 text-center">
	<span
		class="pointer-events-none absolute -top-24 left-1/2 size-[30rem] -translate-x-1/2 rounded-full opacity-20 blur-3xl [background:radial-gradient(circle,var(--beam)_0%,transparent_70%)]"
		aria-hidden="true"
	></span>

	<span
		class="relative flex size-16 rotate-45 items-center justify-center rounded-[22%] border border-[var(--glass-edge)] bg-primary/10 text-primary"
	>
		{#if copy.icon}
			{@const Icon = copy.icon}
			<Icon class="size-6 -rotate-45" strokeWidth={1.5} />
		{/if}
	</span>

	<div class="relative flex flex-col items-center gap-3">
		<span class="font-display text-6xl leading-none font-semibold tracking-tighter text-primary">
			{status}
		</span>
		<h1 class="font-display text-3xl font-semibold tracking-tight text-balance">{copy.title}</h1>
		<p class="max-w-md text-sm leading-relaxed text-pretty text-muted-foreground">{copy.text}</p>
		{#if note}
			<p class="mt-1 max-w-md rounded-lg border px-3 py-2 font-mono text-xs text-muted-foreground">
				{note}
			</p>
		{/if}
	</div>

	<div class="relative flex flex-wrap items-center justify-center gap-3">
		<Button href={home} class="rounded-full px-5">
			<HouseIcon />
			{homeLabel}
		</Button>
		{#if status === 401}
			<Button href="/login" variant="outline" class="rounded-full px-5">
				<LogInIcon />
				Sign in
			</Button>
		{:else}
			<Button variant="outline" class="rounded-full px-5" onclick={() => history.back()}>
				<ArrowLeftIcon />
				Go back
			</Button>
		{/if}
	</div>
</div>
