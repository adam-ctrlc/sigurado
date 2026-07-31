<script lang="ts">
	import { Fingerprint, Shuffle } from '@lucide/svelte';
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';

	interface Props {
		value: string;
		remembered: string[];
		onValue: (token: string) => void;
	}

	let { value, remembered, onValue }: Props = $props();

	function randomToken(): string {
		const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
		let out = 'finger-';
		for (let i = 0; i < 10; i++) {
			out += chars[Math.floor(Math.random() * chars.length)];
		}
		return out;
	}
</script>

<div class="flex flex-col gap-2">
	<div class="flex items-center gap-2">
		<Fingerprint class="text-muted-foreground size-4 shrink-0" />
		<Input
			value={value}
			oninput={(e) => onValue(e.currentTarget.value)}
			placeholder="finger token (e.g. finger-alice-01)"
			class="font-mono"
		/>
		<Button
			type="button"
			variant="outline"
			size="sm"
			onclick={() => onValue(randomToken())}
			title="Present a new, never-seen fingerprint"
		>
			<Shuffle class="size-4" />
			Unknown
		</Button>
	</div>
	{#if remembered.length > 0}
		<div class="flex flex-wrap gap-1.5">
			{#each remembered as token (token)}
				<button
					type="button"
					class="bg-secondary text-secondary-foreground hover:bg-accent rounded-md px-2 py-0.5 font-mono text-xs"
					onclick={() => onValue(token)}
				>
					{token}
				</button>
			{/each}
		</div>
	{/if}
</div>
