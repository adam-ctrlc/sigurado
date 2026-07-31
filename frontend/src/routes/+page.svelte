<script lang="ts">
	import type { Component } from 'svelte';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import DoorOpenIcon from '@lucide/svelte/icons/door-open';
	import PackageIcon from '@lucide/svelte/icons/package';
	import ScrollTextIcon from '@lucide/svelte/icons/scroll-text';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import LockKeyholeIcon from '@lucide/svelte/icons/lock-keyhole';
	import UserXIcon from '@lucide/svelte/icons/user-x';
	import ClockIcon from '@lucide/svelte/icons/clock';
	import CameraIcon from '@lucide/svelte/icons/camera';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import MicrochipIcon from '@lucide/svelte/icons/microchip';
	import PlugZapIcon from '@lucide/svelte/icons/plug-zap';
	import WifiIcon from '@lucide/svelte/icons/wifi';
	import * as Card from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import ThemeToggle from '$lib/components/theme-toggle.svelte';
	import { DECISION_CLASS } from '$lib/events';
	import { auth } from '$lib/stores/auth.svelte';

	interface Item {
		icon: Component;
		title: string;
		text: string;
	}

	const sequence: { step: string; icon: Component; title: string; text: string }[] = [
		{
			step: 'One',
			icon: DoorOpenIcon,
			title: 'Scan at the door',
			text: 'A registered finger unlocks the door and opens a short window at the cabinet, held in their name.'
		},
		{
			step: 'Two',
			icon: PackageIcon,
			title: 'Scan at the cabinet',
			text: 'The cabinet releases only for the same person, and only while their window is still open.'
		},
		{
			step: 'Three',
			icon: ScrollTextIcon,
			title: 'Recorded either way',
			text: 'Granted or denied, the decision is written down with the identity, the reader, and the time.'
		}
	];

	const proof: { icon: Component; text: string }[] = [
		{ icon: UserXIcon, text: 'Tailgating refused' },
		{ icon: ClockIcon, text: 'Timed access window' },
		{ icon: ScrollTextIcon, text: 'Named audit trail' }
	];

	// The same vocabulary and badge colors the real audit log uses.
	const preview = [
		{
			time: '9:41:07 AM',
			label: 'Cabinet unlocked',
			decision: 'granted' as const,
			person: 'Juan Ramos Dela Cruz',
			detail: 'cabinet:3'
		},
		{
			time: '9:41:02 AM',
			label: 'Door opened',
			decision: 'granted' as const,
			person: 'Juan Ramos Dela Cruz',
			detail: 'door:7'
		},
		{
			time: '9:38:55 AM',
			label: 'Cabinet denied: no door scan first',
			decision: 'denied' as const,
			person: 'Ana Villareal',
			detail: 'tailgating blocked'
		},
		{
			time: '9:22:13 AM',
			label: 'Enrollment code issued',
			decision: 'info' as const,
			person: 'Unknown finger',
			detail: 'door:12'
		}
	];

	const guarantees: Item[] = [
		{
			icon: LockKeyholeIcon,
			title: 'A key cannot be lent',
			text: 'There is nothing to borrow, copy, or sign out. The finger at the reader is the identity in the log.'
		},
		{
			icon: UserXIcon,
			title: 'Tailgating gets refused',
			text: 'Walking in behind someone leaves no door session, so the cabinet stays locked and the attempt is recorded.'
		},
		{
			icon: ClockIcon,
			title: 'The window closes on its own',
			text: 'A door scan is good for a set number of seconds. Wander off and the cabinet no longer knows you.'
		},
		{
			icon: CameraIcon,
			title: 'Materials, not just doors',
			text: 'What left the shelf is logged item by item, with an optional photo, against the person who took it.'
		}
	];

	const hardware: Item[] = [
		{
			icon: MicrochipIcon,
			title: 'Two reader nodes',
			text: 'One at the door, one on the cabinet. Each holds its own fingerprint templates and authenticates to the server with its own secret.'
		},
		{
			icon: PlugZapIcon,
			title: 'Solenoid and limit switch',
			text: 'The lock is driven through a relay, and a limit switch reports whether the door actually closed behind you.'
		},
		{
			icon: WifiIcon,
			title: 'Heartbeats and NTP time',
			text: 'Nodes report in on a schedule, so the console shows which readers are live and every record carries a synced timestamp.'
		}
	];

	const entries: Item[] = [
		{
			icon: FingerprintIcon,
			title: 'Enroll a finger',
			text: 'Claim a one-time code from the reader and bind the finger to your account.'
		},
		{
			icon: PackageIcon,
			title: 'Record a checkout',
			text: 'List what you took, quantity by quantity, with an optional photo.'
		},
		{
			icon: CpuIcon,
			title: 'Watch the readers',
			text: 'See which nodes are online, for how long, and every decision as it lands.'
		}
	];

	// Public surface only: everything behind sign-in stays out of the footer.
	const footerColumns: {
		title: string;
		links: { label: string; href?: string }[];
	}[] = [
		{
			title: 'How it works',
			links: [
				{ label: 'The sequence', href: '#sequence' },
				{ label: 'What it prevents', href: '#prevents' },
				{ label: 'The record', href: '#record' },
				{ label: 'Hardware', href: '#hardware' }
			]
		},
		{
			title: 'At the readers',
			links: [
				{ label: 'Fingerprint at the door' },
				{ label: 'Fingerprint at the cabinet' },
				{ label: 'One-time enrollment codes' },
				{ label: 'Solenoid lock and limit switch' }
			]
		},
		{
			title: 'Access',
			links: [
				{ label: 'Sign in', href: '/login' },
				{ label: 'Accounts are made by an admin' }
			]
		}
	];

	const homeHref = $derived(auth.isAuthed ? '/dashboard' : '/login');
	const homeLabel = $derived(auth.isAuthed ? 'Open the dashboard' : 'Sign in');
</script>

<svelte:head><title>Sigurado - Dual-biometric materials accountability</title></svelte:head>

<div class="bg-background text-foreground min-h-screen">
	<header class="bg-background/80 sticky top-0 z-30 border-b backdrop-blur">
		<!-- Three tracks from md up, so the nav is centered against the viewport
		     rather than just sitting next to the logo. -->
		<div
			class="mx-auto flex max-w-6xl items-center gap-4 px-6 py-3 md:grid md:grid-cols-[1fr_auto_1fr]"
		>
			<a href="/" class="flex shrink-0 items-center gap-2">
				<span
					class="bg-primary text-primary-foreground flex size-8 items-center justify-center rounded-full"
				>
					<ShieldCheckIcon class="size-4" />
				</span>
				<span class="font-display text-base font-semibold tracking-tight">Sigurado</span>
			</a>

			<nav class="text-muted-foreground hidden items-center justify-center gap-6 text-sm md:flex">
				<a href="#sequence" class="hover:text-foreground transition-colors">The sequence</a>
				<a href="#record" class="hover:text-foreground transition-colors">The record</a>
				<a href="#hardware" class="hover:text-foreground transition-colors">Hardware</a>
			</nav>

			<div class="ml-auto flex shrink-0 items-center gap-2 md:ml-0 md:justify-self-end">
				<!-- The toggle is a nicety; on a 320px screen the CTA matters more. -->
				<span class="hidden sm:inline-flex"><ThemeToggle /></span>
				<Button href={homeHref} size="sm">{homeLabel}</Button>
			</div>
		</div>
	</header>

	<!-- hero -->
	<section class="relative overflow-hidden border-b">
		<div
			class="pointer-events-none absolute inset-0 [background-image:linear-gradient(to_right,var(--grid-line)_1px,transparent_1px),linear-gradient(to_bottom,var(--grid-line)_1px,transparent_1px)] [background-size:56px_56px] [mask-image:radial-gradient(ellipse_70%_60%_at_50%_0%,#000_30%,transparent_100%)]"
			aria-hidden="true"
		></div>

		<div class="relative mx-auto flex max-w-3xl flex-col items-center gap-6 px-6 py-20 text-center">
			<Badge variant="outline" class="h-auto max-w-full gap-1.5 py-1 text-center whitespace-normal">
				<FingerprintIcon class="size-3.5 shrink-0" />
				Dual-biometric sequential access
			</Badge>

			<h1
				class="font-display text-5xl leading-[1.02] font-semibold tracking-tight text-balance sm:text-6xl"
			>
				One finger in. <span class="text-primary">Same finger out.</span>
			</h1>

			<p class="text-muted-foreground max-w-xl text-lg text-pretty">
				The door opens for a registered finger. The cabinet opens for that same finger, and only
				while the window is still open. Every attempt, granted or refused, lands in the log with a
				name and a time.
			</p>

			<div class="flex flex-wrap items-center justify-center gap-3">
				<Button href={homeHref} size="lg">
					{homeLabel}
					<ArrowRightIcon />
				</Button>
				<Button href="#sequence" variant="outline" size="lg">See how it works</Button>
			</div>

			<div
				class="text-muted-foreground mt-2 flex flex-wrap items-center justify-center gap-x-5 gap-y-2 text-xs"
			>
				{#each proof as item (item.text)}
					{@const Icon = item.icon}
					<span class="flex items-center gap-1.5">
						<Icon class="text-primary size-3.5" />
						{item.text}
					</span>
				{/each}
			</div>
		</div>
	</section>

	<!-- the sequence -->
	<section id="sequence" class="mx-auto max-w-6xl px-6 py-20">
		<div class="flex max-w-2xl flex-col gap-3">
			<h2 class="font-display text-3xl font-semibold tracking-tight">
				Two scans, in order, or nothing opens
			</h2>
			<p class="text-muted-foreground text-pretty">
				A single reader answers "is this a registered finger". Two readers in sequence answer the
				question that actually matters: is the person at the cabinet the person who came through
				the door.
			</p>
		</div>

		<ol class="mt-10 grid gap-8 md:grid-cols-3 md:gap-6">
			{#each sequence as item, i (item.step)}
				{@const Icon = item.icon}
				<li class="flex flex-col gap-4">
					<!-- marker row: the connector sits here, clear of the text below -->
					<div class="flex items-center gap-4">
						<span
							class="border-primary text-primary bg-background flex size-10 shrink-0 items-center justify-center rounded-full border-2"
						>
							<Icon class="size-5" />
						</span>
						{#if i < sequence.length - 1}
							<span class="bg-border hidden h-px flex-1 md:block" aria-hidden="true"></span>
						{/if}
					</div>
					<div class="flex flex-col gap-1.5">
						<span class="text-muted-foreground text-xs tracking-wide uppercase">{item.step}</span>
						<span class="font-medium">{item.title}</span>
						<p class="text-muted-foreground text-sm leading-relaxed">{item.text}</p>
					</div>
				</li>
			{/each}
		</ol>
	</section>

	<!-- what it guarantees -->
	<section id="prevents" class="border-y">
		<div class="mx-auto max-w-6xl px-6 py-20">
			<div class="flex max-w-2xl flex-col gap-3">
				<h2 class="font-display text-3xl font-semibold tracking-tight">
					What that ordering buys you
				</h2>
				<p class="text-muted-foreground text-pretty">
					A paper logbook records what people choose to write. A shared key records nothing at all.
				</p>
			</div>

			<div class="mt-10 grid gap-4 sm:grid-cols-2">
				{#each guarantees as item (item.title)}
					{@const Icon = item.icon}
					<Card.Root class="hover:border-primary/40 transition-colors">
						<Card.Content class="flex flex-col gap-3 py-6">
							<span
								class="bg-muted text-muted-foreground flex size-10 items-center justify-center rounded-lg"
							>
								<Icon class="size-5" />
							</span>
							<span class="font-medium">{item.title}</span>
							<p class="text-muted-foreground text-sm leading-relaxed">{item.text}</p>
						</Card.Content>
					</Card.Root>
				{/each}
			</div>
		</div>
	</section>

	<!-- the record -->
	<section id="record" class="mx-auto max-w-6xl px-6 py-20">
		<div class="grid items-center gap-10 lg:grid-cols-[1fr_1.25fr]">
			<div class="flex flex-col gap-4">
				<h2 class="font-display text-3xl font-semibold tracking-tight">An audit trail that names people</h2>
				<p class="text-muted-foreground text-pretty">
					Every scan lands in a live log with the person, the reader, the finger, and the exact
					time. Denials are kept as carefully as grants, because a refused attempt is the more
					interesting record.
				</p>
				<ul class="text-muted-foreground flex flex-col gap-2.5 text-sm">
					<li class="flex items-start gap-2">
						<ScrollTextIcon class="text-primary mt-0.5 size-4 shrink-0" />
						Streams live, so the console updates without a reload.
					</li>
					<li class="flex items-start gap-2">
						<FingerprintIcon class="text-primary mt-0.5 size-4 shrink-0" />
						A finger enrolled later still names its earlier attempts.
					</li>
					<li class="flex items-start gap-2">
						<PackageIcon class="text-primary mt-0.5 size-4 shrink-0" />
						Checkouts list what left the shelf, item by item.
					</li>
				</ul>
				<Button href={homeHref} variant="outline" size="sm" class="mt-2 w-fit">
					{homeLabel}
					<ArrowRightIcon />
				</Button>
			</div>

			<!-- a plain window frame, in the app's own tokens -->
			<div class="overflow-hidden rounded-xl border shadow-sm">
				<div class="bg-muted/50 flex items-center gap-2 border-b px-4 py-2.5">
					<span class="flex gap-1.5">
						<span class="bg-muted-foreground/30 size-2.5 rounded-full"></span>
						<span class="bg-muted-foreground/30 size-2.5 rounded-full"></span>
						<span class="bg-muted-foreground/30 size-2.5 rounded-full"></span>
					</span>
					<span class="text-muted-foreground mx-auto font-mono text-[11px]">
						sigurado / audit log
					</span>
				</div>

				<!-- Scrolls sideways on a phone, exactly like the real audit table, so
				     every value stays in its own column. -->
				<div class="overflow-x-auto">
					<div class="bg-card flex min-w-[30rem] flex-col">
					{#each preview as row, i (row.time)}
						{#if i > 0}<Separator />{/if}
							<div class="flex items-center gap-3 px-4 py-3">
							<span class="text-muted-foreground w-20 shrink-0 font-mono text-xs">{row.time}</span>
							<Badge class="{DECISION_CLASS[row.decision]} shrink-0">{row.label}</Badge>
							<span class="ml-auto flex shrink-0 flex-col text-right">
								<span class="text-xs">{row.person}</span>
								<span class="text-muted-foreground font-mono text-[11px]">{row.detail}</span>
							</span>
						</div>
						{/each}
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- hardware -->
	<section id="hardware" class="border-y">
		<div class="mx-auto max-w-6xl px-6 py-20">
			<div class="flex max-w-2xl flex-col gap-3">
				<h2 class="font-display text-3xl font-semibold tracking-tight">Built from parts you can buy</h2>
				<p class="text-muted-foreground text-pretty">
					Two microcontroller nodes, two optical fingerprint sensors, a solenoid lock, and a limit
					switch. Nothing exotic, and nothing that stops working when the network hiccups.
				</p>
			</div>

			<div class="mt-10 grid gap-4 md:grid-cols-3">
				{#each hardware as item (item.title)}
					{@const Icon = item.icon}
					<Card.Root class="hover:border-primary/40 transition-colors">
						<Card.Content class="flex flex-col gap-3 py-6">
							<span
								class="bg-muted text-muted-foreground flex size-10 items-center justify-center rounded-lg"
							>
								<Icon class="size-5" />
							</span>
							<span class="font-medium">{item.title}</span>
							<p class="text-muted-foreground text-sm leading-relaxed">{item.text}</p>
						</Card.Content>
					</Card.Root>
				{/each}
			</div>
		</div>
	</section>

	<!-- close -->
	<section class="mx-auto max-w-6xl px-6 py-20">
		<Card.Root class="relative overflow-hidden">
			<div
				class="pointer-events-none absolute inset-0 [background-image:linear-gradient(to_right,var(--grid-line)_1px,transparent_1px),linear-gradient(to_bottom,var(--grid-line)_1px,transparent_1px)] [background-size:44px_44px] [mask-image:radial-gradient(ellipse_60%_70%_at_50%_0%,#000_20%,transparent_100%)]"
				aria-hidden="true"
			></div>
			<div
				class="bg-primary/10 pointer-events-none absolute -top-24 left-1/2 size-96 -translate-x-1/2 rounded-full blur-3xl"
				aria-hidden="true"
			></div>

			<Card.Content class="relative flex flex-col gap-10 py-14">
				<div class="flex flex-col items-center gap-5 text-center">
					<span
						class="bg-primary text-primary-foreground ring-primary/15 flex size-12 items-center justify-center rounded-full ring-8"
					>
						<ShieldCheckIcon class="size-6" />
					</span>
					<h2 class="font-display max-w-xl text-4xl font-semibold tracking-tight text-balance">
						Stop guessing who opened the cabinet
					</h2>
					<p class="text-muted-foreground max-w-lg text-pretty">
						The readers already know. Sign in and the record is waiting for you.
					</p>
					<Button href={homeHref} size="lg">
						{homeLabel}
						<ArrowRightIcon />
					</Button>
				</div>

				<Separator />

				<div class="grid gap-6 sm:grid-cols-3">
					{#each entries as entry (entry.title)}
						{@const Icon = entry.icon}
						<div class="flex items-start gap-3">
							<span
								class="bg-muted text-muted-foreground flex size-9 shrink-0 items-center justify-center rounded-lg"
							>
								<Icon class="size-4" />
							</span>
							<div class="flex flex-col gap-0.5">
								<span class="text-sm font-medium">{entry.title}</span>
								<span class="text-muted-foreground text-xs leading-relaxed">{entry.text}</span>
							</div>
						</div>
					{/each}
				</div>
			</Card.Content>
		</Card.Root>
	</section>

	<footer class="border-t">
		<div class="mx-auto max-w-6xl px-6 py-14">
			<div class="grid gap-10 md:grid-cols-[1.4fr_1fr_1fr_1fr]">
				<div class="flex flex-col gap-3">
					<div class="flex items-center gap-2">
						<span
							class="bg-primary text-primary-foreground flex size-8 items-center justify-center rounded-full"
						>
							<ShieldCheckIcon class="size-4" />
						</span>
						<span class="font-display text-base font-semibold tracking-tight">Sigurado</span>
					</div>
					<p class="text-muted-foreground max-w-xs text-sm text-pretty">
						Dual-biometric sequential access for a school lab stockroom. A fingerprint at the door,
						the same fingerprint at the cabinet, and a record with a name on it.
					</p>
				</div>

				{#each footerColumns as column (column.title)}
					<div class="flex flex-col gap-3">
						<h3 class="text-xs font-medium tracking-wide uppercase">{column.title}</h3>
						<ul class="text-muted-foreground flex flex-col gap-2.5 text-sm">
							{#each column.links as link (link.label)}
								<li>
									{#if link.href}
										<a href={link.href} class="hover:text-foreground transition-colors">
											{link.label}
										</a>
									{:else}
										<span>{link.label}</span>
									{/if}
								</li>
							{/each}
						</ul>
					</div>
				{/each}
			</div>

			<Separator class="my-8" />

			<div
				class="text-muted-foreground flex flex-col items-center gap-2 text-xs sm:flex-row sm:justify-between"
			>
				<span>Sigurado. Dual-biometric access and materials accountability.</span>
				<span class="flex items-center gap-1.5">
					<CpuIcon class="size-3.5" />
					Two reader nodes, one audit trail.
				</span>
			</div>
		</div>
	</footer>
</div>
