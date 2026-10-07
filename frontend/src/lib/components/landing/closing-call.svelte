<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { entries } from './content';

	let { href, label }: { href: string; label: string } = $props();
</script>

<section class="mx-auto max-w-6xl px-6 pb-20 lg:pb-24">
	<div class="relative overflow-hidden rounded-3xl border border-white/10 bg-card px-6 py-16">
		<span
			class="pointer-events-none absolute -top-40 left-1/2 size-[36rem] -translate-x-1/2 rounded-full opacity-25 blur-3xl [background:radial-gradient(circle,var(--beam)_0%,transparent_70%)]"
			aria-hidden="true"
		></span>
		<span
			class="pointer-events-none absolute inset-0 [background-image:linear-gradient(to_right,var(--grid-line)_1px,transparent_1px),linear-gradient(to_bottom,var(--grid-line)_1px,transparent_1px)] [background-size:48px_48px] [mask-image:radial-gradient(ellipse_60%_70%_at_50%_0%,#000_20%,transparent_100%)]"
			aria-hidden="true"
		></span>

		<div class="relative flex flex-col gap-12">
			<div class="flex flex-col items-center gap-5 text-center">
				<span
					class="flex size-14 items-center justify-center rounded-2xl border border-primary/40 bg-primary/15 text-primary"
				>
					<ShieldCheckIcon class="size-6" />
				</span>
				<h2 class="max-w-xl font-display text-4xl font-semibold tracking-tight text-balance sm:text-5xl">
					Stop guessing who opened the cabinet
				</h2>
				<p class="max-w-lg text-pretty text-muted-foreground">
					The readers already know. Sign in and the record is waiting for you.
				</p>
				<Button {href} size="lg" class="rounded-full px-6">
					{label}
					<ArrowRightIcon />
				</Button>
			</div>

			<Separator />

			<div class="grid gap-6 sm:grid-cols-3">
				{#each entries as entry (entry.title)}
					{@const Icon = entry.icon}
					<div class="flex items-start gap-3">
						<span
							class="flex size-9 shrink-0 items-center justify-center rounded-lg border border-white/10 bg-white/5 text-primary"
						>
							<Icon class="size-4" />
						</span>
						<div class="flex flex-col gap-0.5">
							<span class="text-sm font-medium">{entry.title}</span>
							<span class="text-xs leading-relaxed text-muted-foreground">{entry.text}</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	</div>
</section>
