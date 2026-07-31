<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page as nav } from '$app/state';
	import { toast } from 'svelte-sonner';
	import CloudUploadIcon from '@lucide/svelte/icons/cloud-upload';
	import ExternalLinkIcon from '@lucide/svelte/icons/external-link';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import * as Card from '$lib/components/ui/card';
	import * as Alert from '$lib/components/ui/alert';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { auth } from '$lib/stores/auth.svelte';
	import { sheetIndex, syncGoogle, type SheetIndex } from '$lib/api/sheets';
	import { formatRelative } from '$lib/format';

	let { children }: { children: Snippet } = $props();

	let index = $state<SheetIndex | null>(null);
	let syncing = $state(false);
	let nowMs = $state(Date.now());

	const google = $derived(index?.google ?? null);
	const failure = $derived(google?.tabs.find((t) => t.last_error)?.last_error ?? null);
	const waiting = $derived((google?.events_waiting ?? 0) + (google?.checkouts_waiting ?? 0));
	const lastSynced = $derived(google?.tabs.find((t) => t.last_synced_at)?.last_synced_at ?? null);

	const sheetUrl = $derived(
		google?.spreadsheet_id
			? `https://docs.google.com/spreadsheets/d/${google.spreadsheet_id}`
			: null
	);

	// The tab strip keeps whatever search and page size you are using, the way
	// switching sheets in a spreadsheet keeps your place.
	const carried = $derived.by(() => {
		const keep = new URLSearchParams();
		for (const key of ['q', 'per_page']) {
			const value = nav.url.searchParams.get(key);
			if (value) keep.set(key, value);
		}
		const qs = keep.toString();
		return qs.length > 0 ? `?${qs}` : '';
	});

	function isActive(tab: string): boolean {
		return nav.url.pathname === `/admin/sheets/${tab}`;
	}

	async function load(): Promise<void> {
		try {
			index = await sheetIndex();
		} catch {
			// The tabs are fixed, so a failure here only costs the mirror panel.
		}
	}

	async function pushToGoogle(): Promise<void> {
		syncing = true;
		try {
			const report = await syncGoogle();
			const added = report.events + report.checkouts;
			toast.success(added > 0 ? `Sent ${added} new rows to Google` : 'Google is already up to date');
		} catch {
			toast.error('The push did not finish. The reason is below.');
		} finally {
			syncing = false;
			await load();
		}
	}

	onMount(() => {
		if (!auth.isAdmin) {
			void goto('/dashboard');
			return;
		}
		void load();
		const tick = setInterval(() => (nowMs = Date.now()), 30_000);
		return () => clearInterval(tick);
	});
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Sheet</h1>
		<p class="text-muted-foreground text-sm">
			The whole trail as a spreadsheet, built from what the readers posted here. Nothing to connect
			and nothing to sign in to.
		</p>
	</div>

	{@render children()}

	<!-- The tab strip sits under the grid, where a spreadsheet keeps it. -->
	<div class="-mx-1 max-w-full overflow-x-auto px-1 py-px">
		<nav
			class="bg-muted inline-flex w-fit items-end gap-1 rounded-lg p-1"
			aria-label="Sheet tabs"
		>
			{#each index?.tabs ?? [] as tab (tab.tab)}
				{@const active = isActive(tab.tab)}
				<a
					href="/admin/sheets/{tab.tab}{carried}"
					aria-current={active ? 'page' : undefined}
					class="flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium whitespace-nowrap transition-colors
					{active
						? 'bg-background text-foreground dark:border-input dark:bg-input/30 shadow-sm'
						: 'text-foreground/60 hover:text-foreground dark:text-muted-foreground'}"
				>
					{tab.title}
					{#if !tab.history}
						<span class="text-muted-foreground text-[11px]">derived</span>
					{/if}
				</a>
			{/each}
		</nav>
	</div>

	{#if google}
		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2 text-base">
					<CloudUploadIcon class="size-4" />
					Also mirrored to Google
				</Card.Title>
				<Card.Description>
					Optional, and switched on in the backend settings. The page above does not need it.
				</Card.Description>
			</Card.Header>
			<Card.Content class="flex flex-col gap-4">
				{#if failure}
					<Alert.Root variant="destructive">
						<TriangleAlertIcon />
						<Alert.Title>The last push did not finish</Alert.Title>
						<Alert.Description>{failure}</Alert.Description>
					</Alert.Root>
				{/if}

				<div class="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm">
					<span class="text-muted-foreground">
						Every {google.every_secs} seconds
					</span>
					{#if lastSynced}
						<span class="text-muted-foreground">
							Last push {formatRelative(lastSynced, nowMs)}
						</span>
					{/if}
					{#if waiting > 0}
						<Badge variant="secondary">{waiting} rows still to send</Badge>
					{/if}
					{#each google.tabs as tab (tab.tab)}
						<span class="text-muted-foreground text-xs">
							{tab.tab} {tab.rows_sent}
						</span>
					{/each}
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<Button variant="outline" size="sm" onclick={() => void pushToGoogle()} disabled={syncing}>
						{syncing ? 'Pushing...' : 'Push now'}
					</Button>
					{#if sheetUrl}
						<Button variant="ghost" size="sm" href={sheetUrl} target="_blank" rel="noreferrer">
							<ExternalLinkIcon data-icon="inline-start" />
							Open in Google
						</Button>
					{/if}
				</div>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
