<script lang="ts">
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import ScrollTextIcon from '@lucide/svelte/icons/scroll-text';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import PackageIcon from '@lucide/svelte/icons/package';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { DECISION_CLASS } from '$lib/events';
	import { preview } from './content';

	let { href, label }: { href: string; label: string } = $props();
</script>

<section id="record" class="border-y border-white/8 bg-white/2">
	<div class="mx-auto max-w-6xl px-6 py-20 lg:py-24">
		<div class="grid items-center gap-12 lg:grid-cols-[1fr_1.25fr]">
			<div class="flex flex-col gap-5">
				<h2 class="font-display text-4xl font-semibold tracking-tight">
					An audit trail that names people
				</h2>
				<p class="text-pretty text-muted-foreground">
					Every scan lands in a live log with the person, the reader, the finger, and the exact time.
					Denials are kept as carefully as grants, because a refused attempt is the more interesting
					record.
				</p>
				<ul class="flex flex-col gap-3 text-sm text-muted-foreground">
					<li class="flex items-start gap-2.5">
						<ScrollTextIcon class="mt-0.5 size-4 shrink-0 text-primary" />
						Streams live, so the console updates without a reload.
					</li>
					<li class="flex items-start gap-2.5">
						<FingerprintIcon class="mt-0.5 size-4 shrink-0 text-primary" />
						A finger enrolled later still names its earlier attempts.
					</li>
					<li class="flex items-start gap-2.5">
						<PackageIcon class="mt-0.5 size-4 shrink-0 text-primary" />
						Checkouts list what left the shelf, item by item.
					</li>
				</ul>
				<Button {href} variant="outline" size="sm" class="mt-1 w-fit rounded-full px-5">
					{label}
					<ArrowRightIcon />
				</Button>
			</div>

			<!-- a plain window frame, in the app's own tokens -->
			<div class="overflow-hidden rounded-2xl border border-white/10 bg-card shadow-[0_40px_90px_-40px_#000]">
				<div class="flex items-center gap-2 border-b border-white/8 bg-white/3 px-4 py-3">
					<span class="flex gap-1.5">
						<span class="size-2.5 rounded-full bg-white/15"></span>
						<span class="size-2.5 rounded-full bg-white/15"></span>
						<span class="size-2.5 rounded-full bg-white/15"></span>
					</span>
					<span class="mx-auto font-mono text-[11px] text-muted-foreground">
						sigurado / audit log
					</span>
				</div>

				<!-- Scrolls sideways on a phone, exactly like the real audit table, so
				     every value stays in its own column. -->
				<div class="overflow-x-auto">
					<div class="flex min-w-[30rem] flex-col">
						{#each preview as row, i (row.time)}
							{#if i > 0}<Separator />{/if}
							<div class="flex items-center gap-3 px-4 py-3.5">
								<span class="w-20 shrink-0 font-mono text-xs text-muted-foreground">{row.time}</span>
								<Badge class="{DECISION_CLASS[row.decision]} shrink-0">{row.label}</Badge>
								<span class="ml-auto flex shrink-0 flex-col text-right">
									<span class="text-xs">{row.person}</span>
									<span class="font-mono text-[11px] text-muted-foreground">{row.detail}</span>
								</span>
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>
	</div>
</section>
