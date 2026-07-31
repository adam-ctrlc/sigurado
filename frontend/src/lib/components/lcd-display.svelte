<script lang="ts">
	interface Props {
		line1?: string;
		line2?: string;
	}

	let { line1 = '', line2 = '' }: Props = $props();

	const WIDTH = 16;

	function format(text: string): string {
		return text.slice(0, WIDTH).padEnd(WIDTH, ' ');
	}
</script>

<div
	class="rounded-md border border-emerald-950 bg-emerald-950/90 px-3 py-2 font-mono text-sm leading-6 tracking-widest text-emerald-300 shadow-inner select-none"
	aria-label="device LCD"
>
	<div class="whitespace-pre">{format(line1)}</div>
	<div class="whitespace-pre">{format(line2)}</div>
</div>
