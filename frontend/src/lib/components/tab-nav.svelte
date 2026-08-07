<script lang="ts">
	import { page } from '$app/state';

	interface Item {
		href: string;
		label: string;
	}

	interface Props {
		items: Item[];
	}

	let { items }: Props = $props();

	// Real navigation rather than local state, so each view has its own URL and
	// the back button works. Styled to match Tabs.List so nothing looks different.
	function isActive(href: string): boolean {
		return page.url.pathname === href || page.url.pathname.startsWith(`${href}/`);
	}
</script>

<!-- Scrolls rather than shrinks: three long labels do not fit a narrow phone,
     and a squeezed tab is harder to read than one you slide to. -->
<div class="scrollbar-none -mx-1 max-w-full overflow-x-auto px-1 py-px">
	<nav
		class="bg-muted text-muted-foreground inline-flex w-fit items-center justify-center gap-0.5 rounded-lg p-[3px]"
		aria-label="Sections"
	>
		{#each items as item (item.href)}
			{@const active = isActive(item.href)}
			<a
				href={item.href}
				aria-current={active ? 'page' : undefined}
				class="focus-visible:border-ring focus-visible:ring-ring/50 inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 rounded-md border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap transition-all focus-visible:ring-[3px] focus-visible:outline-none
				{active
					? 'bg-background text-foreground dark:border-input dark:bg-input/30 shadow-sm'
					: 'text-foreground/60 hover:text-foreground dark:text-muted-foreground dark:hover:text-foreground'}"
			>
				{item.label}
			</a>
		{/each}
	</nav>
</div>
