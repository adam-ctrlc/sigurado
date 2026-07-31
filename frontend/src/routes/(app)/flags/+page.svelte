<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page as nav } from '$app/state';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import UserIcon from '@lucide/svelte/icons/user';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import ScrollTextIcon from '@lucide/svelte/icons/scroll-text';
	import * as Card from '$lib/components/ui/card';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Select from '$lib/components/ui/select';
	import * as Empty from '$lib/components/ui/empty';
	import * as Accordion from '$lib/components/ui/accordion';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { auth } from '$lib/stores/auth.svelte';
	import { listFlags, type Flag, type FlagSummary, type Severity } from '$lib/api/flags';
	import { SEVERITY_CLASS, SEVERITY_LABELS, SEVERITY_RING, flagKind } from '$lib/flags';
	import { formatClock, formatDate, formatRelative } from '$lib/format';

	type Window = '7' | '14' | '30' | '90';
	type Filter = Severity | 'all';

	const windows: { value: Window; label: string }[] = [
		{ value: '7', label: 'Last 7 days' },
		{ value: '14', label: 'Last 14 days' },
		{ value: '30', label: 'Last 30 days' },
		{ value: '90', label: 'Last 90 days' }
	];

	function windowLabel(value: string): string {
		return windows.find((w) => w.value === value)?.label ?? 'Last 14 days';
	}

	// The query string carries the filters, so a view like ?severity=high&days=30
	// can be sent to somebody and the back button behaves.
	const params = $derived(nav.url.searchParams);
	const days = $derived<Window>((params.get('days') as Window | null) ?? '14');
	const severity = $derived<Filter>((params.get('severity') as Filter | null) ?? 'all');

	let flags = $state<Flag[]>([]);
	let summary = $state<FlagSummary>({ high: 0, medium: 0, low: 0, total: 0, days: 14 });
	let loading = $state(true);
	let failed = $state(false);
	let nowMs = $state(Date.now());

	const filters = $derived<{ value: Filter; label: string; count: number }[]>([
		{ value: 'all', label: 'Everything', count: summary.total },
		{ value: 'high', label: 'Needs attention', count: summary.high },
		{ value: 'medium', label: 'Worth a look', count: summary.medium },
		{ value: 'low', label: 'Information', count: summary.low }
	]);

	/**
	 * Occurrences of the same check collapse into one group, so four forgotten
	 * checkouts read as one thing to chase rather than four unrelated alarms.
	 */
	interface Group {
		kind: string;
		severity: Severity;
		/** Never empty: a group only exists because a flag created it. */
		occurrences: [Flag, ...Flag[]];
		total: number;
	}

	const groups = $derived.by<Group[]>(() => {
		const byKind = new Map<string, Group>();
		for (const flag of flags) {
			const existing = byKind.get(flag.kind);
			if (existing) {
				existing.occurrences.push(flag);
				existing.total += flag.count;
			} else {
				byKind.set(flag.kind, {
					kind: flag.kind,
					severity: flag.severity,
					occurrences: [flag],
					total: flag.count
				});
			}
		}
		const rank: Record<Severity, number> = { high: 0, medium: 1, low: 2 };
		return [...byKind.values()].sort(
			(a, b) => rank[a.severity] - rank[b.severity] || b.total - a.total
		);
	});

	/** Groups start open when they matter, so nothing urgent hides behind a click. */
	const openGroups = $derived(
		groups.filter((g) => g.severity === 'high').map((g) => `${g.kind}`)
	);

	async function setParams(next: Record<string, string | undefined>): Promise<void> {
		const url = new URL(nav.url);
		for (const [key, value] of Object.entries(next)) {
			const empty = value === undefined || value.length === 0 || value === 'all';
			if (empty || (key === 'days' && value === '14')) url.searchParams.delete(key);
			else url.searchParams.set(key, value);
		}
		await goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	async function load(): Promise<void> {
		loading = true;
		failed = false;
		try {
			// The server narrows and sorts; this only groups what comes back.
			const result = await listFlags({
				days: Number(days),
				severity: severity === 'all' ? undefined : severity
			});
			flags = result.flags;
			summary = result.summary;
		} catch {
			flags = [];
			failed = true;
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		if (!auth.isStaff) {
			void goto('/dashboard');
			return;
		}
		const tick = setInterval(() => (nowMs = Date.now()), 30_000);
		return () => clearInterval(tick);
	});

	$effect(() => {
		void [days, severity];
		void load();
	});
</script>

<svelte:head><title>Flags - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div class="flex flex-wrap items-start justify-between gap-4">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Flags</h1>
			<p class="text-muted-foreground text-sm">
				Awkward questions the audit trail raises on its own. Nothing here blocks anybody: a flag is
				a prompt to go and ask.
			</p>
		</div>
		<div class="flex items-center gap-2">
			<Select.Root
				type="single"
				value={days}
				onValueChange={(value) => void setParams({ days: value })}
			>
				<Select.Trigger class="w-40">{windowLabel(days)}</Select.Trigger>
				<Select.Content>
					<Select.Group>
						{#each windows as option (option.value)}
							<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
						{/each}
					</Select.Group>
				</Select.Content>
			</Select.Root>
			<Button variant="outline" size="icon" onclick={() => void load()} aria-label="Refresh flags">
				<RefreshCwIcon />
			</Button>
		</div>
	</div>

	<!-- how bad is it -->
	<Card.Root>
		<Card.Content class="grid grid-cols-2 gap-6 py-6 md:grid-cols-4">
			{#each filters as stat (stat.value)}
				<div class="flex flex-col gap-1">
					<span class="text-muted-foreground text-xs tracking-wide uppercase">{stat.label}</span>
					{#if loading}
						<Skeleton class="h-8 w-12" />
					{:else}
						<span class="text-3xl font-semibold tabular-nums">{stat.count}</span>
					{/if}
				</div>
			{/each}
		</Card.Content>
	</Card.Root>

	<Card.Root>
		<Card.Header>
			<Card.Title>What the trail noticed</Card.Title>
			<Card.Description>
				Grouped by check, worst first, over the {windowLabel(days).toLowerCase()}.
			</Card.Description>
		</Card.Header>

		<Card.Content class="flex flex-col gap-4">
			<Tabs.Root value={severity} onValueChange={(value) => void setParams({ severity: value })}>
				<Tabs.List>
					{#each filters as filter (filter.value)}
						<Tabs.Trigger value={filter.value}>{filter.label}</Tabs.Trigger>
					{/each}
				</Tabs.List>
			</Tabs.Root>

			{#if loading}
				<div class="flex flex-col gap-4 py-2">
					{#each Array.from({ length: 4 }) as _, i (i)}
						<div class="flex items-center gap-3">
							<Skeleton class="size-9 rounded-lg" />
							<Skeleton class="h-4 flex-1" />
							<Skeleton class="h-4 w-16" />
						</div>
					{/each}
				</div>
			{:else if failed}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon">
							<TriangleAlertIcon />
						</Empty.Media>
						<Empty.Title>Could not read the flags</Empty.Title>
						<Empty.Description>
							The server did not answer. Try again in a moment.
						</Empty.Description>
					</Empty.Header>
					<Empty.Content>
						<Button variant="outline" onclick={() => void load()}>Try again</Button>
					</Empty.Content>
				</Empty.Root>
			{:else if groups.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon">
							<ShieldCheckIcon />
						</Empty.Media>
						<Empty.Title>
							{severity === 'all'
								? 'Nothing looks out of place'
								: 'Nothing at that level right now'}
						</Empty.Title>
						<Empty.Description>
							{severity === 'all'
								? 'Every opening in this window has a record behind it, and no reader reported anything strange.'
								: 'Switch to Everything to see the quieter findings.'}
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<Accordion.Root type="multiple" value={openGroups} class="w-full">
					{#each groups as group (group.kind)}
						{@const info = flagKind(group.kind)}
						{@const Icon = info.icon}
						<Accordion.Item value={group.kind}>
							<!-- gap-3 keeps the count clear of the chevron the trigger adds itself -->
							<Accordion.Trigger class="items-center gap-3 hover:no-underline">
								<span
									class="flex size-9 shrink-0 items-center justify-center rounded-lg border {SEVERITY_RING[
										group.severity
									]}"
								>
									<Icon class="size-4" />
								</span>
								<span class="flex min-w-0 flex-1 flex-col gap-1 text-left">
									<span class="truncate text-sm font-medium">{info.label}</span>
									<span class="text-muted-foreground text-xs">{info.watches}</span>
								</span>
								<span class="flex shrink-0 items-center gap-2">
									<span class="text-muted-foreground hidden text-xs sm:inline">
										{SEVERITY_LABELS[group.severity]}
									</span>
									<!-- size-* with no padding keeps the count a circle, not a pill -->
									<Badge
										class="{SEVERITY_CLASS[
											group.severity
										]} size-6 rounded-full p-0 text-xs tabular-nums"
									>
										{group.total}
									</Badge>
								</span>
							</Accordion.Trigger>
							<Accordion.Content>
								<div class="flex flex-col gap-3 pb-2">
									<p class="text-muted-foreground text-sm">{group.occurrences[0].detail}</p>
									<ul class="flex flex-col gap-2">
										{#each group.occurrences as flag, i (`${flag.kind}-${flag.at}-${i}`)}
											<li class="rounded-lg border p-3">
												<div class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
													<span class="text-sm font-medium">{flag.title}</span>
													<span class="text-muted-foreground font-mono text-xs">
														{formatClock(flag.at)} - {formatDate(flag.at)}
													</span>
												</div>
												<div
													class="text-muted-foreground mt-1.5 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs"
												>
													<span>{formatRelative(flag.at, nowMs)}</span>
													{#if flag.person}
														<span class="flex items-center gap-1.5">
															<UserIcon class="size-3" />
															{flag.person}
														</span>
													{/if}
													{#if flag.device}
														<span class="flex items-center gap-1.5">
															<CpuIcon class="size-3" />
															{flag.device}
														</span>
													{/if}
													{#if flag.count > 1}
														<span>{flag.count} occurrences</span>
													{/if}
													{#if flag.person}
														<a
															class="hover:text-foreground flex items-center gap-1.5 underline underline-offset-4"
															href="/logs?q={encodeURIComponent(flag.person)}"
														>
															<ScrollTextIcon class="size-3" />
															See their trail
														</a>
													{/if}
												</div>
											</li>
										{/each}
									</ul>
								</div>
							</Accordion.Content>
						</Accordion.Item>
					{/each}
				</Accordion.Root>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
