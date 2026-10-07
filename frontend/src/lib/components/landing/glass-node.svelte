<script lang="ts">
	import type { Component } from 'svelte';
	import { cn } from '$lib/utils';

	/**
	 * One of the three floating tiles in the hero: a square stood on its corner,
	 * with the icon counter-rotated so it reads upright. Decorative, so it is
	 * hidden from the accessibility tree by the section that places it.
	 */
	let {
		icon: Icon,
		variant = 'glass',
		class: className
	}: {
		icon?: Component;
		variant?: 'glass' | 'gold' | 'wire';
		class?: string;
	} = $props();

	const shell: Record<'glass' | 'gold' | 'wire', string> = {
		glass:
			'border-[var(--glass-edge)] bg-white/6 backdrop-blur-md shadow-[inset_0_2px_1px_var(--glass-edge),0_30px_70px_-30px_#000]',
		gold: 'border-primary/70 bg-gradient-to-br from-primary/95 via-primary/70 to-primary/25 shadow-[inset_0_2px_1px_oklch(1_0_0/45%),0_0_90px_-10px_var(--beam)]',
		wire: 'border-white/12 bg-transparent'
	};

	const glyph: Record<'glass' | 'gold' | 'wire', string> = {
		glass: 'text-white/85',
		gold: 'text-primary-foreground',
		wire: 'text-white/25'
	};
</script>

<div class={cn('relative rotate-45 rounded-[22%] border', shell[variant], className)}>
	{#if Icon}
		<span class={cn('absolute inset-0 grid -rotate-45 place-items-center', glyph[variant])}>
			<Icon class="size-[38%]" strokeWidth={1.5} />
		</span>
	{/if}
</div>
