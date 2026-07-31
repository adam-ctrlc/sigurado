<script lang="ts">
	import { onMount } from 'svelte';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import MailIcon from '@lucide/svelte/icons/mail';
	import AtSignIcon from '@lucide/svelte/icons/at-sign';
	import CalendarIcon from '@lucide/svelte/icons/calendar';
	import PhoneIcon from '@lucide/svelte/icons/phone';
	import ShieldIcon from '@lucide/svelte/icons/shield';
	import PackageOpenIcon from '@lucide/svelte/icons/package-open';
	import ImageOffIcon from '@lucide/svelte/icons/image-off';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import UserIcon from '@lucide/svelte/icons/user';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import EyeIcon from '@lucide/svelte/icons/eye';
	import EyeOffIcon from '@lucide/svelte/icons/eye-off';
	import KeyRoundIcon from '@lucide/svelte/icons/key-round';
	import WandSparklesIcon from '@lucide/svelte/icons/wand-sparkles';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import * as Card from '$lib/components/ui/card';
	import * as Avatar from '$lib/components/ui/avatar';
	import * as Field from '$lib/components/ui/field';
	import * as Empty from '$lib/components/ui/empty';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as InputGroup from '$lib/components/ui/input-group';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import RoleBadge from '$lib/components/role-badge.svelte';
	import { auth } from '$lib/stores/auth.svelte';
	import { myEnrollments } from '$lib/api/enrollment';
	import { myLogins, updateMe, changePassword, type UpdateMeInput } from '$lib/api/auth';
	import { ApiError } from '$lib/api/client';
	import { toast } from 'svelte-sonner';
	import { listCheckouts } from '$lib/api/checkouts';
	import { API_BASE } from '$lib/api/client';
	import type { Enrollment } from '$lib/schemas/enrollment';
	import type { LoginEvent } from '$lib/schemas/user';
	import { describeUserAgent } from '$lib/user-agent';
	import PasswordRules from '$lib/components/password-rules.svelte';
	import TablePager from '$lib/components/table-pager.svelte';
	import { MAX_LENGTH, MIN_LENGTH, passwordOk } from '$lib/password';
	import type { Checkout } from '$lib/schemas/checkout';
	import { formatDate, formatDateTime } from '$lib/format';

	const filesOrigin = API_BASE.replace(/\/api\/?$/, '');

	let enrollments = $state<Enrollment[]>([]);
	let checkouts = $state<Checkout[]>([]);
	let logins = $state<LoginEvent[]>([]);
	let loginsPage = $state(1);
	let loginsTotal = $state(0);
	let loginsLoading = $state(false);
	const LOGINS_PER_PAGE = 5;

	async function refreshLogins(): Promise<void> {
		loginsLoading = true;
		try {
			const result = await myLogins({ page: loginsPage, per_page: LOGINS_PER_PAGE });
			logins = result.items;
			loginsTotal = result.total;
			loginsPage = result.page;
		} catch {
			// keep whatever is on screen
		} finally {
			loginsLoading = false;
		}
	}

	let editOpen = $state(false);
	let savingProfile = $state(false);
	let form = $state({
		first_name: '',
		middle_name: '',
		last_name: '',
		suffix: '',
		username: '',
		email: '',
		phone_number: ''
	});

	// Only offer to build a username once there is a name to build it from.
	const canSuggest = $derived(
		form.first_name.trim().length > 0 && form.last_name.trim().length > 0
	);

	let passwordOpen = $state(false);
	let savingPassword = $state(false);
	let currentPassword = $state('');
	let newPassword = $state('');
	let confirmPassword = $state('');
	let showNew = $state(false);

	const passwordMismatch = $derived(
		confirmPassword.length > 0 && newPassword !== confirmPassword
	);
	const newPasswordOk = $derived(passwordOk(newPassword));
	let loading = $state(true);

	const user = $derived(auth.user);
	const initials = $derived(
		(user?.display_name ?? '?')
			.split(' ')
			.map((part) => part[0] ?? '')
			.join('')
			.slice(0, 2)
			.toUpperCase()
	);
	const boundCount = $derived(enrollments.filter((e) => e.status === 'bound').length);
	const myCheckouts = $derived(user ? checkouts.filter((c) => c.user_id === user.id) : []);

	const stats = $derived([
		{ label: 'Fingers enrolled', value: boundCount },
		{ label: 'Checkouts', value: myCheckouts.length },
		{
			label: 'Last checkout',
			text: myCheckouts[0] ? formatDate(myCheckouts[0].created_at) : 'None yet'
		}
	]);

	function openEdit(): void {
		form = {
			first_name: user?.first_name ?? '',
			middle_name: user?.middle_name ?? '',
			last_name: user?.last_name ?? '',
			suffix: user?.suffix ?? '',
			username: user?.username ?? '',
			email: user?.email ?? '',
			phone_number: user?.phone_number ?? ''
		};
		editOpen = true;
	}

	/** "Juan", "Ramos", "Dela Cruz" -> "juan.r.delacruz" */
	function suggestUsername(): void {
		const clean = (value: string): string =>
			value
				.normalize('NFD')
				.replace(/[\u0300-\u036f]/g, '')
				.toLowerCase()
				.replace(/[^a-z0-9]/g, '');

		const first = clean(form.first_name);
		const last = clean(form.last_name);
		if (first.length === 0 || last.length === 0) return;

		const middle = clean(form.middle_name).slice(0, 1);
		form.username = [first, middle, last].filter((part) => part.length > 0).join('.');
	}

	async function saveProfile(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (savingProfile) return;
		savingProfile = true;
		try {
			const input: UpdateMeInput = {
				phone_number: form.phone_number.trim(),
				first_name: form.first_name.trim(),
				middle_name: form.middle_name.trim(),
				last_name: form.last_name.trim(),
				suffix: form.suffix.trim(),
				username: form.username.trim(),
				email: form.email.trim()
			};
			auth.setUser(await updateMe(input));
			toast.success('Profile updated.');
			editOpen = false;
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not save your profile');
		} finally {
			savingProfile = false;
		}
	}

	function openPassword(): void {
		currentPassword = '';
		newPassword = '';
		confirmPassword = '';
		showNew = false;
		passwordOpen = true;
	}

	async function savePassword(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (savingPassword || passwordMismatch || !newPasswordOk) return;
		savingPassword = true;
		try {
			await changePassword(currentPassword, newPassword);
			toast.success('Password changed.');
			passwordOpen = false;
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not change your password');
		} finally {
			savingPassword = false;
		}
	}

	function photoUrl(path: string): string {
		return `${filesOrigin}${path}`;
	}

	onMount(async () => {
		const [e, c] = await Promise.allSettled([myEnrollments(), listCheckouts()]);
		if (e.status === 'fulfilled') enrollments = e.value;
		if (c.status === 'fulfilled') checkouts = c.value.items;
		await refreshLogins();
		loading = false;
	});
</script>

<svelte:head><title>Profile - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div>
		<h1 class="text-2xl font-semibold tracking-tight">Profile</h1>
		<p class="text-muted-foreground text-sm">
			Your account, the fingers bound to it, and what you have taken out.
		</p>
	</div>

	<!-- identity -->
	<Card.Root>
		<Card.Content class="flex flex-col gap-6 py-6">
			<div class="flex flex-col items-start gap-4 sm:flex-row sm:items-center">
				<Avatar.Root class="size-16">
					<Avatar.Fallback class="text-lg">{initials}</Avatar.Fallback>
				</Avatar.Root>
				<div class="flex flex-col gap-1.5">
					<h2 class="font-display text-xl font-semibold">{user?.display_name ?? ''}</h2>
					<div class="flex flex-wrap items-center gap-2">
						{#if user}
							<RoleBadge role={user.role} />
						{/if}
						<Badge variant={user?.is_active ? 'secondary' : 'outline'}>
							{user?.is_active ? 'Active' : 'Disabled'}
						</Badge>
					</div>
				</div>
			</div>

			<Separator />

			<div class="grid grid-cols-2 gap-6 md:grid-cols-3">
				{#each stats as stat (stat.label)}
					<div class="flex flex-col gap-1">
						<span class="text-muted-foreground text-xs tracking-wide uppercase">{stat.label}</span>
						{#if loading}
							<Skeleton class="h-8 w-20" />
						{:else if stat.text !== undefined}
							<span class="text-lg font-medium">{stat.text}</span>
						{:else}
							<span class="text-3xl font-semibold tabular-nums">{stat.value}</span>
						{/if}
					</div>
				{/each}
			</div>
		</Card.Content>
	</Card.Root>

	<!-- account -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Account</Card.Title>
			<Card.Description>
				Your name and email are yours to change. Username and role stay with the administrator.
			</Card.Description>
			<Card.Action>
				<div class="flex flex-wrap items-center gap-2">
					<Button variant="outline" size="sm" onclick={openEdit}>
						<PencilIcon />
						Edit profile
					</Button>
					<Button variant="outline" size="sm" onclick={openPassword}>
						<KeyRoundIcon />
						Change password
					</Button>
				</div>
			</Card.Action>
		</Card.Header>
		<Card.Content>
			<Field.FieldGroup>
				<div class="grid gap-6 md:grid-cols-3">
					<Field.Field>
						<Field.FieldLabel>
							<UserIcon class="size-3.5" /> First name
						</Field.FieldLabel>
						<p class="text-sm">{user?.first_name ?? ''}</p>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel>
							<UserIcon class="size-3.5" /> Middle name
						</Field.FieldLabel>
						<p class="text-sm">{user?.middle_name ?? '-'}</p>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel>
							<UserIcon class="size-3.5" /> Last name
						</Field.FieldLabel>
						<p class="text-sm">
							{user?.last_name ?? ''}{user?.suffix ? `, ${user.suffix}` : ''}
						</p>
					</Field.Field>
				</div>

				<Field.Separator />

				<div class="grid gap-6 md:grid-cols-2">
					<Field.Field>
						<Field.FieldLabel>
							<AtSignIcon class="size-3.5" /> Username
						</Field.FieldLabel>
						<p class="text-sm">{user?.username ?? ''}</p>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel>
							<MailIcon class="size-3.5" /> Email
						</Field.FieldLabel>
						<p class="text-sm">{user?.email ?? ''}</p>
					</Field.Field>
				</div>

				<Field.Separator />

				<div class="grid gap-6 md:grid-cols-2">
					<Field.Field>
						<Field.FieldLabel>
							<ShieldIcon class="size-3.5" /> Role
						</Field.FieldLabel>
						<p class="text-sm capitalize">{user?.role ?? ''}</p>
						<Field.FieldDescription>
							Determines which pages and actions are available to you.
						</Field.FieldDescription>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel>
							<PhoneIcon class="size-3.5" /> Mobile number
						</Field.FieldLabel>
						<p class="text-sm">{user?.phone_number ?? 'Not set'}</p>
					</Field.Field>
				</div>

				<Field.Separator />

				<div class="grid gap-6 md:grid-cols-2">
					<Field.Field>
						<Field.FieldLabel>
							<CalendarIcon class="size-3.5" /> Member since
						</Field.FieldLabel>
						<p class="text-sm">{formatDateTime(user?.created_at)}</p>
					</Field.Field>
				</div>
			</Field.FieldGroup>
		</Card.Content>
	</Card.Root>

	<!-- fingerprints -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Fingerprints</Card.Title>
			<Card.Description>
				Enroll once at each reader: the door and the cabinet store templates separately.
			</Card.Description>
			<Card.Action>
				<Button href="/enrollment" variant="outline" size="sm">Enroll a fingerprint</Button>
			</Card.Action>
		</Card.Header>
		<Card.Content class="flex flex-col gap-3">
			{#if loading}
				{#each Array.from({ length: 2 }) as _, i (i)}
					<div class="flex items-center justify-between gap-2">
						<Skeleton class="h-4 w-40" />
						<Skeleton class="h-5 w-20" />
					</div>
				{/each}
			{:else if enrollments.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><FingerprintIcon /></Empty.Media>
						<Empty.Title>Not enrolled yet</Empty.Title>
						<Empty.Description>
							Bind a finger at the door and again at the cabinet to use both.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each enrollments as item, i (item.id)}
					{#if i > 0}<Separator />{/if}
					<div class="flex items-start justify-between gap-3">
						<div class="flex min-w-0 flex-col gap-0.5">
							<span class="truncate text-sm">
								{item.status === 'bound' ? 'Finger bound' : 'Enrollment started'}
							</span>
							<span class="text-muted-foreground text-xs">{formatDateTime(item.created_at)}</span>
						</div>
						<Badge variant={item.status === 'bound' ? 'secondary' : 'outline'} class="shrink-0">
							{item.status === 'bound' ? 'Done' : item.status.replaceAll('_', ' ')}
						</Badge>
					</div>
				{/each}
			{/if}
		</Card.Content>
	</Card.Root>

	<!-- checkouts -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Your checkouts</Card.Title>
			<Card.Description>Materials you have taken from the cabinet, most recent first.</Card.Description>
			<Card.Action>
				<Button href="/checkout" variant="outline" size="sm">Record a checkout</Button>
			</Card.Action>
		</Card.Header>
		<Card.Content class="flex flex-col gap-4">
			{#if loading}
				{#each Array.from({ length: 2 }) as _, i (i)}
					<div class="flex gap-3">
						<Skeleton class="size-14 rounded-lg" />
						<div class="flex flex-1 flex-col gap-2">
							<Skeleton class="h-4 w-full" />
							<Skeleton class="h-3 w-24" />
						</div>
					</div>
				{/each}
			{:else if myCheckouts.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><PackageOpenIcon /></Empty.Media>
						<Empty.Title>Nothing checked out yet</Empty.Title>
						<Empty.Description>
							Anything you take from the cabinet gets recorded here with a timestamp.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each myCheckouts.slice(0, 6) as item, i (item.id)}
					{#if i > 0}<Separator />{/if}
					<div class="flex items-start gap-3">
						{#if item.photo_path}
							<img
								src={photoUrl(item.photo_path)}
								alt="Materials taken"
								class="size-14 shrink-0 rounded-lg border object-cover"
							/>
						{:else}
							<div
								class="bg-muted text-muted-foreground flex size-14 shrink-0 items-center justify-center rounded-lg border"
							>
								<ImageOffIcon class="size-5" />
							</div>
						{/if}
						<div class="flex min-w-0 flex-1 flex-col gap-1">
							<p class="text-sm leading-relaxed">{item.note}</p>
							<span class="text-muted-foreground text-xs">{formatDateTime(item.created_at)}</span>
						</div>
					</div>
				{/each}
			{/if}
		</Card.Content>
	</Card.Root>

	<!-- sign-in history -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Recent sign-ins</Card.Title>
		<Card.Description>
				Every attempt on your account, newest first. Anything you do not recognize is worth
				reporting.
			</Card.Description>
		</Card.Header>
		<Card.Content class="flex flex-col gap-3">
		{#if loading || loginsLoading}
				{#each Array.from({ length: 3 }) as _, i (i)}
					<div class="flex flex-col gap-2">
						<Skeleton class="h-4 w-48" />
						<Skeleton class="h-3 w-32" />
					</div>
				{/each}
			{:else if logins.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><MonitorIcon /></Empty.Media>
						<Empty.Title>Nothing recorded yet</Empty.Title>
						<Empty.Description>
							Sign-ins are logged with the address and browser they came from.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				{#each logins as item, i (item.id)}
					{#if i > 0}<Separator />{/if}
					<div class="flex items-start justify-between gap-3">
						<div class="flex min-w-0 flex-col gap-0.5">
							<span class="text-sm">{formatDateTime(item.created_at)}</span>
							<span class="text-muted-foreground truncate text-xs">
								{describeUserAgent(item.user_agent)} - {item.ip_address ?? 'unknown address'}
							</span>
						</div>
						<Badge variant={item.success ? 'secondary' : 'outline'} class="shrink-0">
							{item.success ? 'Success' : 'Failed'}
						</Badge>
					</div>
				{/each}

				<TablePager
					total={loginsTotal}
					perPage={LOGINS_PER_PAGE}
					page={loginsPage}
					noun="sign-ins"
					onPage={(next) => {
						loginsPage = next;
						void refreshLogins();
					}}
				/>
			{/if}
		</Card.Content>
	</Card.Root>
</div>

<Dialog.Root bind:open={editOpen}>
	<Dialog.Content class="sm:max-w-md">
		<form onsubmit={saveProfile} class="grid gap-4">
			<Dialog.Header>
				<Dialog.Title>Edit your profile</Dialog.Title>
			</Dialog.Header>

			<Separator class="-mx-4 w-auto" />

			<div class="grid gap-4">
				<Dialog.Description>
					Only your role is fixed. Ask an administrator if that needs to change.
				</Dialog.Description>

				<Field.FieldGroup>
					<div class="grid gap-4 sm:grid-cols-2">
						<Field.Field>
							<Field.FieldLabel for="my_first_name">First name</Field.FieldLabel>
							<Input id="my_first_name" bind:value={form.first_name} required />
						</Field.Field>

						<Field.Field>
							<Field.FieldLabel for="my_middle_name">Middle name</Field.FieldLabel>
							<Input id="my_middle_name" bind:value={form.middle_name} />
							<Field.FieldDescription>Optional.</Field.FieldDescription>
						</Field.Field>
					</div>

					<div class="grid gap-4 sm:grid-cols-[1fr_7rem]">
						<Field.Field>
							<Field.FieldLabel for="my_last_name">Last name</Field.FieldLabel>
							<Input id="my_last_name" bind:value={form.last_name} required />
						</Field.Field>

						<Field.Field>
							<Field.FieldLabel for="my_suffix">Suffix</Field.FieldLabel>
							<Input id="my_suffix" bind:value={form.suffix} placeholder="Jr." />
						</Field.Field>
					</div>

					<Field.Field>
						<Field.FieldLabel for="my_username">Username</Field.FieldLabel>
						<InputGroup.Root>
							<InputGroup.Input
								id="my_username"
								bind:value={form.username}
								autocomplete="username"
								minlength={3}
								required
							/>
							<InputGroup.Addon align="inline-end">
								<InputGroup.Button
									type="button"
									size="xs"
									disabled={!canSuggest}
									onclick={suggestUsername}
								>
									<WandSparklesIcon />
									Generate
								</InputGroup.Button>
							</InputGroup.Addon>
						</InputGroup.Root>
						<Field.FieldDescription>
							{canSuggest
								? 'Generate builds one from your name, for example juan.r.delacruz.'
								: 'Fill in your first and last name to generate one.'}
						</Field.FieldDescription>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel for="my_phone">Mobile number</Field.FieldLabel>
						<Input
							id="my_phone"
							bind:value={form.phone_number}
							placeholder="09171234567"
							inputmode="tel"
						/>
						<Field.FieldDescription>
							Optional. Used for SMS alerts if you are faculty or an admin.
						</Field.FieldDescription>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel for="my_email">Email</Field.FieldLabel>
						<Input id="my_email" type="email" bind:value={form.email} required />
						<Field.FieldDescription>You can sign in with this or your username.</Field.FieldDescription>
					</Field.Field>
				</Field.FieldGroup>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (editOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={savingProfile}>
					{#if savingProfile}<Loader2Icon class="animate-spin" />{/if}
					Save changes
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<Dialog.Root bind:open={passwordOpen}>
	<Dialog.Content class="sm:max-w-md">
		<form onsubmit={savePassword} class="grid gap-4">
			<Dialog.Header>
				<Dialog.Title>Change your password</Dialog.Title>
			</Dialog.Header>

			<Separator class="-mx-4 w-auto" />

			<div class="grid gap-4">
				<Dialog.Description>
					Enter your current password first, then the new one twice.
				</Dialog.Description>

				<Field.FieldGroup>
					<Field.Field>
						<Field.FieldLabel for="current_password">Current password</Field.FieldLabel>
						<Input
							id="current_password"
							type="password"
							bind:value={currentPassword}
							autocomplete="current-password"
							required
						/>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel for="new_password">New password</Field.FieldLabel>
						<InputGroup.Root>
							<InputGroup.Input
								id="new_password"
								type={showNew ? 'text' : 'password'}
								bind:value={newPassword}
								autocomplete="new-password"
								minlength={MIN_LENGTH}
								maxlength={MAX_LENGTH}
								required
							/>
							<InputGroup.Addon align="inline-end">
								<InputGroup.Button
									type="button"
									size="icon-xs"
									onclick={() => (showNew = !showNew)}
									aria-label={showNew ? 'Hide password' : 'Show password'}
								>
									{#if showNew}
										<EyeOffIcon />
									{:else}
										<EyeIcon />
									{/if}
								</InputGroup.Button>
							</InputGroup.Addon>
						</InputGroup.Root>
						<PasswordRules password={newPassword} showWhenEmpty />
					</Field.Field>

					<Field.Field data-invalid={passwordMismatch ? '' : undefined}>
						<Field.FieldLabel for="confirm_password">Confirm new password</Field.FieldLabel>
						<Input
							id="confirm_password"
							type="password"
							bind:value={confirmPassword}
							autocomplete="new-password"
							aria-invalid={passwordMismatch ? 'true' : undefined}
							required
						/>
						{#if passwordMismatch}
							<Field.FieldError>The two passwords do not match.</Field.FieldError>
						{/if}
					</Field.Field>
				</Field.FieldGroup>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (passwordOpen = false)}>
					Cancel
				</Button>
				<Button
					type="submit"
					disabled={savingPassword || passwordMismatch || !newPasswordOk}
				>
					{#if savingPassword}<Loader2Icon class="animate-spin" />{/if}
					Change password
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>
