<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import ScanLineIcon from '@lucide/svelte/icons/scan-line';
	import PackageIcon from '@lucide/svelte/icons/package';
	import ClipboardListIcon from '@lucide/svelte/icons/clipboard-list';
	import PackageOpenIcon from '@lucide/svelte/icons/package-open';
	import UploadIcon from '@lucide/svelte/icons/upload';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import ImageOffIcon from '@lucide/svelte/icons/image-off';
	import XIcon from '@lucide/svelte/icons/x';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import * as Field from '$lib/components/ui/field';
	import * as Table from '$lib/components/ui/table';
	import * as InputOTP from '$lib/components/ui/input-otp';
	import * as Alert from '$lib/components/ui/alert';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Separator } from '$lib/components/ui/separator';
	import TablePager from '$lib/components/table-pager.svelte';
	import {
		listCheckouts,
		createCheckout,
		pendingCode,
		type CreateCheckoutInput
	} from '$lib/api/checkouts';
	import { API_BASE, ApiError } from '$lib/api/client';
	import { auth } from '$lib/stores/auth.svelte';
	import { formatDateTime } from '$lib/format';
	import type { Checkout } from '$lib/schemas/checkout';

	const filesOrigin = API_BASE.replace(/\/api\/?$/, '');

	type Line = { id: number; name: string; qty: number };

	let nextLineId = 1;
	function blankLine(): Line {
		return { id: nextLineId++, name: '', qty: 1 };
	}

	const CODE_LENGTH = 6;
	// Hint only. Follows the real alphabet, which leaves out 0, 1, I and O so a
	// code read off the LCD is never ambiguous.
	const CODE_HINT = 'K48TQ2';

	let lines = $state<Line[]>([blankLine(), blankLine(), blankLine()]);
	let code = $state('');
	let codeWaiting = $state(false);
	let codeChecked = $state(false);
	let photo = $state<File | null>(null);
	let preview = $state<string | null>(null);
	let fileInput = $state<HTMLInputElement | null>(null);
	let submitting = $state(false);
	let list = $state<Checkout[]>([]);
	let loading = $state(true);
	let page = $state(1);
	let total = $state(0);
	const PER_PAGE = 5;

	const steps = [
		{
			icon: ScanLineIcon,
			title: 'Scan at the cabinet',
			text: 'The cabinet only opens for the same person who just scanned at the door.'
		},
		{
			icon: PackageIcon,
			title: 'Take what you need',
			text: 'Close the door behind you. The limit switch reports it back to the server.'
		},
		{
			icon: ClipboardListIcon,
			title: 'Record it here',
			text: 'The cabinet shows a code as it opens. Type it below with what you took.'
		}
	];

	// The API stores one free-text note, so the rows are flattened into the same
	// "3x 10k resistor" shape people already write by hand.
	const note = $derived(
		lines
			.filter((line) => line.name.trim().length > 0)
			.map((line) => `${Math.max(1, Math.round(line.qty || 1))}x ${line.name.trim()}`)
			.join(', ')
	);

	function addLine(): void {
		lines.push(blankLine());
	}

	function removeLine(id: number): void {
		if (lines.length === 1) return;
		lines = lines.filter((line) => line.id !== id);
	}

	/** The website cannot know if the cabinet opened; the server can. */
	async function refreshCode(): Promise<void> {
		try {
			const result = await pendingCode();
			codeWaiting = result.waiting;
		} catch {
			// leave the previous answer in place
		} finally {
			codeChecked = true;
		}
	}

	function photoUrl(path: string): string {
		return `${filesOrigin}${path}`;
	}

	function onFile(event: Event): void {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0] ?? null;
		if (preview !== null) URL.revokeObjectURL(preview);
		photo = file;
		preview = file ? URL.createObjectURL(file) : null;
	}

	function clearPhoto(): void {
		if (preview !== null) URL.revokeObjectURL(preview);
		photo = null;
		preview = null;
		if (fileInput) fileInput.value = '';
	}

	async function refresh(): Promise<void> {
		try {
			// Searching and paging are the server's job; this page holds one page.
			const result = await listCheckouts({ page, per_page: PER_PAGE });
			list = result.items;
			total = result.total;
			page = result.page;
		} catch {
			// keep the current list; a transient failure should not blank the page
		} finally {
			loading = false;
		}
	}

	async function submit(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (submitting || note.length === 0 || code.trim().length < CODE_LENGTH) return;
		submitting = true;
		try {
			const input: CreateCheckoutInput = { note, code: code.trim().toUpperCase() };
			if (photo) input.photo = photo;
			await createCheckout(input);
			toast.success('Checkout recorded.');
			nextLineId = 1;
			lines = [blankLine(), blankLine(), blankLine()];
			code = '';
			clearPhoto();
			await refreshCode();
			// A new record lands at the top, so go back to the first page.
			page = 1;
			await refresh();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not record checkout');
		} finally {
			submitting = false;
		}
	}

	onMount(() => {
		void refresh();
		void refreshCode();
		// The code arrives from the reader, so poll while the page is open.
		const timer = setInterval(() => void refreshCode(), 5000);
		return () => clearInterval(timer);
	});
</script>

<svelte:head><title>Checkout - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Materials checkout</h1>
		<p class="text-muted-foreground text-sm">
			Record what you took from the cabinet so every item stays accounted for.
		</p>
	</div>

	<!-- how a checkout works -->
	<Card.Root>
		<Card.Content class="py-6">
			<ol class="grid gap-8 md:grid-cols-3 md:gap-6">
				{#each steps as step, i (step.title)}
					{@const Icon = step.icon}
					{@const here = i === steps.length - 1}
					<li class="flex flex-col gap-4">
						<!-- marker row: the connector lives here, clear of the text below -->
						<div class="flex items-center gap-4">
							<span
								class="flex size-10 shrink-0 items-center justify-center rounded-full border-2 transition-colors
								{here
									? 'border-primary text-primary bg-background'
									: 'border-border text-muted-foreground bg-background'}"
							>
								<Icon class="size-5" />
							</span>
							{#if i < steps.length - 1}
								<span class="bg-border hidden h-px flex-1 md:block" aria-hidden="true"></span>
							{/if}
						</div>

						<div class="flex flex-col gap-1.5">
							<div class="flex items-center gap-2">
								<span class="text-sm font-medium {here ? '' : 'text-muted-foreground'}">
									{step.title}
								</span>
								{#if here}
									<Badge variant="secondary" class="h-5 px-1.5 text-[10px]">You are here</Badge>
								{/if}
							</div>
							<p class="text-muted-foreground text-xs leading-relaxed">{step.text}</p>
						</div>
					</li>
				{/each}
			</ol>
		</Card.Content>
	</Card.Root>

	<!-- action panel -->
	<Card.Root>
		<Card.Header>
			<Card.Title>New checkout</Card.Title>
		<Card.Description>
				List each item, type the code the cabinet showed you, and add a photo if you like.
			</Card.Description>
		</Card.Header>

		<Card.Content>
			<form onsubmit={submit} class="grid gap-4 md:grid-cols-[1fr_16rem]">
				<Field.Field>
					<Field.FieldLabel for="item-0">What did you take?</Field.FieldLabel>
					<div class="overflow-hidden rounded-lg border">
						<Table.Root>
							<Table.Header>
								<Table.Row class="hover:bg-transparent">
									<Table.Head>Item</Table.Head>
									<Table.Head class="w-16 text-center sm:w-24">
										<span class="sm:hidden">Qty</span>
										<span class="hidden sm:inline">Quantity</span>
									</Table.Head>
									<Table.Head class="w-12"><span class="sr-only">Remove</span></Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each lines as line, i (line.id)}
									<Table.Row class="hover:bg-transparent">
										<Table.Cell class="p-2">
											<Input
												id="item-{i}"
												bind:value={line.name}
												placeholder="e.g. 10k resistor"
												class="border-0 shadow-none focus-visible:ring-0"
											/>
										</Table.Cell>
										<Table.Cell class="p-2">
											<Input
												type="number"
												min={1}
												bind:value={line.qty}
												aria-label="Quantity for row {i + 1}"
												class="border-0 text-center shadow-none focus-visible:ring-0"
											/>
										</Table.Cell>
										<Table.Cell class="p-2">
											<Button
												type="button"
												variant="ghost"
												size="icon"
												class="text-muted-foreground hover:text-foreground"
												disabled={lines.length === 1}
												onclick={() => removeLine(line.id)}
											>
												<XIcon />
												<span class="sr-only">Remove row {i + 1}</span>
											</Button>
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
						<div class="border-t p-2">
							<Button type="button" variant="ghost" size="sm" onclick={addLine}>
								<PlusIcon />
								Add item
							</Button>
						</div>
					</div>
					<Field.FieldDescription>Specific enough to audit later.</Field.FieldDescription>
				</Field.Field>

				<Field.Field>
					<Field.FieldLabel for="photo">Photo (optional)</Field.FieldLabel>
					{#if preview === null}
						<Input id="photo" type="file" accept="image/*" bind:ref={fileInput} onchange={onFile} />
						<Field.FieldDescription>One shot of the items on the bench.</Field.FieldDescription>
					{:else}
						<div class="flex items-center gap-3">
							<img
								src={preview}
								alt="Selected materials"
								class="size-16 shrink-0 rounded-lg border object-cover"
							/>
							<Button type="button" variant="outline" size="sm" onclick={clearPhoto}>
								<XIcon />
								Remove
							</Button>
						</div>
					{/if}
				</Field.Field>

			<div class="md:col-span-2">
					{#if codeChecked && !codeWaiting}
						<Alert.Root>
							<KeyRoundIcon />
							<Alert.Title>The cabinet has not issued a code for you</Alert.Title>
							<Alert.Description>
								Scan your finger at the cabinet. When it opens, the LCD shows a six character code
								that lets you record what you took. This page notices it on its own.
							</Alert.Description>
						</Alert.Root>
					{/if}
				</div>

				<Field.Field class="md:col-span-2">
					<Field.FieldLabel for="code-0">Code from the cabinet</Field.FieldLabel>
					<InputOTP.Root
						maxlength={CODE_LENGTH}
						bind:value={code}
						pattern="[a-zA-Z0-9]*"
						disabled={codeChecked && !codeWaiting}
						class="justify-start {codeChecked && !codeWaiting ? 'opacity-50' : ''}"
					>
						{#snippet children({ cells })}
							<InputOTP.Group>
								{#each cells.slice(0, 3) as cell, i (i)}
									<InputOTP.Slot
										{cell}
										placeholder={CODE_HINT[i]}
										class="size-11 text-lg font-semibold uppercase first:rounded-l-xl last:rounded-r-xl sm:size-12"
									/>
								{/each}
							</InputOTP.Group>
							<InputOTP.Separator />
							<InputOTP.Group>
								{#each cells.slice(3, 6) as cell, i (i)}
									<InputOTP.Slot
										{cell}
										placeholder={CODE_HINT[i + 3]}
										class="size-11 text-lg font-semibold uppercase first:rounded-l-xl last:rounded-r-xl sm:size-12"
									/>
								{/each}
							</InputOTP.Group>
						{/snippet}
					</InputOTP.Root>
					<Field.FieldDescription>
						{codeWaiting
							? 'Shown on the cabinet LCD when it opened. Good for 15 minutes.'
							: 'Available once the cabinet opens for you.'}
					</Field.FieldDescription>
				</Field.Field>

				<div class="md:col-span-2 md:justify-self-end">
					<Button
						type="submit"
						class="w-full md:w-44"
					disabled={submitting || note.length === 0 || code.trim().length < CODE_LENGTH}
					>
						{#if submitting}
							<Loader2Icon class="animate-spin" />
						{:else}
							<UploadIcon />
						{/if}
						Record checkout
					</Button>
				</div>
			</form>
		</Card.Content>
	</Card.Root>

	<!-- history -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Recent checkouts</Card.Title>
			<Card.Description>
				{auth.isStaff ? 'Every user, most recent first.' : 'Your checkouts, most recent first.'}
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col gap-4">
			{#if loading}
				{#each Array.from({ length: 3 }) as _, i (i)}
					<div class="flex gap-3">
						<Skeleton class="size-14 rounded-lg" />
						<div class="flex flex-1 flex-col gap-2">
							<Skeleton class="h-4 w-full" />
							<Skeleton class="h-3 w-24" />
						</div>
					</div>
				{/each}
			{:else if list.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><PackageOpenIcon /></Empty.Media>
						<Empty.Title>No checkouts yet</Empty.Title>
						<Empty.Description>
							Anything taken from the cabinet gets recorded here with a timestamp.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each list as item, i (item.id)}
					{#if i > 0}<Separator />{/if}
					<div class="flex items-start gap-3">
						{#if item.photo_path}
							<img
								src={photoUrl(item.photo_path)}
								alt="Materials taken"
								class="size-14 shrink-0 rounded-lg border object-cover"
							/>
						{:else}
							<div
								class="bg-muted text-muted-foreground flex size-14 shrink-0 items-center justify-center rounded-lg border"
							>
								<ImageOffIcon class="size-5" />
							</div>
						{/if}
						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<p class="text-sm leading-relaxed">{item.note}</p>
							<div class="text-muted-foreground flex flex-wrap items-center gap-2 text-xs">
								{#if auth.isStaff}
									<Badge variant="outline" class="font-normal">
										{item.user_name ?? 'Unknown'}
									</Badge>
								{/if}
								<span>{formatDateTime(item.created_at)}</span>
							</div>
						</div>
					</div>
				{/each}

				<TablePager
					{total}
					perPage={PER_PAGE}
					{page}
					noun="checkouts"
					onPage={(next) => {
						page = next;
						void refresh();
					}}
				/>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
