<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import CopyIcon from '@lucide/svelte/icons/copy';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import DoorOpenIcon from '@lucide/svelte/icons/door-open';
	import PackageIcon from '@lucide/svelte/icons/package';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
	import SignalIcon from '@lucide/svelte/icons/signal';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Field from '$lib/components/ui/field';
	import * as Select from '$lib/components/ui/select';
	import * as Empty from '$lib/components/ui/empty';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { listDevices, createDevice, rotateSecret, updateDevice } from '$lib/api/devices';
	import { ApiError } from '$lib/api/client';
	import { formatDateTime } from '$lib/format';
	import type { Device, DeviceKind, DeviceWithSecret } from '$lib/schemas/device';

	// A node heartbeats every 30 seconds, so two missed beats reads as offline.
	const ONLINE_WINDOW_MS = 90_000;

	let devices = $state<Device[]>([]);
	let loading = $state(true);
	let createOpen = $state(false);
	let submitting = $state(false);
	let name = $state('');
	let kind = $state<DeviceKind>('door');
	let revealed = $state<DeviceWithSecret | null>(null);
	let nowMs = $state(Date.now());

	const kindLabels: Record<string, string> = { door: 'Door', box: 'Box' };

	function isOnline(device: Device): boolean {
		if (!device.last_seen_at) return false;
		return nowMs - new Date(device.last_seen_at).getTime() < ONLINE_WINDOW_MS;
	}

	const stats = $derived([
		{ label: 'Nodes', value: devices.length },
		{ label: 'Door readers', value: devices.filter((d) => d.kind === 'door').length },
		{ label: 'Box readers', value: devices.filter((d) => d.kind === 'box').length },
		{ label: 'Online', value: devices.filter(isOnline).length }
	]);

	async function refresh(): Promise<void> {
		try {
			devices = await listDevices();
		} catch {
			// keep the current list; a transient failure should not blank the page
		} finally {
			loading = false;
		}
	}

	async function create(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (submitting || name.trim().length === 0) return;
		submitting = true;
		try {
			const created = await createDevice(name.trim(), kind);
			revealed = created;
			name = '';
			kind = 'door';
			createOpen = false;
			await refresh();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not create device');
		} finally {
			submitting = false;
		}
	}

	/** Which node the SIM800L is wired to. Only that one drains the SMS queue. */
	async function toggleSms(device: Device): Promise<void> {
		try {
			await updateDevice(device.id, { sms_capable: !device.sms_capable });
			toast.success(
				device.sms_capable
					? `${device.name} no longer sends SMS`
					: `${device.name} now sends the SMS alerts`
			);
			await refresh();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not update the device');
		}
	}

	async function rotate(device: Device): Promise<void> {
		try {
			revealed = await rotateSecret(device.id);
			await refresh();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not rotate secret');
		}
	}

	async function copy(text: string): Promise<void> {
		await navigator.clipboard.writeText(text);
		toast.success('Copied to clipboard');
	}

	onMount(() => {
		void refresh();
		const tick = setInterval(() => (nowMs = Date.now()), 10_000);
		const poll = setInterval(() => void refresh(), 30_000);
		return () => {
			clearInterval(tick);
			clearInterval(poll);
		};
	});
</script>

<svelte:head><title>Devices - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div class="flex justify-end">
		<Button onclick={() => (createOpen = true)}>
			<PlusIcon />
			Add device
		</Button>
	</div>

	<!-- fleet at a glance -->
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

	<!-- fleet -->
	<Card.Root>
	<Card.Header>
		<Card.Title>Reader nodes</Card.Title>
		<Card.Description>
			A node counts as online while its heartbeat is under 90 seconds old.
		</Card.Description>
	</Card.Header>

	<Card.Content class="flex flex-col gap-4">
		{#if loading}
			<div class="flex flex-col gap-4 py-2">
				{#each Array.from({ length: 2 }) as _, i (i)}
					<div class="flex items-center gap-3">
						<Skeleton class="size-9 rounded-full" />
						<div class="flex flex-1 flex-col gap-2">
							<Skeleton class="h-4 w-40" />
							<Skeleton class="h-3 w-24" />
						</div>
						<Skeleton class="h-5 w-16" />
					</div>
				{/each}
			</div>
		{:else if devices.length === 0}
			<Empty.Root>
				<Empty.Header>
					<Empty.Media variant="icon"><CpuIcon /></Empty.Media>
					<Empty.Title>No devices yet</Empty.Title>
					<Empty.Description>
						Add a door node and a box node, then flash their secrets into the firmware.
					</Empty.Description>
				</Empty.Header>
			</Empty.Root>
		{:else}
			<div class="overflow-hidden rounded-lg border">
				<Table.Root class="min-w-[46rem]">
					<Table.Header>
						<Table.Row class="hover:bg-transparent">
							<Table.Head>Name</Table.Head>
							<Table.Head>Kind</Table.Head>
							<Table.Head>Status</Table.Head>
							<Table.Head>Last seen</Table.Head>
							<Table.Head class="w-12"><span class="sr-only">Actions</span></Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each devices as device (device.id)}
							{@const online = isOnline(device)}
							<Table.Row>
								<Table.Cell>
									<div class="flex items-center gap-3">
										<span
											class="bg-muted text-muted-foreground flex size-9 shrink-0 items-center justify-center rounded-full"
										>
											{#if device.kind === 'door'}
												<DoorOpenIcon class="size-4" />
											{:else}
												<PackageIcon class="size-4" />
											{/if}
										</span>
										<div class="flex min-w-0 flex-col">
											<span class="truncate font-medium">{device.name}</span>
											<span class="text-muted-foreground truncate font-mono text-xs">
												{device.id.slice(0, 8)}
												<!-- Kind has its own column from sm up. -->
												<span class="sm:hidden">- {kindLabels[device.kind]}</span>
											</span>
										</div>
									</div>
								</Table.Cell>
								<Table.Cell>
									<div class="flex flex-wrap items-center gap-1.5">
										<Badge variant="outline">{kindLabels[device.kind]}</Badge>
										{#if device.sms_capable}
											<Badge variant="secondary" class="gap-1">
												<SignalIcon class="size-3" />
												GSM
											</Badge>
										{/if}
									</div>
								</Table.Cell>
								<Table.Cell>
									{#if online}
										<span class="flex items-center gap-2 text-sm">
											<span class="size-2 rounded-full bg-emerald-500"></span>
											Online
										</span>
									{:else}
										<span class="text-muted-foreground flex items-center gap-2 text-sm">
											<span class="bg-muted-foreground/40 size-2 rounded-full"></span>
											Offline
										</span>
									{/if}
								</Table.Cell>
								<Table.Cell class="text-muted-foreground text-sm">
									{device.last_seen_at ? formatDateTime(device.last_seen_at) : 'Never'}
								</Table.Cell>
								<Table.Cell>
									<DropdownMenu.Root>
										<DropdownMenu.Trigger>
											{#snippet child({ props })}
												<Button {...props} variant="ghost" size="icon">
													<MoreHorizontalIcon />
													<span class="sr-only">Actions for {device.name}</span>
												</Button>
											{/snippet}
										</DropdownMenu.Trigger>
										<DropdownMenu.Content align="end">
											<DropdownMenu.Group>
												<DropdownMenu.Item onSelect={() => copy(device.id)}>
													<CopyIcon />
													Copy device ID
												</DropdownMenu.Item>
												<DropdownMenu.Item onSelect={() => toggleSms(device)}>
													<SignalIcon />
													{device.sms_capable ? 'Remove GSM modem' : 'Mark as GSM modem'}
												</DropdownMenu.Item>
												<DropdownMenu.Item onSelect={() => rotate(device)}>
													<KeyRoundIcon />
													Rotate secret
												</DropdownMenu.Item>
											</DropdownMenu.Group>
										</DropdownMenu.Content>
									</DropdownMenu.Root>
								</Table.Cell>
							</Table.Row>
						{/each}
					</Table.Body>
				</Table.Root>
			</div>
		{/if}
	</Card.Content>
	</Card.Root>
</div>

<Dialog.Root bind:open={createOpen}>
	<Dialog.Content class="sm:max-w-md">
		<form onsubmit={create} class="grid gap-4">
			<Dialog.Header>
				<Dialog.Title>Add a device</Dialog.Title>
			</Dialog.Header>

			<Separator class="-mx-4 w-auto" />

			<div class="grid gap-4">
				<Dialog.Description>Each node authenticates with its own one-time secret.</Dialog.Description>
				<Field.FieldGroup>
					<Field.Field>
						<Field.FieldLabel for="name">Name</Field.FieldLabel>
						<Input id="name" bind:value={name} placeholder="e.g. front-door" required />
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel for="kind">Kind</Field.FieldLabel>
						<Select.Root type="single" bind:value={kind}>
							<Select.Trigger id="kind" class="w-full">{kindLabels[kind]}</Select.Trigger>
							<Select.Content>
								<Select.Group>
									<Select.Item value="door" label="Door">Door</Select.Item>
									<Select.Item value="box" label="Box">Box</Select.Item>
								</Select.Group>
							</Select.Content>
						</Select.Root>
						<Field.FieldDescription>
							Door opens the session; box checks it before unlocking.
						</Field.FieldDescription>
					</Field.Field>
				</Field.FieldGroup>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (createOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={submitting}>
					{#if submitting}<Loader2Icon class="animate-spin" />{/if}
					Create
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root
	open={revealed !== null}
	onOpenChange={(v) => {
		if (!v) revealed = null;
	}}
>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Device secret</Dialog.Title>
		</Dialog.Header>

		<Separator class="-mx-4 w-auto" />

		{#if revealed !== null}
			<div class="flex flex-col gap-3">
				<Dialog.Description>
					Copy this now. It is shown only once and cannot be recovered later.
				</Dialog.Description>
				<Field.Field>
					<Field.FieldLabel class="text-xs">Device ID</Field.FieldLabel>
					<div class="flex gap-2">
						<Input readonly value={revealed.id} class="font-mono text-xs" />
						<Button variant="outline" size="icon" onclick={() => revealed && copy(revealed.id)}>
							<CopyIcon />
						</Button>
					</div>
				</Field.Field>
				<Field.Field>
					<Field.FieldLabel class="text-xs">Secret</Field.FieldLabel>
					<div class="flex gap-2">
						<Input readonly value={revealed.secret} class="font-mono text-xs" />
						<Button variant="outline" size="icon" onclick={() => revealed && copy(revealed.secret)}>
							<CopyIcon />
						</Button>
					</div>
				</Field.Field>
			</div>

			<Dialog.Footer>
				<Button onclick={() => (revealed = null)}>Done</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
