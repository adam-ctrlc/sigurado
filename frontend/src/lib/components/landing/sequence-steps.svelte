<script lang="ts">
	import { sequence } from './content';
</script>

<section id="sequence" class="border-y border-white/8 bg-white/2">
	<div class="mx-auto max-w-6xl px-6 py-20 lg:py-24">
		<div class="flex max-w-2xl flex-col gap-4">
			<h2 class="font-display text-4xl font-semibold tracking-tight">
				Two scans, in order, or nothing opens
			</h2>
			<p class="text-pretty text-muted-foreground">
				A single reader answers "is this a registered finger". Two readers in sequence answer the
				question that actually matters: is the person at the cabinet the person who came through the
				door.
			</p>
		</div>

		<ol class="mt-12 grid gap-10 md:grid-cols-3 md:gap-6">
			{#each sequence as item, i (item.step)}
				{@const Icon = item.icon}
				<li class="flex flex-col gap-5">
					<!-- marker row: the connector sits here, clear of the text below -->
					<div class="flex items-center gap-4">
						<span
							class="flex size-12 shrink-0 items-center justify-center rounded-xl border border-primary/30 bg-primary/10 text-primary"
						>
							<Icon class="size-5" />
						</span>
						{#if i < sequence.length - 1}
							<span
								class="hidden h-px flex-1 bg-gradient-to-r from-primary/40 to-transparent md:block"
								aria-hidden="true"
							></span>
						{/if}
					</div>
					<div class="flex flex-col gap-2">
						<span class="font-mono text-xs tracking-widest text-primary/70 uppercase">
							Step {i + 1}
						</span>
						<span class="text-lg font-medium">{item.title}</span>
						<p class="text-sm leading-relaxed text-muted-foreground">{item.text}</p>
					</div>
				</li>
			{/each}
		</ol>
	</div>
</section>
