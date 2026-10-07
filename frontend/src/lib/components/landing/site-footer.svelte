<script lang="ts">
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import CpuIcon from '@lucide/svelte/icons/cpu';
	import { Separator } from '$lib/components/ui/separator';
	import { footerColumns } from './content';
</script>

<footer class="border-t border-white/8">
	<div class="mx-auto max-w-6xl px-6 py-14">
		<div class="grid gap-10 md:grid-cols-[1.4fr_1fr_1fr_1fr]">
			<div class="flex flex-col gap-3">
				<div class="flex items-center gap-2.5">
					<span
						class="flex size-8 items-center justify-center rounded-full border border-primary/30 bg-primary/15 text-primary"
					>
						<ShieldCheckIcon class="size-4" />
					</span>
					<span class="font-display text-base font-semibold tracking-tight">Sigurado</span>
				</div>
				<p class="max-w-xs text-sm text-pretty text-muted-foreground">
					Dual-biometric sequential access for a school lab stockroom. A fingerprint at the door,
					the same fingerprint at the cabinet, and a record with a name on it.
				</p>
			</div>

			{#each footerColumns as column (column.title)}
				<div class="flex flex-col gap-3">
					<h3 class="text-xs font-medium tracking-wide uppercase">{column.title}</h3>
					<ul class="flex flex-col gap-2.5 text-sm text-muted-foreground">
						{#each column.links as link (link.label)}
							<li>
								{#if link.href}
									<a href={link.href} class="transition-colors hover:text-primary">{link.label}</a>
								{:else}
									<span>{link.label}</span>
								{/if}
							</li>
						{/each}
					</ul>
				</div>
			{/each}
		</div>

		<Separator class="my-8" />

		<div
			class="flex flex-col items-center gap-2 text-xs text-muted-foreground sm:flex-row sm:justify-between"
		>
			<span>Sigurado. Dual-biometric access and materials accountability.</span>
			<span class="flex items-center gap-1.5">
				<CpuIcon class="size-3.5" />
				Two reader nodes, one audit trail.
			</span>
		</div>
	</div>
</footer>
