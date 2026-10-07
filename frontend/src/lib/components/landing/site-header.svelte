<script lang="ts">
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import { Button } from '$lib/components/ui/button';

	let { href, label }: { href: string; label: string } = $props();
</script>

<header class="sticky top-0 z-30 border-b border-white/8 bg-background/70 backdrop-blur-xl">
	<div class="mx-auto flex max-w-6xl items-center gap-4 px-6 py-3.5 md:grid md:grid-cols-[1fr_auto_1fr]">
		<a href="/" class="flex shrink-0 items-center gap-2.5">
			<span
				class="flex size-8 items-center justify-center rounded-full border border-primary/30 bg-primary/15 text-primary"
			>
				<ShieldCheckIcon class="size-4" />
			</span>
			<span class="font-display text-base font-semibold tracking-tight">Sigurado</span>
		</a>

		<nav class="hidden items-center justify-center gap-7 text-sm text-muted-foreground md:flex">
			<a href="#sequence" class="transition-colors hover:text-foreground">The sequence</a>
			<a href="#record" class="transition-colors hover:text-foreground">The record</a>
			<a href="#hardware" class="transition-colors hover:text-foreground">Hardware</a>
		</nav>

		<div class="ml-auto flex shrink-0 items-center md:ml-0 md:justify-self-end">
			<Button {href} size="sm" class="rounded-full px-4">{label}</Button>
		</div>
	</div>
</header>
