<script lang="ts">
	/**
	 * Until a finger is bound, the app is no use to anyone but an administrator:
	 * no door opens, and nothing they do at a reader can be recorded against
	 * them. So everybody else is held here until they enroll.
	 *
	 * The dialog has no way out on purpose. No close button, no escape, no
	 * clicking outside. The only ways forward are enrolling or signing out.
	 *
	 * This is a setup step, not a security boundary. Nothing an unenrolled person
	 * could reach would harm anything; the readers already refuse a finger that is
	 * bound to nobody, and a checkout still needs a code from a real opening.
	 */
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import { Button } from '$lib/components/ui/button';
	import { auth } from '$lib/stores/auth.svelte';
	import { allowedWhileUnenrolled } from '$lib/enrollment';

	const held = $derived(auth.needsEnrollment);
	const onAllowedPage = $derived(allowedWhileUnenrolled(page.url.pathname));
	const open = $derived(held && !onAllowedPage);

	const STEPS = [
		'Go to the reader and present the same finger two or three times, until it beeps.',
		'Read the six character code on its small screen.',
		'Type that code on the enrollment page to bind the finger to your name.'
	];

	function signOut(): void {
		auth.clear();
		void goto('/login');
	}
</script>

<AlertDialog.Root {open}>
	<AlertDialog.Content
		size="lg"
		escapeKeydownBehavior="ignore"
		interactOutsideBehavior="ignore"
		onOpenAutoFocus={(event) => event.preventDefault()}
	>
		<AlertDialog.Header>
			<AlertDialog.Title class="flex items-center gap-2">
				<FingerprintIcon class="size-5" />
				Enroll your fingerprint to continue
			</AlertDialog.Title>
			<AlertDialog.Description>
				Your account exists, but no finger is bound to it yet. Until one is, the readers do not
				know you: the door will not open, and nothing you do at the cabinet can be recorded
				against your name.
			</AlertDialog.Description>
		</AlertDialog.Header>

		<div class="flex flex-col gap-4">
			<ol class="flex flex-col gap-3">
				{#each STEPS as step, i (step)}
					<li class="flex gap-3">
						<span
							class="bg-primary/10 text-primary flex size-7 shrink-0 items-center justify-center rounded-full text-sm font-semibold tabular-nums"
						>
							{i + 1}
						</span>
						<span class="pt-0.5 text-sm">{step}</span>
					</li>
				{/each}
			</ol>

			<p class="text-muted-foreground text-sm">
				No reader nearby, or the code has expired? Ask the lab administrator. Nothing else in the
				app is available until this is done.
			</p>
		</div>

		<AlertDialog.Footer class="sm:justify-between">
			<Button variant="ghost" size="sm" onclick={signOut}>
				<LogOutIcon data-icon="inline-start" />
				Sign out
			</Button>
			<div class="flex gap-2">
				<Button variant="outline" href="/guides">
					<BookOpenIcon data-icon="inline-start" />
					Read the guide
				</Button>
				<Button href="/enrollment">
					<FingerprintIcon data-icon="inline-start" />
					Enroll now
				</Button>
			</div>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
