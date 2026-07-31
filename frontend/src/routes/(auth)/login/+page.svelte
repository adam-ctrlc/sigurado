<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import UserIcon from '@lucide/svelte/icons/user';
	import LockIcon from '@lucide/svelte/icons/lock';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import InfoIcon from '@lucide/svelte/icons/info';
	import * as Card from '$lib/components/ui/card';
	import * as Field from '$lib/components/ui/field';
	import * as InputGroup from '$lib/components/ui/input-group';
	import * as Alert from '$lib/components/ui/alert';
	import { Button } from '$lib/components/ui/button';
	import { login } from '$lib/api/auth';
	import { auth } from '$lib/stores/auth.svelte';
	import { ApiError } from '$lib/api/client';

	let identifier = $state('');
	let password = $state('');
	let showPassword = $state(false);
	let submitting = $state(false);
	let mounted = $state(false);
	let error = $state<string | null>(null);

	onMount(() => {
		mounted = true;
		if (auth.isAuthed) void goto(HOME);
	});

	// Every role has a dashboard now: staff see the room, students see their own record.
	const HOME = '/dashboard';

	async function submit(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (submitting) return;
		submitting = true;
		error = null;
		try {
			const result = await login(identifier.trim(), password);
			auth.setSession(result.token, result.user);
			await goto(HOME);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Could not sign in';
		} finally {
			submitting = false;
		}
	}
</script>

<svelte:head><title>Sign in - Sigurado</title></svelte:head>

<Card.Root class="w-full max-w-sm shadow-lg">
	<Card.Header class="justify-items-center text-center">
		<div
			class="bg-primary/10 ring-primary/10 mb-2 flex size-12 items-center justify-center rounded-full ring-8"
		>
			<ShieldCheckIcon class="text-primary size-6" />
		</div>
		<Card.Title class="text-xl">Sign in to Sigurado</Card.Title>
		<Card.Description>Use your email or username.</Card.Description>
	</Card.Header>
	<Card.Content>
		<form class="flex flex-col gap-6" onsubmit={submit}>
			<Field.FieldGroup>
				<Field.Field>
					<Field.FieldLabel for="identifier">Email or username</Field.FieldLabel>
					<InputGroup.Root>
						<InputGroup.Input
							id="identifier"
							bind:value={identifier}
							autocomplete="username"
							placeholder="you@school.edu or juan.r.delacruz"
							required
						/>
						<InputGroup.Addon>
							<UserIcon />
						</InputGroup.Addon>
					</InputGroup.Root>
				</Field.Field>

				<Field.Field>
					<Field.FieldLabel for="password">Password</Field.FieldLabel>
					<InputGroup.Root>
						<InputGroup.Input
							id="password"
							type={showPassword ? 'text' : 'password'}
							bind:value={password}
							autocomplete="current-password"
							placeholder="Your password"
							required
						/>
						<InputGroup.Addon>
							<LockIcon />
						</InputGroup.Addon>
						<InputGroup.Addon align="inline-end">
							<InputGroup.Button
								type="button"
								size="icon-xs"
								onclick={() => (showPassword = !showPassword)}
								aria-label={showPassword ? 'Hide password' : 'Show password'}
							>
								{#if showPassword}
									<EyeOffIcon />
								{:else}
									<EyeIcon />
								{/if}
							</InputGroup.Button>
						</InputGroup.Addon>
					</InputGroup.Root>
				</Field.Field>
			</Field.FieldGroup>

			{#if error !== null}
				<Alert.Root variant="destructive">
					<Alert.Description>{error}</Alert.Description>
				</Alert.Root>
			{/if}

			<Button type="submit" class="w-full" disabled={submitting || !mounted}>
				{#if submitting}
					<Loader2Icon class="animate-spin" />
				{/if}
				Sign in
			</Button>

			<p class="text-muted-foreground flex items-start gap-2 text-xs">
				<InfoIcon class="mt-px size-3.5 shrink-0" />
				<span>New account or forgotten password? Contact the lab administrator.</span>
			</p>
		</form>
	</Card.Content>
</Card.Root>
