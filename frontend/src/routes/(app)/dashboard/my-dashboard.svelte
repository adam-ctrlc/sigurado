<script lang="ts">
	import { onMount } from 'svelte';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import PackageIcon from '@lucide/svelte/icons/package';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import ImageOffIcon from '@lucide/svelte/icons/image-off';
	import PackageOpenIcon from '@lucide/svelte/icons/package-open';
	import ScanLineIcon from '@lucide/svelte/icons/scan-line';
	import * as Card from '$lib/components/ui/card';
	import * as Empty from '$lib/components/ui/empty';
	import * as Alert from '$lib/components/ui/alert';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { auth } from '$lib/stores/auth.svelte';
	import { myEnrollments } from '$lib/api/enrollment';
	import { listCheckouts } from '$lib/api/checkouts';
	import { API_BASE } from '$lib/api/client';
	import { formatDate, formatDateTime } from '$lib/format';
	import type { Enrollment } from '$lib/schemas/enrollment';
	import type { Checkout } from '$lib/schemas/checkout';

	const filesOrigin = API_BASE.replace(/\/api\/?$/, '');

	let enrollments = $state<Enrollment[]>([]);
	let checkouts = $state<Checkout[]>([]);
	let loading = $state(true);

	const user = $derived(auth.user);
	const firstName = $derived(user?.first_name ?? '');
	const boundCount = $derived(enrollments.filter((e) => e.status === 'bound').length);

	function startOfMonth(): number {
		const now = new Date();
		return new Date(now.getFullYear(), now.getMonth(), 1).getTime();
	}

	const thisMonth = $derived(
		checkouts.filter((c) => new Date(c.created_at).getTime() >= startOfMonth()).length
	);

	const summary = $derived([
		{
			icon: FingerprintIcon,
			label: 'Fingers enrolled',
			value: String(boundCount),
			hint: boundCount === 0 ? 'Enroll to get in' : 'Bound to your account'
		},
		{
			icon: PackageIcon,
			label: 'Total checkouts',
			value: String(checkouts.length),
			hint: 'Everything you have logged'
		},
		{
			icon: CalendarIcon,
			label: 'This month',
			value: String(thisMonth),
			hint: 'Recorded since the 1st'
		},
		{
			icon: ClockIcon,
			label: 'Last checkout',
			value: checkouts[0] ? formatDate(checkouts[0].created_at) : 'None yet',
			hint: checkouts[0] ? 'Most recent record' : 'Nothing taken so far',
			small: true
		}
	]);

	function photoUrl(path: string): string {
		return `${filesOrigin}${path}`;
	}

	onMount(async () => {
		const [e, c] = await Promise.allSettled([myEnrollments(), listCheckouts()]);
		if (e.status === 'fulfilled') enrollments = e.value;
		if (c.status === 'fulfilled') checkouts = c.value.items;
		loading = false;
	});
</script>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">
			{firstName ? `Welcome back, ${firstName}` : 'Dashboard'}
		</h1>
		<p class="text-muted-foreground text-sm">
			Your enrollment status and everything you have taken from the cabinet.
		</p>
	</div>

	<!-- your numbers -->
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
						<span class="font-semibold tabular-nums {item.small ? 'text-xl' : 'text-4xl'}">
							{item.value}
						</span>
						<span class="text-muted-foreground text-xs">{item.hint}</span>
					{/if}
				</Card.Content>
			</Card.Root>
		{/each}
	</div>

	<!-- nudge whoever has not enrolled yet -->
	{#if !loading && boundCount === 0}
		<Alert.Root>
			<ScanLineIcon />
			<Alert.Title>You are not enrolled yet</Alert.Title>
			<Alert.Description>
				Scan an unknown finger at a reader, press ENROLL for a code, then enter that code on the
				enrollment page. Do it at the door and again at the cabinet.
			</Alert.Description>
		</Alert.Root>
	{/if}

	<!-- your checkouts -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Your recent checkouts</Card.Title>
			<Card.Description>Most recent first.</Card.Description>
			<Card.Action>
				<Button href="/checkout" size="sm">Record a checkout</Button>
			</Card.Action>
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
			{:else if checkouts.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><PackageOpenIcon /></Empty.Media>
						<Empty.Title>Nothing checked out yet</Empty.Title>
						<Empty.Description>
							Anything you take from the cabinet gets recorded here with a timestamp.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each checkouts.slice(0, 6) as item, i (item.id)}
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
							<span class="text-muted-foreground text-xs">{formatDateTime(item.created_at)}</span>
						</div>
					</div>
				{/each}
			{/if}
		</Card.Content>
	</Card.Root>

	<!-- enrollment trail -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Your fingerprints</Card.Title>
			<Card.Description>
				Each reader keeps its own templates, so enroll at both the door and the cabinet.
			</Card.Description>
			<Card.Action>
				<Button href="/enrollment" variant="outline" size="sm">Enroll a fingerprint</Button>
			</Card.Action>
		</Card.Header>
		<Card.Content class="flex flex-col gap-3">
			{#if loading}
				{#each Array.from({ length: 2 }) as _, i (i)}
					<div class="flex items-center justify-between gap-2">
						<Skeleton class="h-4 w-40" />
						<Skeleton class="h-5 w-20" />
					</div>
				{/each}
			{:else if enrollments.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><FingerprintIcon /></Empty.Media>
						<Empty.Title>Not enrolled yet</Empty.Title>
						<Empty.Description>Start at a reader to get an enrollment code.</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each enrollments.slice(0, 4) as item, i (item.id)}
					{#if i > 0}<Separator />{/if}
					<div class="flex items-start justify-between gap-3">
						<div class="flex min-w-0 flex-col gap-0.5">
							<span class="truncate text-sm">
								{item.status === 'bound' ? 'Finger bound' : 'Enrollment started'}
							</span>
							<span class="text-muted-foreground text-xs">{formatDateTime(item.created_at)}</span>
						</div>
					</div>
				{/each}
			{/if}
		</Card.Content>
	</Card.Root>
</div>
