<script lang="ts">
	import { onMount } from 'svelte';
	import DoorOpenIcon from '@lucide/svelte/icons/door-open';
	import PackageIcon from '@lucide/svelte/icons/package';
	import WifiIcon from '@lucide/svelte/icons/wifi';
	import WifiOffIcon from '@lucide/svelte/icons/wifi-off';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import HistoryIcon from '@lucide/svelte/icons/history';
	import PlugZapIcon from '@lucide/svelte/icons/plug-zap';
	import SignalIcon from '@lucide/svelte/icons/signal';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import * as Card from '$lib/components/ui/card';
	import * as Alert from '$lib/components/ui/alert';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Empty from '$lib/components/ui/empty';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { listDevices, listConnections } from '$lib/api/devices';
	import {
		formatClock,
		formatDate,
		formatDateTime,
		formatDuration,
		formatRelative
	} from '$lib/format';
	import type { Device, DeviceConnection } from '$lib/schemas/device';

	// A node heartbeats every 30 seconds, so two missed beats reads as offline.
	const ONLINE_WINDOW_MS = 90_000;

	let devices = $state<Device[]>([]);
	let loading = $state(true);
	let nowMs = $state(Date.now());
	let refreshing = $state(false);
	let historyFor = $state<Device | null>(null);
	let connections = $state<DeviceConnection[]>([]);
	let historyLoading = $state(false);

	const kindLabels: Record<string, string> = { door: 'Door', box: 'Box' };

	function isOnline(device: Device): boolean {
		if (!device.last_seen_at) return false;
		return nowMs - new Date(device.last_seen_at).getTime() < ONLINE_WINDOW_MS;
	}

	/** How long the node has been up, in the current stretch only. */
	function uptime(device: Device): string | null {
		if (!device.connected_since) return null;
		return formatDuration((nowMs - new Date(device.connected_since).getTime()) / 1000);
	}

	/** The quiet gap between one stretch and the next, newest-first ordering. */
	function gapBefore(index: number): string | null {
		const later = connections[index];
		const earlier = connections[index + 1];
		if (!later || !earlier) return null;
		const seconds =
			(new Date(later.connected_at).getTime() - new Date(earlier.last_seen_at).getTime()) / 1000;
		if (seconds < 1) return null;
		return formatDuration(seconds);
	}

	// The prototype is one door node plus one cabinet node. Anything missing from
	// that pair is as much a problem as a node that has gone quiet.
	const doorNodes = $derived(devices.filter((d) => d.kind === 'door'));
	const boxNodes = $derived(devices.filter((d) => d.kind === 'box'));
	const onlineCount = $derived(devices.filter(isOnline).length);

	const health = $derived.by(() => {
		if (devices.length === 0) {
			return {
				tone: 'warn' as const,
				title: 'No readers registered',
				text: 'Add a door node and a cabinet node, then flash their secrets into the firmware.'
			};
		}
		if (doorNodes.length === 0 || boxNodes.length === 0) {
			const missing = doorNodes.length === 0 ? 'door' : 'cabinet';
			return {
				tone: 'warn' as const,
				title: `No ${missing} reader registered`,
				text: 'The door-then-cabinet sequence needs both halves before it can grant anything.'
			};
		}
		if (onlineCount === devices.length) {
			return {
				tone: 'ok' as const,
				title: 'Both readers online',
				text: 'The door and the cabinet are both reporting in. The sequence can run end to end.'
			};
		}
		if (onlineCount === 0) {
			return {
				tone: 'bad' as const,
				title: 'No readers online',
				text: 'Neither node has sent a heartbeat recently. Check power and Wi-Fi at the lab.'
			};
		}
		const offline = devices.filter((d) => !isOnline(d)).map((d) => d.name);
		return {
			tone: 'bad' as const,
			title: `Only ${onlineCount} of ${devices.length} readers online`,
			text: `Offline: ${offline.join(', ')}. Anyone at that reader will be turned away.`
		};
	});

	async function refresh(): Promise<void> {
		try {
			devices = await listDevices();
		} catch {
			// keep the current list; a transient failure should not blank the page
		} finally {
			loading = false;
		}
	}

	async function manualRefresh(): Promise<void> {
		refreshing = true;
		nowMs = Date.now();
		await refresh();
		refreshing = false;
	}

	async function openHistory(device: Device): Promise<void> {
		historyFor = device;
		connections = [];
		historyLoading = true;
		try {
			connections = await listConnections(device.id);
		} catch {
			// the dialog falls back to its empty state
		} finally {
			historyLoading = false;
		}
	}

	onMount(() => {
		void refresh();
		const tick = setInterval(() => (nowMs = Date.now()), 5_000);
		const poll = setInterval(() => void refresh(), 15_000);
		return () => {
			clearInterval(tick);
			clearInterval(poll);
		};
	});
</script>

<svelte:head><title>Device Status - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
		<!-- is the pair up? -->
		{#if loading}
			<Skeleton class="h-20 w-full rounded-xl" />
		{:else}
			<Alert.Root variant={health.tone === 'ok' ? 'default' : 'destructive'}>
				{#if health.tone === 'ok'}
					<CircleCheckIcon />
				{:else}
					<TriangleAlertIcon />
				{/if}
				<Alert.Title>{health.title}</Alert.Title>
				<Alert.Description>{health.text}</Alert.Description>
			</Alert.Root>
		{/if}

		<div class="grid gap-4 md:grid-cols-2">
			{#if loading}
				{#each Array.from({ length: 2 }) as _, i (i)}
					<Card.Root>
						<Card.Content class="flex flex-col gap-4 py-6">
							<Skeleton class="h-5 w-28" />
							<Skeleton class="h-8 w-24" />
							<Skeleton class="h-3 w-40" />
						</Card.Content>
					</Card.Root>
				{/each}
			{:else}
				{#each devices as device (device.id)}
					{@const online = isOnline(device)}
					<Card.Root>
						<Card.Content class="flex flex-col gap-4 py-6">
							<div class="flex items-start justify-between gap-3">
								<div class="flex items-center gap-3">
									<span
										class="bg-muted text-muted-foreground flex size-10 shrink-0 items-center justify-center rounded-lg"
									>
										{#if device.kind === 'door'}
											<DoorOpenIcon class="size-5" />
										{:else}
											<PackageIcon class="size-5" />
										{/if}
									</span>
									<div class="flex flex-col">
										<span class="font-medium">{device.name}</span>
										<span class="text-muted-foreground text-xs">
											{kindLabels[device.kind]} reader
										</span>
									</div>
								</div>
								<Badge variant={online ? 'secondary' : 'outline'} class="shrink-0 gap-1.5">
									{#if online}
										<WifiIcon class="size-3 text-emerald-600 dark:text-emerald-400" />
										Online
									{:else}
										<WifiOffIcon class="text-muted-foreground size-3" />
										Offline
									{/if}
								</Badge>
							</div>

							<div class="flex flex-col gap-1">
								<div class="flex items-baseline gap-2">
									<span
										class="size-2.5 rounded-full {online
											? 'bg-emerald-500'
											: 'bg-muted-foreground/40'}"
									></span>
									<span class="text-lg font-medium">
										{#if online}
											{@const up = uptime(device)}
											{up ? `Connected ${up}` : 'Connected'}
										{:else if device.last_seen_at}
											Offline, last seen {formatRelative(device.last_seen_at, nowMs)}
										{:else}
											Never seen
										{/if}
									</span>
								</div>
								{#if online && device.connected_since}
									<span class="text-muted-foreground pl-4.5 text-xs">
										Unbroken since {formatDateTime(device.connected_since)}
									</span>
								{/if}
							</div>

							<div class="text-muted-foreground flex flex-col gap-1 text-xs">
								<span>
									Last heartbeat: {device.last_seen_at
										? formatDateTime(device.last_seen_at)
										: 'none received'}
								</span>
								<span class="flex items-center gap-1.5">
									<SignalIcon class="size-3" />
									{device.sms_capable ? 'Carries the SIM800L' : 'No GSM modem'}
								</span>
								<span class="font-mono">{device.id}</span>
							</div>

							<Button variant="outline" size="sm" class="w-fit" onclick={() => openHistory(device)}>
								<HistoryIcon />
								Connection history
							</Button>
						</Card.Content>
					</Card.Root>
				{/each}
			{/if}
		</div>

		<div class="flex items-center gap-3">
			<Button variant="outline" size="sm" onclick={manualRefresh} disabled={refreshing}>
				{#if refreshing}
					<Loader2Icon class="animate-spin" />
				{:else}
					<RefreshCwIcon />
				{/if}
				Check now
			</Button>
			<span class="text-muted-foreground text-xs">
				Refreshes on its own every 15 seconds. A node counts as online while its heartbeat is
				under 90 seconds old.
			</span>
		</div>
</div>

<Dialog.Root
	open={historyFor !== null}
	onOpenChange={(v) => {
		if (!v) historyFor = null;
	}}
>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Connection history</Dialog.Title>
		</Dialog.Header>

		<Separator class="-mx-4 w-auto" />

		<div class="flex flex-col gap-4">
			<Dialog.Description>
				Each entry is one unbroken stretch of contact with {historyFor?.name ?? 'this node'}. A gap
				between two of them is time the node was not reporting.
			</Dialog.Description>

			{#if historyLoading}
				<div class="flex flex-col gap-3">
					{#each Array.from({ length: 3 }) as _, i (i)}
						<div class="flex flex-col gap-2">
							<Skeleton class="h-4 w-52" />
							<Skeleton class="h-3 w-36" />
						</div>
					{/each}
				</div>
			{:else if connections.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><PlugZapIcon /></Empty.Media>
						<Empty.Title>Nothing recorded yet</Empty.Title>
						<Empty.Description>
							Stretches of uptime appear here once the node starts sending heartbeats.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="flex max-h-96 flex-col gap-3 overflow-y-auto">
					{#each connections as period, i (period.id)}
						{@const gap = gapBefore(i)}
						{#if i > 0}<Separator />{/if}
						<div class="flex items-start justify-between gap-3">
							<div class="flex min-w-0 flex-col gap-0.5">
								<span class="text-sm">
									{formatDate(period.connected_at)}, {formatClock(period.connected_at)}
									{#if period.current}
										to now
									{:else}
										to {formatClock(period.last_seen_at)}
									{/if}
								</span>
								<span class="text-muted-foreground text-xs">
									Up {formatDuration(
										period.current
											? (nowMs - new Date(period.connected_at).getTime()) / 1000
											: period.seconds
									)}
									-
									{period.heartbeats}
									{period.heartbeats === 1 ? 'beat' : 'beats'}
								</span>
							</div>
							<Badge variant={period.current ? 'secondary' : 'outline'} class="shrink-0">
								{period.current ? 'Current' : 'Ended'}
							</Badge>
						</div>
						{#if gap}
							<div class="text-muted-foreground flex items-center gap-2 text-xs">
								<WifiOffIcon class="size-3" />
								Offline for {gap} before this
							</div>
						{/if}
					{/each}
				</div>
			{/if}
		</div>

		<Dialog.Footer>
			<Button onclick={() => (historyFor = null)}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
