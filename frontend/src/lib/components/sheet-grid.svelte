<script lang="ts">
	/**
	 * A spreadsheet, rendered from our own data.
	 *
	 * Deliberately familiar: lettered columns, numbered rows, a frozen header, and
	 * a bar above showing the selected cell in full. Cells are truncated in the
	 * grid, so that bar is how you read a long detail without a tooltip.
	 *
	 * Row numbers are absolute, not per page, so row 51 on page 2 really is the
	 * fifty-first row of the sheet.
	 */
	import * as Empty from '$lib/components/ui/empty';
	import TableIcon from '@lucide/svelte/icons/table';
	import SearchIcon from '@lucide/svelte/icons/search';

	interface Props {
		headers: string[];
		rows: string[][];
		/** One-based index of the first data row on this page. */
		firstRow: number;
		searching: boolean;
	}

	let { headers, rows, firstRow, searching }: Props = $props();

	let selected = $state<{ row: number; col: number } | null>(null);

	/** A, B, ... Z, AA, AB, the way a spreadsheet names columns. */
	function columnName(index: number): string {
		let name = '';
		let n = index;
		while (n >= 0) {
			name = String.fromCharCode(65 + (n % 26)) + name;
			n = Math.floor(n / 26) - 1;
		}
		return name;
	}

	// Row 1 holds the header, as it would in a spreadsheet, so data starts at 2.
	const headerRowNumber = 1;
	const cellRef = $derived(
		selected ? `${columnName(selected.col)}${selected.row}` : ''
	);
	const cellValue = $derived.by(() => {
		if (!selected) return '';
		if (selected.row === headerRowNumber) return headers[selected.col] ?? '';
		const row = rows[selected.row - firstRow];
		return row?.[selected.col] ?? '';
	});

	function select(row: number, col: number): void {
		selected = { row, col };
	}

	/** Arrow keys move the selection, as they would in a spreadsheet. */
	function onKeydown(event: KeyboardEvent): void {
		if (!selected) return;
		const lastRow = firstRow + rows.length - 1;
		const lastCol = headers.length - 1;
		let { row, col } = selected;

		switch (event.key) {
			case 'ArrowUp':
				row = Math.max(headerRowNumber, row - 1);
				break;
			case 'ArrowDown':
				row = Math.min(lastRow, row + 1);
				break;
			case 'ArrowLeft':
				col = Math.max(0, col - 1);
				break;
			case 'ArrowRight':
				col = Math.min(lastCol, col + 1);
				break;
			case 'Home':
				col = 0;
				break;
			case 'End':
				col = lastCol;
				break;
			case 'Escape':
				selected = null;
				return;
			default:
				return;
		}
		event.preventDefault();
		selected = { row, col };
	}

	const HEAD = 'bg-muted text-muted-foreground border-r border-b text-xs font-normal';
	const GUTTER =
		'bg-muted text-muted-foreground sticky left-0 z-10 border-r border-b px-2 text-right text-xs tabular-nums';
</script>

{#if rows.length === 0}
	<Empty.Root>
		<Empty.Header>
			<Empty.Media variant="icon">
				{#if searching}
					<SearchIcon />
				{:else}
					<TableIcon />
				{/if}
			</Empty.Media>
			<Empty.Title>{searching ? 'Nothing matches that' : 'Nothing here yet'}</Empty.Title>
			<Empty.Description>
				{searching
					? 'Try a different word, or clear the search to see the whole sheet.'
					: 'Rows appear as the readers report and people record what they took.'}
			</Empty.Description>
		</Empty.Header>
	</Empty.Root>
{:else}
	<div class="flex flex-col gap-2">
		<!-- the bar that shows a cell in full, since the grid truncates -->
		<div class="flex items-stretch gap-2">
			<span
				class="bg-muted text-muted-foreground flex w-16 shrink-0 items-center justify-center rounded-md border font-mono text-xs"
			>
				{cellRef || '--'}
			</span>
			<div
				class="bg-background flex min-h-9 min-w-0 flex-1 items-center rounded-md border px-3 py-1.5 text-sm"
			>
				<span class="min-w-0 break-words">{cellValue}</span>
			</div>
		</div>

		<div
			role="grid"
			tabindex="-1"
			aria-label="Sheet"
			onkeydown={onKeydown}
			class="overflow-x-auto rounded-lg border outline-none"
		>
			<table class="w-full border-separate border-spacing-0 text-sm">
				<thead>
					<!-- the letter strip -->
					<tr>
						<th class="{GUTTER} sticky top-0 z-20 w-12"></th>
						{#each headers as _, col (col)}
							<th class="{HEAD} sticky top-0 z-10 min-w-32 px-2 py-1 text-center font-mono">
								{columnName(col)}
							</th>
						{/each}
					</tr>
					<!-- row 1: the column names, frozen like a spreadsheet header -->
					<tr>
						<td class="{GUTTER} sticky top-7 z-20">{headerRowNumber}</td>
						{#each headers as header, col (header)}
							{@const active = selected?.row === headerRowNumber && selected?.col === col}
							<td
								class="bg-muted/60 sticky top-7 z-[9] cursor-cell truncate border-r border-b px-2 py-1 font-medium
								{active ? 'ring-primary bg-primary/10 ring-2 ring-inset' : ''}"
								onclick={() => select(headerRowNumber, col)}
							>
								{header}
							</td>
						{/each}
					</tr>
				</thead>
				<tbody>
					{#each rows as row, i (firstRow + i)}
						{@const rowNumber = firstRow + i}
						<tr class="group">
							<td class="{GUTTER} group-hover:bg-accent">{rowNumber}</td>
							{#each headers as _, col (col)}
								{@const active = selected?.row === rowNumber && selected?.col === col}
								<td
									class="group-hover:bg-accent/40 max-w-64 cursor-cell truncate border-r border-b px-2 py-1
									{active ? 'ring-primary bg-primary/10 ring-2 ring-inset' : ''}"
									onclick={() => select(rowNumber, col)}
								>
									{row[col] ?? ''}
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
	</div>
{/if}
