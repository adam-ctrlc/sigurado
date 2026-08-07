<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import TriangleAlertIcon from '@lucide/svelte/icons/triangle-alert';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import Undo2Icon from '@lucide/svelte/icons/undo-2';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import HistoryIcon from '@lucide/svelte/icons/history';
	import * as Card from '$lib/components/ui/card';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Alert from '$lib/components/ui/alert';
	import * as Table from '$lib/components/ui/table';
	import * as Field from '$lib/components/ui/field';
	import * as Empty from '$lib/components/ui/empty';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Separator } from '$lib/components/ui/separator';
	import { auth } from '$lib/stores/auth.svelte';
	import { ApiError } from '$lib/api/client';
	import {
		COUNT_LABELS,
		clearData,
		purgeStatus,
		undoClear,
		type PurgeStatus
	} from '$lib/api/purge';
	import { formatDateTime, formatRelative } from '$lib/format';

	let status = $state<PurgeStatus | null>(null);
	let loading = $state(true);
	let open = $state(false);
	let working = $state(false);
	let typed = $state('');
	let note = $state('');
	let nowMs = $state(Date.now());

	// Exact, capitals and all. The server checks the same thing, so a typo here
	// is caught either way; this only stops the button being live too early.
	const phrase = $derived(status?.confirmation ?? 'DELETE ALL DATA');
	const matches = $derived(typed === phrase);
	const cleared = $derived(status?.active ?? null);
	const total = $derived(status?.visible_total ?? 0);

	const KEPT = [
		'Accounts, roles and passwords',
		'Enrolled fingerprints, so nobody has to enroll again',
		'Paired readers and their secrets',
		'Who receives text alerts'
	];

	async function load(): Promise<void> {
		loading = true;
		try {
			status = await purgeStatus();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not read the data status');
		} finally {
			loading = false;
		}
	}

	async function confirmClear(): Promise<void> {
		if (!matches) return;
		working = true;
		try {
			const record = await clearData(typed, note.trim() || undefined);
			const hidden = Object.values(record.hidden).reduce((sum, n) => sum + n, 0);
			toast.success(`${hidden} rows are now hidden. Nothing was deleted, and you can undo this.`);
			open = false;
			typed = '';
			note = '';
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'The records were not cleared');
		} finally {
			working = false;
			await load();
		}
	}

	async function putBack(): Promise<void> {
		working = true;
		try {
			await undoClear();
			toast.success('Everything is back in view.');
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not put the records back');
		} finally {
			working = false;
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

<svelte:head><title>Data - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Data</h1>
		<p class="text-muted-foreground text-sm">
			Clearing the records empties every view at once. It hides rather than deletes, so it can be
			undone, and a reader event is never destroyed.
		</p>
	</div>

	{#if loading && status === null}
		<Card.Root>
			<Card.Content class="flex flex-col gap-3 py-6">
				{#each Array.from({ length: 5 }) as _, i (i)}
					<Skeleton class="h-6 w-full" />
				{/each}
			</Card.Content>
		</Card.Root>
	{:else if status}
		{#if cleared}
			<Alert.Root>
				<EyeOffIcon />
				<Alert.Title>The records are cleared right now</Alert.Title>
				<Alert.Description class="flex flex-col gap-3">
					<span>
						{cleared.performed_by_name ?? 'An administrator'} cleared them
						{formatRelative(cleared.purged_at, nowMs)}, hiding
						{Object.values(cleared.hidden).reduce((sum, n) => sum + n, 0)} rows.
						{#if cleared.note}
							Their note: "{cleared.note}".
						{/if}
						Every row is still in the database, so putting them back is instant.
					</span>
					<Button
						variant="outline"
						size="sm"
						class="w-fit"
						onclick={() => void putBack()}
						disabled={working}
					>
						<Undo2Icon data-icon="inline-start" />
						{working ? 'Putting them back...' : 'Put everything back'}
					</Button>
				</Alert.Description>
			</Alert.Root>
		{/if}

		<Card.Root>
			<Card.Header>
				<Card.Title>What is in view</Card.Title>
				<Card.Description>
					Clearing hides all of it at once. {total.toLocaleString()} rows in total.
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
					{#each COUNT_LABELS as row (row.key)}
						<div class="flex flex-col gap-1 rounded-lg border p-4">
							<span class="text-2xl font-semibold tabular-nums">
								{status.visible[row.key].toLocaleString()}
							</span>
							<span class="text-sm font-medium">{row.label}</span>
							<span class="text-muted-foreground text-xs">{row.detail}</span>
						</div>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2">
					<ShieldCheckIcon class="size-4" />
					What clearing never touches
				</Card.Title>
				<Card.Description>
					The readers keep working exactly as they did. Clearing the records is a change to what
					people can read, not to what the hardware decides.
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<ul class="flex flex-col gap-2">
					{#each KEPT as item (item)}
						<li class="flex items-start gap-2 text-sm">
							<ShieldCheckIcon class="text-muted-foreground mt-0.5 size-4 shrink-0" />
							{item}
						</li>
					{/each}
				</ul>
			</Card.Content>
		</Card.Root>

		<!-- the dangerous part, kept visually separate from everything above -->
		<Card.Root class="border-destructive/40">
			<Card.Header>
				<Card.Title class="text-destructive flex items-center gap-2">
					<TriangleAlertIcon class="size-4" />
					Clear all records
				</Card.Title>
				<Card.Description>
					Empties the audit log, the sheet, the flags, past checkouts, sign-in history and the text
					outbox, for everybody, at once.
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<AlertDialog.Root bind:open>
					<AlertDialog.Trigger>
						{#snippet child({ props })}
							<Button {...props} variant="destructive" disabled={total === 0 && !cleared}>
								<TriangleAlertIcon data-icon="inline-start" />
								Clear all records
							</Button>
						{/snippet}
					</AlertDialog.Trigger>

					<AlertDialog.Content size="lg">
						<AlertDialog.Header>
							<AlertDialog.Title class="text-destructive flex items-center gap-2">
								<TriangleAlertIcon class="size-5" />
								This clears the records for everybody
							</AlertDialog.Title>
							<AlertDialog.Description>
								Read this before you type anything.
							</AlertDialog.Description>
						</AlertDialog.Header>

						<div class="flex flex-col gap-4">
							<Alert.Root variant="destructive">
								<TriangleAlertIcon />
								<Alert.Title>
									{total.toLocaleString()} rows will disappear from every view
								</Alert.Title>
								<Alert.Description>
									The audit log, the sheet, the flags, every past checkout, all sign-in history
									and the text outbox will read as empty, for every administrator and every
									member of faculty, not only for you.
								</Alert.Description>
							</Alert.Root>

							<div class="rounded-lg border p-3">
								<p class="mb-2 text-sm font-medium">Exactly what gets hidden</p>
								<ul class="text-muted-foreground grid gap-1 text-sm sm:grid-cols-2">
									{#each COUNT_LABELS as row (row.key)}
										<li class="flex items-baseline justify-between gap-2">
											<span>{row.label}</span>
											<span class="text-foreground font-medium tabular-nums">
												{status.visible[row.key].toLocaleString()}
											</span>
										</li>
									{/each}
								</ul>
							</div>

							<Alert.Root>
								<ShieldCheckIcon />
								<Alert.Title>You will still be signed in</Alert.Title>
								<Alert.Description>
									No account is touched, including yours. Everybody can still sign in with the
									same username and password afterwards, and every enrolled fingerprint still
									opens the door it opened before.
								</Alert.Description>
							</Alert.Root>

							<Alert.Root>
								<Undo2Icon />
								<Alert.Title>It can be undone</Alert.Title>
								<Alert.Description>
									Nothing is deleted from the database. This marks a moment and every view ignores
									what came before it, so an administrator can put it all back from this page. The
									clearing itself is recorded in the audit log with your name on it.
								</Alert.Description>
							</Alert.Root>

							<Separator />

							<Field.Field>
								<Field.FieldLabel for="purge-note">Why are you clearing it?</Field.FieldLabel>
								<Textarea
									id="purge-note"
									bind:value={note}
									rows={2}
									placeholder="End of term reset, handing the room over, and so on"
								/>
								<Field.FieldDescription>
									Optional, and kept with the record of this clearing.
								</Field.FieldDescription>
							</Field.Field>

							<Field.Field>
								<Field.FieldLabel for="purge-confirm">
									Type <span class="font-mono font-semibold">{phrase}</span> to continue
								</Field.FieldLabel>
								<Input
									id="purge-confirm"
									bind:value={typed}
									autocomplete="off"
									spellcheck={false}
									placeholder={phrase}
									class="font-mono"
									aria-invalid={typed.length > 0 && !matches ? 'true' : undefined}
								/>
								<Field.FieldDescription>
									{typed.length === 0
										? 'Capitals and spacing have to match exactly.'
										: matches
											? 'That matches. The button below is now live.'
											: 'Not yet a match. Capitals and spacing have to be exact.'}
								</Field.FieldDescription>
							</Field.Field>
						</div>

						<AlertDialog.Footer>
							<AlertDialog.Cancel disabled={working}>Keep the records</AlertDialog.Cancel>
							<Button
								variant="destructive"
								disabled={!matches || working}
								onclick={() => void confirmClear()}
							>
								{working ? 'Clearing...' : `Clear ${total.toLocaleString()} rows`}
							</Button>
						</AlertDialog.Footer>
					</AlertDialog.Content>
				</AlertDialog.Root>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title class="flex items-center gap-2">
					<HistoryIcon class="size-4" />
					Past clearings
				</Card.Title>
				<Card.Description>Every clearing ever made, and whether it was undone.</Card.Description>
			</Card.Header>
			<Card.Content>
				{#if status.history.length === 0}
					<Empty.Root class="border-0 py-6">
						<Empty.Header>
							<Empty.Media variant="icon"><HistoryIcon /></Empty.Media>
							<Empty.Title>Nobody has cleared the records</Empty.Title>
							<Empty.Description>
								If anyone does, it is listed here with their name on it.
							</Empty.Description>
						</Empty.Header>
					</Empty.Root>
				{:else}
					<div class="overflow-hidden rounded-lg border">
						<div class="overflow-x-auto">
							<Table.Root class="min-w-[40rem]">
								<Table.Header>
									<Table.Row class="hover:bg-transparent">
										<Table.Head>When</Table.Head>
										<Table.Head>By</Table.Head>
										<Table.Head class="text-right">Rows hidden</Table.Head>
										<Table.Head>Note</Table.Head>
										<Table.Head>State</Table.Head>
									</Table.Row>
								</Table.Header>
								<Table.Body>
									{#each status.history as row (row.id)}
										<Table.Row>
											<Table.Cell>
												<div class="flex flex-col">
													<span class="text-sm">{formatRelative(row.purged_at, nowMs)}</span>
													<span class="text-muted-foreground text-xs">
														{formatDateTime(row.purged_at)}
													</span>
												</div>
											</Table.Cell>
											<Table.Cell class="text-sm">{row.performed_by_name ?? '-'}</Table.Cell>
											<Table.Cell class="text-right font-medium tabular-nums">
												{Object.values(row.hidden)
													.reduce((sum, n) => sum + n, 0)
													.toLocaleString()}
											</Table.Cell>
											<Table.Cell class="text-muted-foreground max-w-64 text-sm whitespace-normal">
												{row.note ?? ''}
											</Table.Cell>
											<Table.Cell>
												{#if row.undone_at}
													<Badge variant="secondary">Put back</Badge>
												{:else}
													<Badge class="bg-destructive/15 text-destructive">In force</Badge>
												{/if}
											</Table.Cell>
										</Table.Row>
									{/each}
								</Table.Body>
							</Table.Root>
						</div>
					</div>
				{/if}
			</Card.Content>
		</Card.Root>
	{/if}
</div>
