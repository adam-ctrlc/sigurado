<script lang="ts">
	import { cn } from '$lib/utils';

	/** The warm light raking in from the top right, plus the faint grid it lands on. */
	let { class: className }: { class?: string } = $props();
</script>

<div class={cn('pointer-events-none absolute inset-0 overflow-hidden', className)} aria-hidden="true">
	<div
		class="absolute inset-0 [background-image:linear-gradient(to_right,var(--grid-line)_1px,transparent_1px),linear-gradient(to_bottom,var(--grid-line)_1px,transparent_1px)] [background-size:64px_64px] [mask-image:radial-gradient(ellipse_80%_70%_at_75%_0%,#000_10%,transparent_75%)]"
	></div>

	<!-- the source, just off the top right corner -->
	<div
		class="absolute -top-40 -right-24 size-[34rem] rounded-full opacity-70 blur-[90px] [background:radial-gradient(circle,var(--beam)_0%,transparent_70%)]"
	></div>

	<!-- and the shaft it throws down and to the left -->
	<div
		class="absolute -top-72 -right-16 h-[70rem] w-[26rem] rotate-[24deg] opacity-35 blur-[60px] [background:linear-gradient(to_bottom,var(--beam)_0%,transparent_65%)]"
	></div>
</div>
