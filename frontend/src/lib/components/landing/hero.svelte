<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import DoorOpenIcon from '@lucide/svelte/icons/door-open';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import PackageIcon from '@lucide/svelte/icons/package';
	import { Button } from '$lib/components/ui/button';
	import GlassNode from './glass-node.svelte';
	import LightBeam from './light-beam.svelte';
	import { parts, proof } from './content';

	let { href, label }: { href: string; label: string } = $props();
</script>

<section class="relative isolate overflow-hidden border-b border-white/8">
	<LightBeam />

	<!-- The name, set enormous and almost invisible, sitting under everything. -->
	<span
		class="pointer-events-none absolute -bottom-6 left-1/2 -translate-x-1/2 select-none font-display text-[21vw] leading-none font-bold tracking-tighter text-white/4"
		aria-hidden="true">Sigurado</span
	>

	<div class="relative mx-auto grid max-w-6xl gap-14 px-6 pt-16 pb-12 lg:grid-cols-[1.1fr_1fr] lg:items-center lg:pt-24">
		<div class="flex flex-col items-start gap-6">
			<span
				class="flex items-center gap-2 rounded-full border border-primary/25 bg-primary/8 py-1 pr-4 pl-1 text-xs"
			>
				<span class="rounded-full bg-primary px-2 py-0.5 font-medium text-primary-foreground">
					Live
				</span>
				<span class="text-primary/90">Two scans. One person. No mystery.</span>
			</span>

			<h1 class="font-display text-5xl leading-[0.98] font-semibold tracking-tighter text-balance sm:text-6xl lg:text-7xl">
				One finger in.<br />
				<span class="text-primary">Same finger out.</span>
			</h1>

			<p class="max-w-lg text-base text-pretty text-muted-foreground sm:text-lg">
				The door opens for a registered finger. The cabinet opens for that same finger, and only
				while the window is still open. Every attempt, granted or refused, lands in the log with a
				name and a time.
			</p>

			<div class="flex flex-wrap items-center gap-3">
				<Button {href} size="lg" class="rounded-full px-6">
					{label}
					<ArrowRightIcon />
				</Button>
				<Button href="#sequence" variant="outline" size="lg" class="rounded-full px-6">
					See how it works
				</Button>
			</div>

			<div class="mt-1 flex flex-wrap items-center gap-x-5 gap-y-2 text-xs text-muted-foreground">
				{#each proof as item (item.text)}
					{@const Icon = item.icon}
					<span class="flex items-center gap-1.5">
						<Icon class="size-3.5 text-primary" />
						{item.text}
					</span>
				{/each}
			</div>
		</div>

		<!-- Door, finger, cabinet: the lit one in the middle is the finger that
		     ties the other two together. -->
		<div class="relative hidden h-80 lg:block" aria-hidden="true">
			<GlassNode icon={DoorOpenIcon} variant="glass" class="absolute top-14 left-2 size-36" />
			<GlassNode icon={FingerprintIcon} variant="gold" class="absolute top-4 left-40 size-44" />
			<GlassNode icon={PackageIcon} variant="wire" class="absolute top-24 left-[19.5rem] size-32" />
		</div>
	</div>

	<div class="relative mx-auto max-w-6xl px-6 pb-14">
		<p class="text-center text-xs tracking-wide text-muted-foreground/70 uppercase">Built from</p>
		<ul class="mt-5 flex flex-wrap items-center justify-center gap-x-9 gap-y-4">
			{#each parts as part (part.label)}
				{@const Icon = part.icon}
				<li class="flex items-center gap-2 text-sm font-medium text-white/45">
					<Icon class="size-4" />
					{part.label}
				</li>
			{/each}
		</ul>
	</div>
</section>
