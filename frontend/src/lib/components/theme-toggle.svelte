<script lang="ts">
	import SunIcon from '@lucide/svelte/icons/sun';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import NotebookIcon from '@lucide/svelte/icons/notebook-text';
	import ContrastIcon from '@lucide/svelte/icons/contrast';
import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import CheckIcon from '@lucide/svelte/icons/check';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { theme, type Theme } from '$lib/stores/theme.svelte';

	type Option = { value: Theme; label: string; hint: string; icon: typeof SunIcon };

	const light: Option[] = [
		{ value: 'light', label: 'Light', hint: 'White and gray', icon: SunIcon },
		{ value: 'paper', label: 'Paper', hint: 'Warm cream and sienna', icon: NotebookIcon }
	];

	const dark: Option[] = [
		{ value: 'gold', label: 'Sigurado', hint: 'Black and gold', icon: SparklesIcon },
		{ value: 'dark', label: 'Dark', hint: 'Neutral dark', icon: MoonIcon },
		{ value: 'tech', label: 'Tech', hint: 'Slate and cyan', icon: CpuIcon },
		{ value: 'contrast', label: 'Contrast', hint: 'Black and amber', icon: ContrastIcon }
	];

	const active = $derived([...dark, ...light].find((o) => o.value === theme.current) ?? dark[0]);
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon" aria-label="Change the theme">
				{#if active}
					{@const Icon = active.icon}
					<Icon />
				{/if}
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content align="end">
		<DropdownMenu.Group>
			<DropdownMenu.GroupHeading>Light</DropdownMenu.GroupHeading>
			{#each light as option (option.value)}
				{@const Icon = option.icon}
				<DropdownMenu.Item onSelect={() => theme.set(option.value)}>
					<Icon />
					<div class="flex flex-1 flex-col">
						<span>{option.label}</span>
						<span class="text-muted-foreground text-xs">{option.hint}</span>
					</div>
					{#if theme.current === option.value}
						<CheckIcon class="text-muted-foreground size-4" />
					{/if}
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Group>

		<DropdownMenu.Separator />

		<DropdownMenu.Group>
			<DropdownMenu.GroupHeading>Dark</DropdownMenu.GroupHeading>
			{#each dark as option (option.value)}
				{@const Icon = option.icon}
				<DropdownMenu.Item onSelect={() => theme.set(option.value)}>
					<Icon />
					<div class="flex flex-1 flex-col">
						<span>{option.label}</span>
						<span class="text-muted-foreground text-xs">{option.hint}</span>
					</div>
					{#if theme.current === option.value}
						<CheckIcon class="text-muted-foreground size-4" />
					{/if}
				</DropdownMenu.Item>
			{/each}
		</DropdownMenu.Group>
	</DropdownMenu.Content>
</DropdownMenu.Root>
