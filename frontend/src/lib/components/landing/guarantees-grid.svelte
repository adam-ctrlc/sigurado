<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { guarantees } from './content';
</script>

<section id="prevents" class="mx-auto max-w-6xl px-6 py-20 lg:py-24">
	<div class="flex max-w-2xl flex-col gap-4">
		<h2 class="font-display text-4xl font-semibold tracking-tight">What that ordering buys you</h2>
		<p class="text-pretty text-muted-foreground">
			A paper logbook records what people choose to write. A shared key records nothing at all.
		</p>
	</div>

	<div class="mt-12 grid gap-4 sm:grid-cols-2">
		{#each guarantees as item (item.title)}
			{@const Icon = item.icon}
			<Card.Root class="rounded-2xl transition-colors hover:ring-primary/25">
				<Card.Content class="flex flex-col gap-3.5 py-7">
					<span
						class="flex size-11 items-center justify-center rounded-xl border border-primary/25 bg-primary/10 text-primary"
					>
						<Icon class="size-5" />
					</span>
					<span class="text-base font-medium">{item.title}</span>
					<p class="text-sm leading-relaxed text-muted-foreground">{item.text}</p>
				</Card.Content>
			</Card.Root>
		{/each}
	</div>
</section>
