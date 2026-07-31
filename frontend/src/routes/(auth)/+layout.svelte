<script lang="ts">
	import type { Snippet } from 'svelte';
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ThemeToggle from '$lib/components/theme-toggle.svelte';
	import { Button } from '$lib/components/ui/button';
	let { children }: { children: Snippet } = $props();
</script>

<div class="bg-background relative flex min-h-screen items-center justify-center overflow-hidden p-4">
	<!-- faint engineering grid, fading out toward the edges -->
	<div
		class="pointer-events-none absolute inset-0 [background-image:linear-gradient(to_right,var(--grid-line)_1px,transparent_1px),linear-gradient(to_bottom,var(--grid-line)_1px,transparent_1px)] [background-size:56px_56px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_50%,#000_40%,transparent_100%)]"
		aria-hidden="true"
	></div>
	<!-- soft accent glow behind the card -->
	<div
		class="bg-primary/10 pointer-events-none absolute top-1/2 left-1/2 size-[32rem] -translate-x-1/2 -translate-y-1/2 rounded-full blur-3xl"
		aria-hidden="true"
	></div>

	<div class="absolute top-4 left-4 z-10">
		<Button href="/" variant="ghost" size="sm" class="text-muted-foreground">
			<ArrowLeftIcon />
			Back to home
		</Button>
	</div>

	<div class="absolute top-4 right-4 z-10">
		<ThemeToggle />
	</div>

	<div class="relative flex w-full flex-col items-center">
		{@render children()}
	</div>
</div>
