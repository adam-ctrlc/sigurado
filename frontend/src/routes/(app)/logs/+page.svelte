<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page as nav } from '$app/state';
	import SearchIcon from '@lucide/svelte/icons/search';
	import ActivityIcon from '@lucide/svelte/icons/activity';
	import DoorOpenIcon from '@lucide/svelte/icons/door-open';
	import PackageIcon from '@lucide/svelte/icons/package';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';
	import * as Card from '$lib/components/ui/card';
	import * as Tabs from '$lib/components/ui/tabs';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import * as Empty from '$lib/components/ui/empty';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Avatar from '$lib/components/ui/avatar';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { auth } from '$lib/stores/auth.svelte';
	import TablePager from '$lib/components/table-pager.svelte';
	import { listLogs, streamLogs, type LogFilter, type LogStream } from '$lib/api/logs';
	import {
		DECISION_CLASS,
		EVENT_LABELS,
		describeEvent,
		eventPerson,
		eventStage,
		fingerLabel
	} from '$lib/events';
	import { formatClock, formatDate, formatPrecise, formatRelative } from '$lib/format';
	import type { AccessEvent, AccessEventType, EventDecision } from '$lib/schemas/event';

	type Filter = EventDecision | 'all';

	const filters: { value: Filter; label: string }[] = [
		{ value: 'all', label: 'All' },
		{ value: 'granted', label: 'Granted' },
		{ value: 'denied', label: 'Denied' },
		{ value: 'info', label: 'Info' }
	];

	const typeOptions: { value: AccessEventType | 'all'; label: string }[] = [
		{ value: 'all', label: 'Every event' },
		...(Object.keys(EVENT_LABELS) as AccessEventType[]).map((value) => ({
			value,
			label: EVENT_LABELS[value]
		}))
	];

	const typeLabels: Record<string, string> = { all: 'Every event', ...EVENT_LABELS };

	const STAGE_ICONS = {
		Door: DoorOpenIcon,
		Cabinet: PackageIcon,
		Enrollment: FingerprintIcon,
		Checkout: ClipboardListIcon
	} as const;

	// The query string is the single source of truth for the filters, so a view
	// like ?decision=denied&q=juan is shareable and the back button works.
	const params = $derived(nav.url.searchParams);
	const selected = $derived<Filter>((params.get('decision') as Filter | null) ?? 'all');
	const typeFilter = $derived<AccessEventType | 'all'>(
		(params.get('event_type') as AccessEventType | null) ?? 'all'
	);
	const urlQuery = $derived(params.get('q') ?? '');
	const currentPage = $derived(Math.max(1, Number(params.get('page') ?? '1') || 1));

	let events = $state<AccessEvent[]>([]);
	let loading = $state(true);
	let inspecting = $state<AccessEvent | null>(null);
	let nowMs = $state(Date.now());
	let stream: LogStream | null = null;
	let total = $state(0);
	let counts = $state({ all: 0, granted: 0, denied: 0, info: 0 });
	const PER_PAGE = 25;

	// The search box is typed into, so it keeps its own copy and pushes to the URL
	// after a pause. `pushed` guards against a back-navigation clobbering a word
	// halfway through being typed.
	const initialQuery = nav.url.searchParams.get('q') ?? '';
	let query = $state(initialQuery);
	let pushed = $state(initialQuery);
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		if (urlQuery !== pushed) {
			query = urlQuery;
			pushed = urlQuery;
		}
	});

	/** Writes filters into the URL. Absent beats "all" so links stay tidy. */
	async function setParams(next: Record<string, string | undefined>): Promise<void> {
		const url = new URL(nav.url);
		for (const [key, value] of Object.entries(next)) {
			const empty = value === undefined || value.length === 0 || value === 'all';
			if (empty || (key === 'page' && value === '1')) url.searchParams.delete(key);
			else url.searchParams.set(key, value);
		}
		await goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	// Totals cover the whole trail, not the page on screen, so they come from
	// count-only queries against the same filters the server applies.
	const stats = $derived([
		{ label: 'Total events', value: counts.all },
		{ label: 'Granted', value: counts.granted },
		{ label: 'Denied', value: counts.denied },
		{ label: 'Info', value: counts.info }
	]);

	/** The filters currently in force, as the server understands them. */
	const filter = $derived({
		decision: selected === 'all' ? undefined : selected,
		event_type: typeFilter === 'all' ? undefined : typeFilter,
		q: urlQuery.trim() || undefined
	});

	/** A live event only belongs on screen when it passes the active filters. */
	function passesFilters(event: AccessEvent): boolean {
		if (selected !== 'all' && event.decision !== selected) return false;
		if (typeFilter !== 'all' && event.event_type !== typeFilter) return false;
		// Text search is the database's job, so a search narrows the feed off.
		return urlQuery.trim().length === 0;
	}

	function initials(name: string | null | undefined): string {
		if (!name) return '?';
		return name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('');
	}

	async function load(): Promise<void> {
		loading = true;
		const request: LogFilter = { ...filter, page: currentPage, per_page: PER_PAGE };
		try {
			const result = await listLogs(request);
			events = result.items;
			total = result.total;
		} catch {
			events = [];
			total = 0;
		} finally {
			loading = false;
		}
	}

	async function loadCounts(): Promise<void> {
		const base = { event_type: filter.event_type, q: filter.q, per_page: 1 };
		try {
			const [all, granted, denied, info] = await Promise.all([
				listLogs(base),
				listLogs({ ...base, decision: 'granted' }),
				listLogs({ ...base, decision: 'denied' }),
				listLogs({ ...base, decision: 'info' })
			]);
			counts = {
				all: all.total,
				granted: granted.total,
				denied: denied.total,
				info: info.total
			};
		} catch {
			// leave the previous counts in place
		}
	}

	function choose(value: string): void {
		void setParams({ decision: value, page: '1' });
	}

	function onTypeChange(value: string | undefined): void {
		void setParams({ event_type: value, page: '1' });
	}

	/** Typing should not fire a request per keystroke. */
	function onSearch(): void {
		clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			pushed = query;
			void setParams({ q: query, page: '1' });
		}, 300);
	}

	onMount(() => {
		if (!auth.isStaff) {
			void goto('/dashboard');
			return;
		}
		stream = streamLogs((event) => {
			// Only the first page shows live arrivals: prepending onto page 3 would
			// misrepresent where the row sits in the trail.
			if (currentPage === 1 && passesFilters(event)) {
				events = [event, ...events].slice(0, PER_PAGE);
				total += 1;
				counts.all += 1;
				counts[event.decision] += 1;
			}
		});
		// Keeps the "5 minutes ago" column honest without refetching.
		const tick = setInterval(() => (nowMs = Date.now()), 15_000);
		return () => {
			stream?.close();
			clearInterval(tick);
		};
	});

	// One place that reacts to the URL: whatever the query string says, fetch it.
	$effect(() => {
		// Read them so the effect re-runs when any filter or the page changes.
		void [filter.decision, filter.event_type, filter.q, currentPage];
		void load();
		void loadCounts();
	});
</script>

<svelte:head><title>Audit Log - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Audit log</h1>
		<p class="text-muted-foreground text-sm">
			Every scan, denial, and enrollment, with the identity, reader, and time it happened.
		</p>
	</div>

	<!-- what is in view -->
	<Card.Root>
		<Card.Content class="grid grid-cols-2 gap-6 py-6 md:grid-cols-4">
			{#each stats as stat (stat.label)}
				<div class="flex flex-col gap-1">
					<span class="text-muted-foreground text-xs tracking-wide uppercase">{stat.label}</span>
					{#if loading}
						<Skeleton class="h-8 w-12" />
					{:else}
						<span class="text-3xl font-semibold tabular-nums">{stat.value}</span>
					{/if}
				</div>
			{/each}
		</Card.Content>
	</Card.Root>

	<!-- the trail -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Events</Card.Title>
			<Card.Description>
				Newest first, streamed live. Select a row for the full record.
			</Card.Description>
		</Card.Header>

		<Card.Content class="flex flex-col gap-4">
			<Tabs.Root value={selected} onValueChange={choose}>
				<Tabs.List>
					{#each filters as filter (filter.value)}
						<Tabs.Trigger value={filter.value}>{filter.label}</Tabs.Trigger>
					{/each}
				</Tabs.List>
			</Tabs.Root>

			<div class="flex flex-wrap items-center gap-3">
				<div class="relative min-w-56 flex-1">
					<SearchIcon
						class="text-muted-foreground pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2"
					/>
					<Input
						bind:value={query}
						oninput={onSearch}
						placeholder="Search person, reader, finger, or message"
						aria-label="Search events"
						class="pl-9"
					/>
				</div>
				<Select.Root type="single" value={typeFilter} onValueChange={onTypeChange}>
					<Select.Trigger class="w-full sm:w-64">{typeLabels[typeFilter]}</Select.Trigger>
					<Select.Content>
						<Select.Group>
							{#each typeOptions as option (option.value)}
								<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
							{/each}
						</Select.Group>
					</Select.Content>
				</Select.Root>
			</div>

			{#if loading}
				<div class="flex flex-col gap-4 py-2">
					{#each Array.from({ length: 8 }) as _, i (i)}
						<div class="flex items-center gap-3">
							<Skeleton class="h-4 w-24" />
							<Skeleton class="h-5 w-40" />
							<Skeleton class="h-4 flex-1" />
						</div>
					{/each}
				</div>
			{:else if events.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon">
							{#if events.length === 0}
								<ActivityIcon />
							{:else}
								<SearchIcon />
							{/if}
						</Empty.Media>
						<Empty.Title>
							{total === 0 && !filter.q && !filter.event_type && !filter.decision
							? 'No activity yet'
							: 'Nothing matches those filters'}
						</Empty.Title>
						<Empty.Description>
							{events.length === 0
								? 'Scans, denials and enrollments appear here the moment they happen.'
								: 'Clear the search or pick a different event type.'}
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="overflow-hidden rounded-lg border">
					<Table.Root class="min-w-[62rem]">
						<Table.Header>
							<Table.Row class="hover:bg-transparent">
								<Table.Head class="w-40">Time</Table.Head>
								<Table.Head>Event</Table.Head>
								<Table.Head>Person</Table.Head>
								<Table.Head>Reader</Table.Head>
								<Table.Head>Detail</Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each events as event (event.id)}
								{@const stage = eventStage(event.event_type)}
								{@const StageIcon = STAGE_ICONS[stage]}
								<Table.Row class="cursor-pointer" onclick={() => (inspecting = event)}>
									<Table.Cell>
										<div class="flex flex-col gap-0.5">
											<span class="font-mono text-xs">{formatClock(event.created_at)}</span>
											<span class="text-muted-foreground text-xs">
												{formatDate(event.created_at)}
											</span>
											<span class="text-muted-foreground text-[11px]">
												{formatRelative(event.created_at, nowMs)}
											</span>
										</div>
									</Table.Cell>
									<Table.Cell class="max-w-56 whitespace-normal">
										<div class="flex flex-col gap-1.5">
											<Badge class="{DECISION_CLASS[event.decision]} h-auto w-fit whitespace-normal">
												{EVENT_LABELS[event.event_type]}
											</Badge>
											<span class="text-muted-foreground flex items-center gap-1.5 text-xs">
												<StageIcon class="size-3" />
												{stage}
											</span>
											</div>
									</Table.Cell>
									<Table.Cell>
										{@const person = eventPerson(event)}
										{#if person.name}
											<div class="flex items-center gap-2">
												<Avatar.Root class="size-7">
													<Avatar.Fallback class="text-[10px]">
														{initials(person.name)}
													</Avatar.Fallback>
												</Avatar.Root>
												<div class="flex min-w-0 flex-col">
													<span class="truncate text-sm">{person.name}</span>
													{#if person.retroactive}
														<span class="text-muted-foreground text-[11px]">
															matched later by finger
														</span>
													{/if}
												</div>
											</div>
										{:else}
											<div class="flex flex-col gap-0.5">
												<span class="text-muted-foreground font-mono text-xs">
													{fingerLabel(event)}
												</span>
												<span class="text-muted-foreground text-[11px]">not registered yet</span>
											</div>
										{/if}
									</Table.Cell>
									<Table.Cell>
										{#if event.device_name}
											<span class="flex items-center gap-2 text-sm">
												<CpuIcon class="text-muted-foreground size-3.5" />
												{event.device_name}
											</span>
										{:else}
											<span class="text-muted-foreground text-sm">-</span>
										{/if}
									</Table.Cell>
									<Table.Cell class="max-w-xs">
										<div class="flex flex-col gap-1">
											{#if event.message}
												<span class="text-muted-foreground truncate text-xs">{event.message}</span>
											{/if}
											{#if event.finger_token}
												<span class="text-muted-foreground font-mono text-[11px]">
													{event.finger_token}
												</span>
											{/if}
											{#if !event.message && !event.finger_token}
												<span class="text-muted-foreground text-xs">-</span>
											{/if}
										</div>
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>

				<TablePager
					{total}
					perPage={PER_PAGE}
					page={currentPage}
					noun="events"
					onPage={(next) => void setParams({ page: String(next) })}
				/>
			{/if}
		</Card.Content>
	</Card.Root>
</div>

<Dialog.Root
	open={inspecting !== null}
	onOpenChange={(v) => {
		if (!v) inspecting = null;
	}}
>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>
				{inspecting ? EVENT_LABELS[inspecting.event_type] : 'Event'}
			</Dialog.Title>
		</Dialog.Header>

		<Separator class="-mx-4 w-auto" />

		{#if inspecting}
			<div class="flex max-h-[60vh] flex-col gap-4 overflow-y-auto">
				<div class="flex flex-wrap items-center gap-2">
					<Badge class={DECISION_CLASS[inspecting.decision]}>{inspecting.decision}</Badge>
					<Badge variant="outline">{eventStage(inspecting.event_type)}</Badge>
					<Badge variant="outline" class="font-mono text-[11px]">{inspecting.event_type}</Badge>
				</div>

				<p class="text-muted-foreground text-sm leading-relaxed">
					{describeEvent(inspecting)}
				</p>

				<Separator />

				<dl class="grid gap-3 text-sm sm:grid-cols-[9rem_1fr]">
					<dt class="text-muted-foreground">Timestamp</dt>
					<dd>{formatPrecise(inspecting.created_at)}</dd>

					<dt class="text-muted-foreground">Relative</dt>
					<dd>{formatRelative(inspecting.created_at, nowMs)}</dd>

					<dt class="text-muted-foreground">Person</dt>
					<dd>
						{#if inspecting.user_name}
							{inspecting.user_name}
						{:else if inspecting.finger_owner_name}
							{inspecting.finger_owner_name}
							<span class="text-muted-foreground">(matched later by finger)</span>
						{:else}
							Not identified
						{/if}
					</dd>

					<dt class="text-muted-foreground">Reader</dt>
					<dd>{inspecting.device_name ?? 'Not recorded'}</dd>

					<dt class="text-muted-foreground">Finger token</dt>
					<dd class="font-mono text-xs">{inspecting.finger_token ?? '-'}</dd>

					<dt class="text-muted-foreground">Finger belongs to</dt>
					<dd>
						{#if inspecting.finger_owner_name}
							{inspecting.finger_owner_name}
						{:else if inspecting.user_name}
							{inspecting.user_name}
						{:else}
							Nobody yet: this finger is not enrolled
						{/if}
					</dd>

					<dt class="text-muted-foreground">Message</dt>
					<dd>{inspecting.message ?? '-'}</dd>

					<dt class="text-muted-foreground">Door session</dt>
					<dd class="font-mono text-xs break-all">{inspecting.access_session_id ?? '-'}</dd>

					<dt class="text-muted-foreground">User ID</dt>
					<dd class="font-mono text-xs break-all">
						{inspecting.user_id ?? inspecting.finger_owner_id ?? '-'}
					</dd>

					<dt class="text-muted-foreground">Device ID</dt>
					<dd class="font-mono text-xs break-all">{inspecting.device_id ?? '-'}</dd>

					<dt class="text-muted-foreground">Event ID</dt>
					<dd class="font-mono text-xs break-all">{inspecting.id}</dd>
				</dl>
			</div>
		{/if}

		<Dialog.Footer>
			<Button onclick={() => (inspecting = null)}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
