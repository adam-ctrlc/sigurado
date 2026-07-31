<script lang="ts">
	import ChevronLeftIcon from '@lucide/svelte/icons/chevron-left';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import * as Pagination from '$lib/components/ui/pagination';

	interface Props {
		/** Total rows matching the current filters, from the server. */
		total: number;
		perPage: number;
		page: number;
		/** What the rows are, for the count line: "users", "events". */
		noun?: string;
		onPage: (page: number) => void;
	}

	let { total, perPage, page, noun = 'rows', onPage }: Props = $props();

	const first = $derived(total === 0 ? 0 : (page - 1) * perPage + 1);
	const last = $derived(Math.min(page * perPage, total));

	// Square cells joined into one strip. `first:`/`last:` cannot be used on the
	// cells themselves: each one is the only child of its own <li>, so both would
	// match every cell and round all four corners. The strip's own rounded border
	// clips the two ends instead, and the divider lives on the <li>.
	const CELL = 'size-9 rounded-none border-0 shadow-none';
	const ITEM = 'border-r last:border-r-0';
</script>

<div class="flex flex-col items-center gap-2 pt-1">
	<!-- Always rendered, even for a single page: bits-ui needs a count of at least
	     one to emit the [1] block, and a control that disappears reads as broken. -->
	<Pagination.Root count={Math.max(total, 1)} {perPage} {page} onPageChange={onPage}>
		{#snippet children({ pages, currentPage })}
			<Pagination.Content class="overflow-hidden rounded-lg border">
				<Pagination.Item class={ITEM}>
					<Pagination.PrevButton class="{CELL} px-0!" aria-label="Previous page">
						<ChevronLeftIcon class="size-4" />
					</Pagination.PrevButton>
				</Pagination.Item>

				{#each pages as p (p.key)}
					{#if p.type === 'ellipsis'}
						<Pagination.Item class={ITEM}>
							<Pagination.Ellipsis
								class="{CELL} text-muted-foreground flex items-center justify-center"
							/>
						</Pagination.Item>
					{:else}
						<Pagination.Item class={ITEM}>
							<Pagination.Link
								page={p}
								isActive={currentPage === p.value}
								class="{CELL} {currentPage === p.value
									? 'bg-primary text-primary-foreground dark:bg-primary dark:text-primary-foreground hover:bg-primary hover:text-primary-foreground dark:hover:bg-primary font-medium'
									: ''}"
							>
								{p.value}
							</Pagination.Link>
						</Pagination.Item>
					{/if}
				{/each}

				<Pagination.Item class={ITEM}>
					<Pagination.NextButton class="{CELL} px-0!" aria-label="Next page">
						<ChevronRightIcon class="size-4" />
					</Pagination.NextButton>
				</Pagination.Item>
			</Pagination.Content>
		{/snippet}
	</Pagination.Root>

	<p class="text-muted-foreground text-xs">
		{#if total === 0}
			No {noun} to show
		{:else}
			Showing {first} to {last} of {total} {noun}
		{/if}
	</p>
</div>
