<script lang="ts">
	import { onMount } from 'svelte';
	import { toast } from 'svelte-sonner';
	import CheckIcon from '@lucide/svelte/icons/check';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import ScanLineIcon from '@lucide/svelte/icons/scan-line';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import CircleCheckIcon from '@lucide/svelte/icons/circle-check';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import * as Alert from '$lib/components/ui/alert';
	import * as InputOTP from '$lib/components/ui/input-otp';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { Separator } from '$lib/components/ui/separator';
	import { myEnrollments, verifyCode } from '$lib/api/enrollment';
	import { me } from '$lib/api/auth';
	import { auth } from '$lib/stores/auth.svelte';
	import { ApiError } from '$lib/api/client';
	import { formatDateTime } from '$lib/format';
	import type { Enrollment, EnrollmentStatus } from '$lib/schemas/enrollment';

	const CODE_LENGTH = 6;
	// Hint only. Follows the real code alphabet, which leaves out 0, 1, I and O
	// so a code is never ambiguous when read off the LCD.
	const PLACEHOLDER = 'D27S43';

	let list = $state<Enrollment[]>([]);
	let loading = $state(true);
	let code = $state('');
	let submitting = $state(false);

	let nowMs = $state(Date.now());

	const latest = $derived(list[0]);

	/** A code only counts while the reader's five minute window is still open. */
	const codeLive = $derived(
		latest?.status === 'pending_code' && new Date(latest.code_expires_at).getTime() > nowMs
	);
	const codeExpired = $derived(
		latest?.status === 'pending_code' && new Date(latest.code_expires_at).getTime() <= nowMs
	);

	// 0 = the reader has not issued anything, 1 = enter the code,
	// 2 = go back and scan, 3 = finished
	const stage = $derived(
		latest?.status === 'bound'
			? 3
			: latest?.status === 'awaiting_scan'
				? 2
				: codeLive
					? 1
					: 0
	);

	const steps = [
		{
			icon: KeyRoundIcon,
			title: 'Get a code',
			text: 'At the reader, scan your finger. If it is unknown, press ENROLL and the LCD shows a code.'
		},
		{
			icon: FingerprintIcon,
			title: 'Enter it here',
			text: 'Type that code below to claim the enrollment for your account.'
		},
		{
			icon: ScanLineIcon,
			title: 'Scan to finish',
			text: 'Return to the reader and scan the same finger once more to bind it.'
		}
	];

	const statusLabel: Record<EnrollmentStatus, string> = {
		pending_code: 'Awaiting your code',
		awaiting_scan: 'Scan again at the reader',
		bound: 'Enrolled',
		expired: 'Expired',
		cancelled: 'Cancelled'
	};

	const statusVariant: Record<EnrollmentStatus, 'default' | 'secondary' | 'outline'> = {
		pending_code: 'outline',
		awaiting_scan: 'default',
		bound: 'secondary',
		expired: 'outline',
		cancelled: 'outline'
	};

	async function refresh(): Promise<void> {
		try {
			list = await myEnrollments();
			// Somebody held by the enrollment gate is freed by this, so ask the
			// server whether the finger landed rather than waiting for a reload.
			if (auth.needsEnrollment && list.some((e) => e.status === 'bound')) {
				auth.setUser(await me());
				if (auth.isEnrolled) {
					toast.success('You are enrolled. The rest of the app is open to you now.');
				}
			}
		} catch {
			// keep the current list; a transient failure should not blank the page
		} finally {
			loading = false;
		}
	}

	async function submit(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (submitting || code.trim().length < CODE_LENGTH) return;
		submitting = true;
		try {
			await verifyCode(code.trim().toUpperCase());
			toast.success('Code confirmed. Now scan the same finger at the reader.');
			code = '';
			await refresh();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not confirm the code');
		} finally {
			submitting = false;
		}
	}

	onMount(() => {
		void refresh();
		// Poll while the next move belongs to the reader: either it has yet to
		// issue a code, or it still has to bind the finger. Either way the page
		// should flip on its own rather than needing a reload.
		const timer = setInterval(() => {
			nowMs = Date.now();
			if (stage === 0 || stage === 2) void refresh();
		}, 4000);
		return () => clearInterval(timer);
	});
</script>

<svelte:head><title>Enrollment - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Fingerprint enrollment</h1>
		<p class="text-muted-foreground text-sm">
			Bind a finger to your account so the readers recognize you.
		</p>
	</div>

	<!-- progress stepper -->
	<Card.Root>
		<Card.Content class="py-6">
			<ol class="grid gap-8 md:grid-cols-3 md:gap-6">
				{#each steps as step, i (step.title)}
					{@const Icon = step.icon}
					{@const done = stage > i + 1}
					{@const current = stage === i + 1 || (stage === 0 && i === 0)}
					<li class="flex flex-col gap-4">
						<!-- marker row: the connector lives here, clear of the text below -->
						<div class="flex items-center gap-4">
							<span
								class="flex size-10 shrink-0 items-center justify-center rounded-full border-2 transition-colors
								{done
									? 'border-primary bg-primary text-primary-foreground'
									: current
										? 'border-primary text-primary bg-background'
										: 'border-border text-muted-foreground bg-background'}"
							>
								{#if done}
									<CheckIcon class="size-5" />
								{:else}
									<Icon class="size-5" />
								{/if}
							</span>
							{#if i < steps.length - 1}
								<span
									class="hidden h-px flex-1 md:block {done ? 'bg-primary' : 'bg-border'}"
									aria-hidden="true"
								></span>
							{/if}
						</div>

						<div class="flex flex-col gap-1.5">
							<div class="flex items-center gap-2">
								<span class="text-sm font-medium {current ? '' : 'text-muted-foreground'}">
									{step.title}
								</span>
								{#if current}
									<Badge variant="secondary" class="h-5 px-1.5 text-[10px]">Now</Badge>
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
			<Card.Title>
				{#if stage === 3}
					Fingerprint enrolled
				{:else if stage === 2}
					Almost there
				{:else}
					Enter your enrollment code
				{/if}
			</Card.Title>
			<Card.Description>
				{#if stage === 3}
					This finger now identifies you at the reader.
				{:else if stage === 2}
					Your code was accepted. One scan left to finish.
				{:else}
					Six characters, shown on the reader's LCD.
				{/if}
			</Card.Description>
		</Card.Header>

		<Card.Content class="flex flex-col gap-6">
			{#if loading}
				<Skeleton class="mx-auto h-14 w-72" />
				<Skeleton class="mx-auto h-9 w-32" />
			{:else if stage === 2}
				<Alert.Root>
					<ScanLineIcon />
					<Alert.Title>Go back to the reader</Alert.Title>
					<Alert.Description>
						Scan the same finger once more. This page updates by itself the moment it binds.
					</Alert.Description>
				</Alert.Root>
				<div class="text-muted-foreground flex items-center gap-2 text-sm">
					<Loader2Icon class="size-4 animate-spin" />
					Waiting for the reader...
				</div>
			{:else if stage === 3}
				<div class="flex flex-col items-center gap-3 py-6 text-center">
					<span
						class="flex size-14 items-center justify-center rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
					>
						<CircleCheckIcon class="size-7" />
					</span>
					<p class="font-medium">You are enrolled</p>
					<p class="text-muted-foreground max-w-sm text-sm">
						Each reader stores its own templates, so enroll again at the other one to use both the
						door and the cabinet.
					</p>
				</div>
			{:else}
				<form onsubmit={submit} class="flex flex-col items-center gap-6 py-4">
					<InputOTP.Root
						maxlength={CODE_LENGTH}
						bind:value={code}
						pattern="[a-zA-Z0-9]*"
						disabled={stage === 0}
						class="justify-center {stage === 0 ? 'opacity-50' : ''}"
					>
						{#snippet children({ cells })}
							<InputOTP.Group>
								{#each cells.slice(0, 3) as cell, i (i)}
									<InputOTP.Slot
										{cell}
										placeholder={PLACEHOLDER[i]}
										class="size-11 text-xl font-semibold uppercase first:rounded-l-xl last:rounded-r-xl sm:size-14 sm:text-2xl md:size-20 md:text-3xl"
									/>
								{/each}
							</InputOTP.Group>
							<InputOTP.Separator />
							<InputOTP.Group>
								{#each cells.slice(3, 6) as cell, i (i)}
									<InputOTP.Slot
										{cell}
										placeholder={PLACEHOLDER[i + 3]}
										class="size-11 text-xl font-semibold uppercase first:rounded-l-xl last:rounded-r-xl sm:size-14 sm:text-2xl md:size-20 md:text-3xl"
									/>
								{/each}
							</InputOTP.Group>
						{/snippet}
					</InputOTP.Root>

					{#if stage === 0}
						<p class="text-muted-foreground max-w-sm text-center text-sm">
							{#if codeExpired}
								The last code expired. Press ENROLL at the reader for a fresh one.
							{:else}
								The reader has not sent an enrollment code yet. Scan the finger at the door or
								cabinet reader and press ENROLL to get one.
							{/if}
						</p>
					{/if}

					<Button
						type="submit"
						size="lg"
						class="min-w-44"
						disabled={submitting || stage === 0 || code.length < CODE_LENGTH}
					>
						{#if submitting}
							<Loader2Icon class="animate-spin" />
						{/if}
						{#if stage === 0}
							{codeExpired ? 'Code expired' : 'Waiting for a code'}
						{:else}
							Confirm code
						{/if}
					</Button>
				</form>
			{/if}
		</Card.Content>
	</Card.Root>

	<!-- history -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Your enrollments</Card.Title>
			<Card.Description>Most recent first.</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col gap-3">
			{#if loading}
				{#each Array.from({ length: 3 }) as _, i (i)}
					<div class="flex items-center justify-between gap-2">
						<Skeleton class="h-4 w-32" />
						<Skeleton class="h-5 w-20" />
					</div>
				{/each}
			{:else if list.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><FingerprintIcon /></Empty.Media>
						<Empty.Title>No enrollments yet</Empty.Title>
						<Empty.Description>
							Start at the reader: scan an unknown finger, then press ENROLL for a code.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each list as item, i (item.id)}
					{#if i > 0}<Separator />{/if}
					<div class="flex items-start justify-between gap-3">
						<div class="flex min-w-0 flex-col gap-0.5">
							<span class="truncate text-sm">{statusLabel[item.status]}</span>
							<span class="text-muted-foreground text-xs">{formatDateTime(item.created_at)}</span>
						</div>
						<Badge variant={statusVariant[item.status]} class="shrink-0">
							{item.status === 'bound' ? 'Done' : item.status.replaceAll('_', ' ')}
						</Badge>
					</div>
				{/each}
			{/if}
		</Card.Content>
	</Card.Root>
</div>
