<script lang="ts">
	import CheckIcon from '@lucide/svelte/icons/check';
	import CircleIcon from '@lucide/svelte/icons/circle';
	import { RULES } from '$lib/password';

	interface Props {
		password: string;
		/** Hidden until someone starts typing, so an empty form is not all red. */
		showWhenEmpty?: boolean;
	}

	let { password, showWhenEmpty = false }: Props = $props();
	const visible = $derived(showWhenEmpty || password.length > 0);
</script>

{#if visible}
	<ul class="flex flex-wrap gap-x-4 gap-y-1.5">
		{#each RULES as rule (rule.label)}
			{@const met = rule.met(password)}
			<li
				class="flex items-center gap-1.5 text-xs {met
					? 'text-emerald-600 dark:text-emerald-400'
					: 'text-muted-foreground'}"
			>
				{#if met}
					<CheckIcon class="size-3.5" />
				{:else}
					<CircleIcon class="size-3" />
				{/if}
				{rule.label}
			</li>
		{/each}
	</ul>
{/if}
