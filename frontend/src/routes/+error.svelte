<script lang="ts">
	import { page } from '$app/state';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import ErrorView from '$lib/components/error-view.svelte';

	// Everything the root layout renders, so an unmatched URL still arrives
	// somewhere that looks like the rest of Sigurado rather than a bare page.
	const title = $derived(
		page.status === 404 ? 'Page not found - Sigurado' : `${page.status} - Sigurado`
	);
</script>

<svelte:head><title>{title}</title></svelte:head>

<div class="flex min-h-screen flex-col bg-background text-foreground">
	<header class="border-b">
		<div class="mx-auto flex max-w-6xl items-center px-6 py-3.5">
			<a href="/" class="flex items-center gap-2.5">
				<span
					class="flex size-8 items-center justify-center rounded-full border border-primary/30 bg-primary/15 text-primary"
				>
					<ShieldCheckIcon class="size-4" />
				</span>
				<span class="font-display text-base font-semibold tracking-tight">Sigurado</span>
			</a>
		</div>
	</header>

	<main class="relative flex flex-1 items-center justify-center overflow-hidden">
		<!-- The landing's grid, so a wrong turn still lands somewhere that looks
		     like the same building. -->
		<span
			class="pointer-events-none absolute inset-0 [background-image:linear-gradient(to_right,var(--grid-line)_1px,transparent_1px),linear-gradient(to_bottom,var(--grid-line)_1px,transparent_1px)] [background-size:64px_64px] [mask-image:radial-gradient(ellipse_55%_55%_at_50%_45%,#000_10%,transparent_75%)]"
			aria-hidden="true"
		></span>
		<ErrorView status={page.status} detail={page.error?.message} />
	</main>
</div>
