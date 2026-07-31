<script lang="ts">
	import { onMount } from 'svelte';
	import RadioIcon from '@lucide/svelte/icons/radio';
	import DoorClosedIcon from '@lucide/svelte/icons/door-closed';
	import DoorOpenIcon from '@lucide/svelte/icons/door-open';
	import ShieldAlertIcon from '@lucide/svelte/icons/shield-alert';
	import ScanLineIcon from '@lucide/svelte/icons/scan-line';
	import PackageIcon from '@lucide/svelte/icons/package';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import { Badge } from '$lib/components/ui/badge';
	import { Progress } from '$lib/components/ui/progress';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import EventTable from '$lib/components/event-table.svelte';
	import { activeSessions } from '$lib/api/sessions';
	import { listCheckouts } from '$lib/api/checkouts';
	import { listLogs, streamLogs, type LogStream } from '$lib/api/logs';
	import type { AccessSession } from '$lib/schemas/session';
	import type { AccessEvent } from '$lib/schemas/event';
	import type { Checkout } from '$lib/schemas/checkout';

	let sessions = $state<AccessSession[]>([]);
	let sessionsLoading = $state(true);
	let events = $state<AccessEvent[]>([]);
	let logsLoading = $state(true);
	let checkouts = $state<Checkout[]>([]);
	let nowMs = $state(Date.now());
	let stream: LogStream | null = null;

	/** The dashboard is a glance, not the trail: the audit log holds everything. */
	const FEED_ROWS = 10;

	function remaining(session: AccessSession): number {
		return Math.max(0, Math.floor((new Date(session.expires_at).getTime() - nowMs) / 1000));
	}

	// The window is fixed when the door grants, so the bar is a share of what was granted.
	function share(session: AccessSession): number {
		const total = new Date(session.expires_at).getTime() - new Date(session.opened_at).getTime();
		if (total <= 0) return 0;
		return Math.min(100, Math.max(0, ((remaining(session) * 1000) / total) * 100));
	}

	function isToday(iso: string): boolean {
		const then = new Date(iso);
		const now = new Date(nowMs);
		return (
			then.getFullYear() === now.getFullYear() &&
			then.getMonth() === now.getMonth() &&
			then.getDate() === now.getDate()
		);
	}

	const scansToday = $derived(
		events.filter((e) => isToday(e.created_at) && e.decision !== 'info').length
	);
	const deniedToday = $derived(
		events.filter((e) => isToday(e.created_at) && e.decision === 'denied').length
	);
	const checkoutsToday = $derived(checkouts.filter((c) => isToday(c.created_at)).length);
	const loading = $derived(sessionsLoading || logsLoading);

	const summary = $derived([
		{
			icon: DoorOpenIcon,
			label: 'Open right now',
			value: sessions.length,
			hint: sessions.length === 0 ? 'Nobody is inside the window' : 'Door sessions awaiting the box'
		},
		{
			icon: ScanLineIcon,
			label: 'Scans today',
			value: scansToday,
			hint: 'Door and cabinet decisions combined'
		},
		{
			icon: ShieldAlertIcon,
			label: 'Denials today',
			value: deniedToday,
			hint:
				scansToday === 0
					? 'Nothing turned away yet'
					: `${Math.round((deniedToday / scansToday) * 100)}% of today's scans`
		},
		{
			icon: PackageIcon,
			label: 'Checkouts today',
			value: checkoutsToday,
			hint: 'Materials logged out of the cabinet'
		}
	]);

	async function refreshSessions(): Promise<void> {
		try {
			sessions = await activeSessions();
		} catch {
			// transient; keep the previous list
		} finally {
			sessionsLoading = false;
		}
	}

	async function init(): Promise<void> {
		try {
			events = (await listLogs({ per_page: 200 })).items;
		} catch {
			// leave empty
		} finally {
			logsLoading = false;
		}
		try {
			checkouts = (await listCheckouts({ per_page: 200 })).items;
		} catch {
			// leave empty
		}
		await refreshSessions();
		stream = streamLogs((event) => {
			events = [event, ...events].slice(0, 200);
			if (event.event_type.startsWith('door') || event.event_type.startsWith('box')) {
				void refreshSessions();
			}
		});
	}

	onMount(() => {
		void init();
		const tick = setInterval(() => (nowMs = Date.now()), 1000);
		const poll = setInterval(() => void refreshSessions(), 5000);
		return () => {
			stream?.close();
			clearInterval(tick);
			clearInterval(poll);
		};
	});
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Dashboard</h1>
		<p class="text-muted-foreground text-sm">
			Who is inside the window right now, and what the readers have decided today.
		</p>
	</div>

	<!-- summary of the day -->
	<div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
		{#each summary as item (item.label)}
			{@const Icon = item.icon}
			<Card.Root>
				<Card.Content class="flex flex-col gap-3 py-5">
					<div class="flex items-center justify-between gap-2">
						<span class="text-muted-foreground text-xs tracking-wide uppercase">{item.label}</span>
						<Icon class="text-muted-foreground size-4" />
					</div>
					{#if loading}
						<Skeleton class="h-9 w-16" />
						<Skeleton class="h-3 w-32" />
					{:else}
						<span class="text-4xl font-semibold tabular-nums">{item.value}</span>
						<span class="text-muted-foreground text-xs">{item.hint}</span>
					{/if}
				</Card.Content>
			</Card.Root>
		{/each}
	</div>

	<!-- open door sessions -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Active door sessions</Card.Title>
			<Card.Description>
				The cabinet unlocks only for the person holding one of these, and only until it expires.
			</Card.Description>
		</Card.Header>

		<Card.Content>
			{#if sessionsLoading}
				<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{#each Array.from({ length: 3 }) as _, i (i)}
						<div class="flex flex-col gap-3 rounded-lg border p-4">
							<Skeleton class="h-4 w-32" />
							<Skeleton class="h-8 w-20" />
							<Skeleton class="h-2 w-full" />
						</div>
					{/each}
				</div>
			{:else if sessions.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><DoorClosedIcon /></Empty.Media>
						<Empty.Title>No active door sessions</Empty.Title>
						<Empty.Description>
							When someone scans at the door, their session appears here with a live countdown.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{#each sessions as session (session.id)}
						<div class="flex flex-col gap-3 rounded-lg border p-4">
							<div class="flex items-start justify-between gap-2">
								<div class="flex min-w-0 flex-col">
									<span class="truncate font-medium">{session.user_name ?? 'Unknown'}</span>
									<span class="text-muted-foreground truncate text-xs">
										at {session.door_device_name ?? 'door'}
									</span>
								</div>
								<Badge variant="secondary" class="shrink-0 gap-1.5">
									<RadioIcon class="size-3 animate-pulse text-emerald-600 dark:text-emerald-400" />
									Open
								</Badge>
							</div>
							<div class="flex items-baseline gap-1.5">
								<span class="text-primary font-mono text-3xl tabular-nums">
									{remaining(session)}
								</span>
								<span class="text-muted-foreground text-xs">seconds left</span>
							</div>
							<Progress value={share(session)} class="h-1.5" />
						</div>
					{/each}
				</div>
			{/if}
		</Card.Content>
	</Card.Root>

	<!-- live activity -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Live activity</Card.Title>
			<Card.Description>The most recent decisions from both readers.</Card.Description>
		</Card.Header>
		<Card.Content>
			<EventTable events={events.slice(0, FEED_ROWS)} loading={logsLoading} rows={FEED_ROWS} />
		</Card.Content>
	</Card.Root>
</div>
