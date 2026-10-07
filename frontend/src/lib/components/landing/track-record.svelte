<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import PackageIcon from '@lucide/svelte/icons/package';
	import { Button } from '$lib/components/ui/button';
	import GlassNode from './glass-node.svelte';
	import { stats } from './content';

	let { href, label }: { href: string; label: string } = $props();
</script>

<section class="mx-auto max-w-6xl px-6 py-20 lg:py-24">
	<div class="grid gap-6 lg:grid-cols-2 lg:items-end">
		<h2 class="font-display text-4xl leading-[1.05] font-semibold tracking-tight sm:text-5xl">
			What the record<br />
			<span class="text-primary">actually knows</span>
		</h2>
		<p class="max-w-md text-pretty text-muted-foreground lg:justify-self-end">
			Not a headcount and not a promise. These are the numbers the system is built out of: how many
			readers have to agree, how long a window lasts, and how much of it anybody can go back and
			change.
		</p>
	</div>

	<div class="mt-12 grid gap-4 lg:grid-cols-[1fr_1.1fr]">
		<div class="grid gap-4 sm:grid-cols-2">
			{#each stats as stat (stat.label)}
				<div
					class="relative flex min-h-44 flex-col justify-end overflow-hidden rounded-2xl border border-white/8 bg-card p-6"
				>
					<span
						class="pointer-events-none absolute -bottom-24 -left-16 size-56 rounded-full opacity-45 blur-3xl [background:radial-gradient(circle,var(--beam)_0%,transparent_70%)]"
						aria-hidden="true"
					></span>
					<span class="relative font-display text-4xl font-semibold tracking-tight">
						{stat.value}
					</span>
					<span class="relative mt-2 text-sm font-medium">{stat.label}</span>
					<span class="relative text-xs text-muted-foreground">{stat.note}</span>
				</div>
			{/each}
		</div>

		<div
			class="relative flex flex-col overflow-hidden rounded-2xl border border-white/8 bg-card p-8 pb-0"
		>
			<span
				class="pointer-events-none absolute -top-24 right-0 size-72 rounded-full opacity-25 blur-3xl [background:radial-gradient(circle,var(--beam)_0%,transparent_70%)]"
				aria-hidden="true"
			></span>

			<h3 class="relative font-display text-3xl font-semibold tracking-tight">Want to see it?</h3>
			<p class="relative mt-3 max-w-sm text-sm leading-relaxed text-muted-foreground">
				The console shows every scan as it lands, the checkouts written against each opening, and
				the eleven checks that go looking for the ones that do not line up.
			</p>
			<Button {href} class="relative mt-6 w-fit rounded-full px-6">
				{label}
				<ArrowRightIcon />
			</Button>

			<!-- The two readers, one above the other, with the session running between them. -->
			<div class="relative mt-8 h-52" aria-hidden="true">
				<GlassNode
					icon={FingerprintIcon}
					variant="glass"
					class="absolute top-0 left-1/2 size-24 -translate-x-1/2"
				/>
				<span
					class="absolute top-20 left-1/2 h-12 w-px -translate-x-1/2 bg-gradient-to-b from-primary/70 to-transparent"
				></span>
				<GlassNode
					icon={PackageIcon}
					variant="gold"
					class="absolute top-26 left-1/2 size-28 -translate-x-1/2"
				/>
			</div>
		</div>
	</div>
</section>
