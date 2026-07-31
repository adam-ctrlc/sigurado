<script lang="ts">
	import MousePointerClickIcon from '@lucide/svelte/icons/mouse-pointer-click';
	import InfoIcon from '@lucide/svelte/icons/info';
	import * as Card from '$lib/components/ui/card';
	import * as Alert from '$lib/components/ui/alert';
	import { Separator } from '$lib/components/ui/separator';
	import type { Guide } from '$lib/guides/types';

	interface Props {
		guide: Guide;
	}

	let { guide }: Props = $props();

	function slug(title: string): string {
		return title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
	}

	const ids = $derived(guide.sections.map((section) => slug(section.title)));

	let activeId = $state('');
	let column = $state<HTMLElement | null>(null);

	// The section whose heading sits at or just above the top of the reading area.
	// Measured on scroll rather than with an observer, so the answer is the same
	// whether the page moved by a wheel, a keypress, or a jump to an anchor.
	$effect(() => {
		const list = ids;
		void column;
		const pick = (): void => {
			let current = list[0] ?? '';
			for (const id of list) {
				const el = document.getElementById(id);
				if (el && el.getBoundingClientRect().top <= 120) current = id;
			}
			// The final section is often too short to ever reach the top, so the
			// bottom of the page hands it the highlight.
			const atEnd = window.innerHeight + window.scrollY >= document.body.scrollHeight - 8;
			activeId = atEnd ? (list.at(-1) ?? current) : current;
		};

		pick();
		// Capture, because a scroll event does not bubble and the scrolling element
		// depends on the shell around this component.
		document.addEventListener('scroll', pick, { capture: true, passive: true });
		window.addEventListener('resize', pick);
		// A jump to an anchor scrolls once, and anything that changes height
		// afterwards, a font arriving or a card growing, silently moves the section
		// that was under the reading line. Watching the column catches that.
		const observer = new ResizeObserver(() => pick());
		if (column) observer.observe(column);
		return () => {
			document.removeEventListener('scroll', pick, { capture: true });
			window.removeEventListener('resize', pick);
			observer.disconnect();
		};
	});
</script>

<div class="grid gap-6 lg:grid-cols-[minmax(0,1fr)_16rem]">
	<div bind:this={column} class="flex min-w-0 flex-col gap-6">
		<Card.Root>
			<Card.Header>
				<Card.Title class="text-xl">{guide.title}</Card.Title>
				<Card.Description>{guide.audience}</Card.Description>
			</Card.Header>
			<Card.Content>
				<p class="text-sm leading-relaxed">{guide.intro}</p>
			</Card.Content>
		</Card.Root>

		{#each guide.sections as section, index (section.title)}
			<Card.Root id={slug(section.title)} class="scroll-mt-20">
				<Card.Header>
					<Card.Title class="flex items-baseline gap-3">
						<span
							class="bg-primary/10 text-primary flex size-7 shrink-0 items-center justify-center rounded-full text-sm font-semibold tabular-nums"
						>
							{index + 1}
						</span>
						<span class="text-base font-semibold">{section.title}</span>
					</Card.Title>
					<Card.Description class="sm:pl-10">{section.summary}</Card.Description>
				</Card.Header>
				<Card.Content class="flex flex-col gap-4 sm:pl-16">
					<ol class="flex flex-col gap-4">
						{#each section.steps as step, i (step.action)}
							<li class="flex items-baseline gap-3">
								<span class="text-muted-foreground w-9 shrink-0 text-right text-xs tabular-nums">
									{index + 1}.{i + 1}
								</span>
								<div class="flex min-w-0 flex-col gap-1">
									<span class="text-sm font-medium">{step.action}</span>
									{#if step.detail}
										<span class="text-muted-foreground text-sm leading-relaxed">
											{step.detail}
										</span>
									{/if}
									{#if step.click}
										<span
											class="text-muted-foreground mt-0.5 flex w-fit items-center gap-1.5 rounded-md border px-2 py-1 text-xs"
										>
											<MousePointerClickIcon class="size-3 shrink-0" />
											<span>Click <span class="text-foreground font-medium">{step.click}</span></span>
										</span>
									{/if}
								</div>
							</li>
						{/each}
					</ol>

					{#if section.note}
						<Alert.Root>
							<InfoIcon />
							<Alert.Title>Worth knowing</Alert.Title>
							<Alert.Description>{section.note}</Alert.Description>
						</Alert.Root>
					{/if}
				</Card.Content>
			</Card.Root>
		{/each}
	</div>

	<!-- Jumping between numbered sections is the whole point of a long guide. -->
	<nav class="hidden lg:block" aria-label="Sections in this guide">
		<div class="sticky top-20 flex flex-col gap-3">
			<span class="text-muted-foreground text-xs tracking-wide uppercase">On this page</span>
			<Separator />
			<ol class="flex flex-col gap-1">
				{#each guide.sections as section, index (section.title)}
					{@const id = slug(section.title)}
					{@const active = activeId === id}
					<li>
						<!-- A fixed number column keeps every title on the same left edge,
						     and matching type sizes keep the two on one baseline. -->
						<a
							href="#{id}"
							aria-current={active ? 'true' : undefined}
							class="flex items-baseline gap-2 border-l-2 py-1 pl-3 text-sm leading-snug transition-colors
							{active
								? 'border-primary text-foreground bg-primary/5 font-medium'
								: 'text-muted-foreground hover:text-foreground border-transparent'}"
						>
							<span class="w-4 shrink-0 text-right tabular-nums">{index + 1}.</span>
							<span class="min-w-0">{section.title}</span>
						</a>
					</li>
				{/each}
			</ol>
		</div>
	</nav>
</div>
