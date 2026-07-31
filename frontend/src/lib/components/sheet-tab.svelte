<script lang="ts">
	/**
	 * One tab of the sheet: toolbar, grid, pager.
	 *
	 * The search box and the page number live in the address bar, so a view can be
	 * sent to somebody, and both are applied by the server rather than here.
	 */
	import { goto } from '$app/navigation';
	import { page as nav } from '$app/state';
	import { toast } from 'svelte-sonner';
	import SearchIcon from '@lucide/svelte/icons/search';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import SheetGrid from '$lib/components/sheet-grid.svelte';
	import TablePager from '$lib/components/table-pager.svelte';
	import { ApiError } from '$lib/api/client';
	import { auth } from '$lib/stores/auth.svelte';
	import { csvUrl, sheetPage, type SheetPage, type SheetTab } from '$lib/api/sheets';

	interface Props {
		tab: SheetTab;
	}

	let { tab }: Props = $props();

	const sizes = ['25', '50', '100'];

	const params = $derived(nav.url.searchParams);
	const currentPage = $derived(Math.max(1, Number(params.get('page') ?? '1') || 1));
	const perPage = $derived(params.get('per_page') ?? '50');
	const urlQuery = $derived(params.get('q') ?? '');

	let sheet = $state<SheetPage | null>(null);
	let loading = $state(true);
	let downloading = $state(false);

	// The box is typed into, so it keeps its own copy and pushes to the URL after
	// a pause. `pushed` stops a back-navigation clobbering a half-typed word.
	let query = $state(nav.url.searchParams.get('q') ?? '');
	let pushed = $state(nav.url.searchParams.get('q') ?? '');
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	$effect(() => {
		if (urlQuery !== pushed) {
			query = urlQuery;
			pushed = urlQuery;
		}
	});

	const firstRow = $derived(sheet ? (sheet.rows.page - 1) * sheet.rows.per_page + 2 : 2);

	async function setParams(next: Record<string, string | undefined>): Promise<void> {
		const url = new URL(nav.url);
		for (const [key, value] of Object.entries(next)) {
			const empty = value === undefined || value.length === 0;
			if (empty || (key === 'page' && value === '1') || (key === 'per_page' && value === '50')) {
				url.searchParams.delete(key);
			} else {
				url.searchParams.set(key, value);
			}
		}
		await goto(url, { replaceState: true, keepFocus: true, noScroll: true });
	}

	function onSearch(): void {
		clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			pushed = query;
			void setParams({ q: query, page: '1' });
		}, 300);
	}

	async function load(): Promise<void> {
		loading = true;
		try {
			sheet = await sheetPage(tab, {
				page: currentPage,
				per_page: Number(perPage),
				q: urlQuery.trim() || undefined
			});
		} catch (err) {
			sheet = null;
			toast.error(err instanceof ApiError ? err.message : 'Could not read the sheet');
		} finally {
			loading = false;
		}
	}

	/**
	 * The download needs the bearer token, which a plain link cannot carry, so the
	 * file is fetched and handed to the browser as a blob.
	 */
	async function download(): Promise<void> {
		downloading = true;
		try {
			const res = await fetch(csvUrl(tab, { q: urlQuery.trim() || undefined }), {
				headers: auth.token ? { Authorization: `Bearer ${auth.token}` } : {}
			});
			if (!res.ok) throw new Error(`server said ${res.status}`);
			const blob = await res.blob();
			const url = URL.createObjectURL(blob);
			const link = document.createElement('a');
			link.href = url;
			link.download = `sigurado-${tab}.csv`;
			link.click();
			URL.revokeObjectURL(url);
		} catch {
			toast.error('Could not download the file');
		} finally {
			downloading = false;
		}
	}

	$effect(() => {
		void [tab, currentPage, perPage, urlQuery];
		void load();
	});
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>{sheet?.title ?? 'Loading'}</Card.Title>
		<Card.Description>{sheet?.summary ?? ''}</Card.Description>
	</Card.Header>

	<Card.Content class="flex flex-col gap-4">
		<div class="flex flex-wrap items-center gap-3">
			<div class="relative min-w-56 flex-1">
				<SearchIcon
					class="text-muted-foreground pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2"
				/>
				<Input
					bind:value={query}
					oninput={onSearch}
					placeholder="Search every column"
					aria-label="Search the sheet"
					class="pl-9"
				/>
			</div>

			<Select.Root
				type="single"
				value={perPage}
				onValueChange={(value) => void setParams({ per_page: value, page: '1' })}
			>
				<Select.Trigger class="w-32">{perPage} rows</Select.Trigger>
				<Select.Content>
					<Select.Group>
						{#each sizes as size (size)}
							<Select.Item value={size} label="{size} rows">{size} rows</Select.Item>
						{/each}
					</Select.Group>
				</Select.Content>
			</Select.Root>

			<Button variant="outline" onclick={() => void load()} aria-label="Reload the sheet">
				<RefreshCwIcon />
			</Button>
			<Button variant="outline" onclick={() => void download()} disabled={downloading}>
				<DownloadIcon data-icon="inline-start" />
				{downloading ? 'Preparing...' : 'CSV'}
			</Button>
		</div>

		{#if loading && sheet === null}
			<div class="flex flex-col gap-2">
				{#each Array.from({ length: 8 }) as _, i (i)}
					<Skeleton class="h-7 w-full" />
				{/each}
			</div>
		{:else if sheet}
			<SheetGrid
				headers={sheet.headers}
				rows={sheet.rows.items}
				{firstRow}
				searching={urlQuery.trim().length > 0}
			/>
			<TablePager
				total={sheet.rows.total}
				perPage={sheet.rows.per_page}
				page={sheet.rows.page}
				noun="rows"
				onPage={(value) => void setParams({ page: String(value) })}
			/>
		{/if}
	</Card.Content>
</Card.Root>
