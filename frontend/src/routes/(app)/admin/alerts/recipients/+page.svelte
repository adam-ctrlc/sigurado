<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import SendIcon from '@lucide/svelte/icons/send';
	import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import PhoneIcon from '@lucide/svelte/icons/phone';
	import CheckIcon from '@lucide/svelte/icons/check';
	import ChevronsUpDownIcon from '@lucide/svelte/icons/chevrons-up-down';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Alert from '$lib/components/ui/alert';
	import * as Field from '$lib/components/ui/field';
	import * as Empty from '$lib/components/ui/empty';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import * as Avatar from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { Switch } from '$lib/components/ui/switch';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import RoleBadge from '$lib/components/role-badge.svelte';
	import {
		listRecipients,
		listCandidates,
		createRecipient,
		updateRecipient,
		deleteRecipient,
		sendTest,
		type CreateRecipientInput
	} from '$lib/api/sms';
	import { listDevices } from '$lib/api/devices';
	import { ApiError } from '$lib/api/client';
	import type { SmsRecipient, SmsCandidate } from '$lib/schemas/sms';
	import type { Device } from '$lib/schemas/device';

	let recipients = $state<SmsRecipient[]>([]);
	let candidates = $state<SmsCandidate[]>([]);
	let devices = $state<Device[]>([]);
	let loading = $state(true);

	let dialogOpen = $state(false);
	let editing = $state<SmsRecipient | null>(null);
	let submitting = $state(false);
	let removing = $state<SmsRecipient | null>(null);
	let deletingNow = $state(false);
	let testing = $state<string | null>(null);
	let pickerOpen = $state(false);

	const blank: CreateRecipientInput = {
		user_id: '',
		phone_number: '',
		notify_cabinet_opened: true,
		notify_access_denied: true,
		notify_door_opened: false
	};
	let form = $state<CreateRecipientInput>({ ...blank });

	// Without a GSM node marked, nothing in the queue can ever leave the server.
	const gsmNode = $derived(devices.find((d) => d.sms_capable));
	const chosen = $derived(candidates.find((c) => c.id === form.user_id));
	/** A subscription is useless until the account has a number on it. */
	const needsNumber = $derived(
		editing === null ? chosen !== undefined && !chosen.phone_number : !editing.phone_number
	);
	const missingNumbers = $derived(recipients.filter((r) => !r.phone_number).length);
	const activeCount = $derived(recipients.filter((r) => r.is_active).length);
	/** The list must never be emptied, or alerts would be on but silent. */
	const canRemove = $derived(recipients.length > 1);

	async function refresh(): Promise<void> {
		try {
			recipients = await listRecipients();
		} catch {
			// keep the current list
		} finally {
			loading = false;
		}
	}

	async function refreshCandidates(): Promise<void> {
		try {
			candidates = await listCandidates();
		} catch {
			// the picker simply stays empty
		}
	}

	async function refreshDevices(): Promise<void> {
		try {
			devices = await listDevices();
		} catch {
			// the banner simply will not appear
		}
	}

	function initials(name: string): string {
		return name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('');
	}

	function subscriptions(recipient: SmsRecipient): string[] {
		const list: string[] = [];
		if (recipient.notify_cabinet_opened) list.push('Cabinet opened');
		if (recipient.notify_access_denied) list.push('Access refused');
		if (recipient.notify_door_opened) list.push('Door opened');
		return list;
	}

	function openCreate(): void {
		editing = null;
		form = { ...blank };
		void refreshCandidates();
		dialogOpen = true;
	}

	function openEdit(recipient: SmsRecipient): void {
		editing = recipient;
		form = {
			user_id: recipient.user_id,
			phone_number: recipient.phone_number ?? '',
			notify_cabinet_opened: recipient.notify_cabinet_opened,
			notify_access_denied: recipient.notify_access_denied,
			notify_door_opened: recipient.notify_door_opened
		};
		dialogOpen = true;
	}

	async function save(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (submitting) return;
		submitting = true;
		try {
			if (editing) {
				await updateRecipient(editing.id, {
					phone_number: form.phone_number?.trim() ?? '',
					notify_cabinet_opened: form.notify_cabinet_opened,
					notify_access_denied: form.notify_access_denied,
					notify_door_opened: form.notify_door_opened
				});
				toast.success(`Saved ${editing.display_name}`);
			} else {
				const created = await createRecipient({
					...form,
					phone_number: form.phone_number?.trim() || undefined
				});
				toast.success(`Added ${created.display_name}`);
			}
			form = { ...blank };
			editing = null;
			dialogOpen = false;
			await Promise.all([refresh(), refreshCandidates()]);
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not save the recipient');
		} finally {
			submitting = false;
		}
	}

	async function toggleActive(recipient: SmsRecipient): Promise<void> {
		try {
			await updateRecipient(recipient.id, { is_active: !recipient.is_active });
			await refresh();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Update failed');
		}
	}

	async function test(recipient: SmsRecipient): Promise<void> {
		testing = recipient.id;
		try {
			await sendTest(recipient.id);
			toast.success(
				`Test queued for ${recipient.display_name}. The node sends it on its next poll.`
			);
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not queue the test');
		} finally {
			testing = null;
		}
	}

	async function confirmDelete(): Promise<void> {
		if (!removing || deletingNow || !canRemove) return;
		deletingNow = true;
		try {
			await deleteRecipient(removing.id);
			toast.success(`Removed ${removing.display_name}`);
			removing = null;
			await Promise.all([refresh(), refreshCandidates()]);
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not remove the recipient');
		} finally {
			deletingNow = false;
		}
	}

	onMount(() => {
		void refresh();
		void refreshCandidates();
		void refreshDevices();
	});
</script>

<svelte:head><title>SMS Recipients - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	{#if !loading && recipients.length > 0 && activeCount === 0}
		<Alert.Root variant="destructive">
			<TriangleAlertIcon />
			<Alert.Title>Every recipient is switched off</Alert.Title>
			<Alert.Description>
				Alerts are still queued but nobody is subscribed, so nothing will be sent.
			</Alert.Description>
		</Alert.Root>
	{/if}

	{#if missingNumbers > 0}
		<Alert.Root variant="destructive">
			<TriangleAlertIcon />
			<Alert.Title>
				{missingNumbers === 1
					? 'One subscriber has no mobile number'
					: `${missingNumbers} subscribers have no mobile number`}
			</Alert.Title>
			<Alert.Description>
				Nothing can be sent to an account without a number. Edit them to add one.
			</Alert.Description>
		</Alert.Root>
	{/if}

	{#if devices.length > 0 && !gsmNode}
		<Alert.Root variant="destructive">
			<TriangleAlertIcon />
			<Alert.Title>No node is marked as carrying the modem</Alert.Title>
			<Alert.Description>
				Messages will pile up unsent. Open Devices, then turn on "GSM modem" for the node the
				SIM800L is wired to.
			</Alert.Description>
		</Alert.Root>
	{/if}

	<Card.Root>
		<Card.Header>
			<Card.Title>Who gets a text</Card.Title>
			<Card.Description>
				Faculty and admin accounts only, using the number on their profile.{#if gsmNode}
					{' '}Sent through {gsmNode.name}, the node carrying the SIM800L.
				{/if}
			</Card.Description>
			<Card.Action>
				<Button onclick={openCreate} size="sm">
					<UserPlusIcon />
					Add recipient
				</Button>
			</Card.Action>
		</Card.Header>

		<Card.Content class="flex flex-col gap-4">
			{#if loading}
				<div class="flex flex-col gap-4 py-2">
					{#each Array.from({ length: 2 }) as _, i (i)}
						<div class="flex items-center gap-3">
							<Skeleton class="size-9 rounded-full" />
							<div class="flex flex-1 flex-col gap-2">
								<Skeleton class="h-4 w-40" />
								<Skeleton class="h-3 w-28" />
							</div>
							<Skeleton class="h-5 w-24" />
						</div>
					{/each}
				</div>
			{:else if recipients.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><PhoneIcon /></Empty.Media>
						<Empty.Title>Nobody is on the list</Empty.Title>
						<Empty.Description>
							Subscribe a faculty or admin account and it starts receiving a text whenever the
							cabinet opens.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="overflow-hidden rounded-lg border">
					<Table.Root class="min-w-[52rem]">
						<Table.Header>
							<Table.Row class="hover:bg-transparent">
								<Table.Head>Person</Table.Head>
								<Table.Head>Number</Table.Head>
								<Table.Head>Alerts</Table.Head>
								<Table.Head>Active</Table.Head>
								<Table.Head class="w-12"><span class="sr-only">Actions</span></Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each recipients as recipient (recipient.id)}
								<Table.Row>
									<Table.Cell>
										<div class="flex items-center gap-3">
											<Avatar.Root class="size-9">
												<Avatar.Fallback class="text-xs">
													{initials(recipient.display_name)}
												</Avatar.Fallback>
											</Avatar.Root>
											<div class="flex min-w-0 flex-col">
												<div class="flex items-center gap-2">
													<span class="truncate font-medium">{recipient.display_name}</span>
													<RoleBadge role={recipient.role} />
												</div>
												<span class="text-muted-foreground truncate text-xs">
													@{recipient.username}
												</span>
													</div>
										</div>
									</Table.Cell>
									<Table.Cell class="text-sm">
										{#if recipient.phone_number}
											<span class="font-mono">{recipient.phone_number}</span>
										{:else}
											<span class="text-destructive text-xs">No number yet</span>
										{/if}
									</Table.Cell>
									<Table.Cell>
										<div class="flex flex-wrap gap-1.5">
											{#each subscriptions(recipient) as label (label)}
												<Badge variant="outline" class="font-normal">{label}</Badge>
											{/each}
											{#if subscriptions(recipient).length === 0}
												<span class="text-muted-foreground text-xs">Nothing selected</span>
											{/if}
										</div>
									</Table.Cell>
									<Table.Cell>
										<Switch
											checked={recipient.is_active}
											onCheckedChange={() => toggleActive(recipient)}
											aria-label="Receive alerts"
										/>
									</Table.Cell>
									<Table.Cell>
										<DropdownMenu.Root>
											<DropdownMenu.Trigger>
												{#snippet child({ props })}
													<Button {...props} variant="ghost" size="icon">
														<MoreHorizontalIcon />
														<span class="sr-only">Actions for {recipient.display_name}</span>
													</Button>
												{/snippet}
											</DropdownMenu.Trigger>
											<DropdownMenu.Content align="end">
												<DropdownMenu.Group>
													<DropdownMenu.Item onSelect={() => openEdit(recipient)}>
														<PencilIcon />
														Edit
													</DropdownMenu.Item>
													<DropdownMenu.Item
														disabled={!recipient.phone_number}
														onSelect={() => test(recipient)}
													>
														{#if testing === recipient.id}
															<Loader2Icon class="animate-spin" />
														{:else}
															<SendIcon />
														{/if}
														Send a test
													</DropdownMenu.Item>
												</DropdownMenu.Group>
												<DropdownMenu.Separator />
												<DropdownMenu.Group>
													<DropdownMenu.Item
														variant="destructive"
														disabled={!canRemove}
														onSelect={() => (removing = recipient)}
													>
														<Trash2Icon />
														{canRemove ? 'Remove' : 'Cannot remove the last one'}
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

				{#if !canRemove}
					<p class="text-muted-foreground text-xs">
						At least one person stays subscribed. Add someone else first to remove this one.
					</p>
				{/if}
			{/if}
		</Card.Content>
	</Card.Root>
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<form onsubmit={save} class="grid gap-4">
			<Dialog.Header>
				<Dialog.Title>{editing ? 'Edit recipient' : 'Add a recipient'}</Dialog.Title>
			</Dialog.Header>

			<Separator class="-mx-4 w-auto" />

			<div class="grid gap-4">
				<Dialog.Description>
					One text per subscribed event, sent from the cabinet node's SIM.
				</Dialog.Description>

				<Field.FieldGroup>
					{#if editing}
						<Field.Field>
							<Field.FieldLabel>Person</Field.FieldLabel>
							<div class="flex items-center gap-3">
								<Avatar.Root class="size-9">
									<Avatar.Fallback class="text-xs">
										{initials(editing.display_name)}
									</Avatar.Fallback>
								</Avatar.Root>
								<div class="flex flex-col">
									<span class="text-sm font-medium">{editing.display_name}</span>
									<span class="text-muted-foreground text-xs">@{editing.username}</span>
								</div>
							</div>
						</Field.Field>
					{:else}
						<Field.Field>
							<Field.FieldLabel for="user">Person</Field.FieldLabel>
							<Popover.Root bind:open={pickerOpen}>
								<Popover.Trigger id="user" disabled={candidates.length === 0}>
									{#snippet child({ props })}
										<Button
											{...props}
											variant="outline"
											role="combobox"
											aria-expanded={pickerOpen}
											class="w-full justify-between font-normal"
										>
											{#if chosen}
												<span class="flex items-center gap-2">
													{chosen.display_name}
													<span class="text-muted-foreground text-xs">@{chosen.username}</span>
												</span>
											{:else}
												<span class="text-muted-foreground">
													{candidates.length === 0
														? 'Nobody left to add'
														: 'Choose a faculty or admin account'}
												</span>
											{/if}
											<ChevronsUpDownIcon class="opacity-50" />
										</Button>
									{/snippet}
								</Popover.Trigger>
								<Popover.Content class="w-(--bits-popover-anchor-width) p-0" align="start">
									<Command.Root>
										<Command.Input placeholder="Search name or username" />
										<Command.List>
											<Command.Empty>No staff account matches that.</Command.Empty>
											<Command.Group>
												{#each candidates as candidate (candidate.id)}
													<Command.Item
														value="{candidate.display_name} {candidate.username}"
														onSelect={() => {
															form.user_id = candidate.id;
															// Prefill from the account so the number field is not
															// blank for someone who already has one.
															form.phone_number = candidate.phone_number ?? '';
															pickerOpen = false;
														}}
													>
														<CheckIcon
															class={form.user_id === candidate.id ? '' : 'text-transparent'}
														/>
														<div class="flex min-w-0 flex-1 flex-col">
															<span class="truncate text-sm">{candidate.display_name}</span>
															<span class="text-muted-foreground truncate text-xs">
																@{candidate.username}
																{#if !candidate.phone_number}
																	- no number yet
																{/if}
															</span>
														</div>
														<RoleBadge role={candidate.role} />
													</Command.Item>
												{/each}
											</Command.Group>
										</Command.List>
									</Command.Root>
								</Popover.Content>
							</Popover.Root>
							<Field.FieldDescription>
								{candidates.length === 0
									? 'Every faculty and admin account is already subscribed.'
									: 'Students never receive alerts, so only staff appear here.'}
							</Field.FieldDescription>
						</Field.Field>
					{/if}

					<Field.Field>
						<Field.FieldLabel for="phone">Mobile number</Field.FieldLabel>
						<Input
							id="phone"
							bind:value={form.phone_number}
							placeholder="09171234567"
							inputmode="tel"
							required={needsNumber}
						/>
						<Field.FieldDescription>
							{needsNumber
								? 'This account has no number yet. Saving here adds it to their profile.'
								: 'Saved on their profile, so it is stored in one place only.'}
						</Field.FieldDescription>
					</Field.Field>

					<Field.FieldSet>
						<Field.FieldLegend class="text-sm">Send a text when</Field.FieldLegend>
						<div class="flex flex-col gap-3 pt-1">
							<div class="flex items-center justify-between gap-4">
								<div class="flex flex-col">
									<span class="text-sm">The cabinet opens</span>
									<span class="text-muted-foreground text-xs">Someone took materials out.</span>
								</div>
								<Switch
									bind:checked={form.notify_cabinet_opened}
									aria-label="Alert when the cabinet opens"
								/>
							</div>
							<div class="flex items-center justify-between gap-4">
								<div class="flex flex-col">
									<span class="text-sm">Access is refused</span>
									<span class="text-muted-foreground text-xs">
										Includes a tailgater turned away at the cabinet.
									</span>
								</div>
								<Switch
									bind:checked={form.notify_access_denied}
									aria-label="Alert when access is refused"
								/>
							</div>
							<div class="flex items-center justify-between gap-4">
								<div class="flex flex-col">
									<span class="text-sm">The door opens</span>
									<span class="text-muted-foreground text-xs">Noisy: every entry sends a text.</span>
								</div>
								<Switch
									bind:checked={form.notify_door_opened}
									aria-label="Alert when the door opens"
								/>
							</div>
						</div>
					</Field.FieldSet>
				</Field.FieldGroup>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (dialogOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={submitting || (!editing && form.user_id.length === 0)}>
					{#if submitting}<Loader2Icon class="animate-spin" />{/if}
					{editing ? 'Save changes' : 'Add recipient'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<AlertDialog.Root
	open={removing !== null}
	onOpenChange={(v) => {
		if (!v) removing = null;
	}}
>
	<AlertDialog.Content interactOutsideBehavior="close" escapeKeydownBehavior="close">
		<AlertDialog.Header>
			<AlertDialog.Title>Remove {removing?.display_name ?? 'this recipient'}?</AlertDialog.Title>
		</AlertDialog.Header>

		<Separator class="-mx-4 w-auto" />

		<AlertDialog.Description>
			They stop receiving alerts immediately. Their account and number are untouched, and messages
			already sent stay in the outbox.
		</AlertDialog.Description>

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={deletingNow}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				onclick={(event) => {
					event.preventDefault();
					void confirmDelete();
				}}
				disabled={deletingNow}
			>
				{#if deletingNow}<Loader2Icon class="animate-spin" />{/if}
				Remove
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
