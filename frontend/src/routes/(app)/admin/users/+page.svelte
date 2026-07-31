<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { toast } from 'svelte-sonner';
	import UserPlusIcon from '@lucide/svelte/icons/user-plus';
	import PencilIcon from '@lucide/svelte/icons/pencil';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';
	import UserCheckIcon from '@lucide/svelte/icons/user-check';
	import UserXIcon from '@lucide/svelte/icons/user-x';
	import UsersIcon from '@lucide/svelte/icons/users';
	import SearchIcon from '@lucide/svelte/icons/search';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';
	import MoreHorizontalIcon from '@lucide/svelte/icons/more-horizontal';
	import HistoryIcon from '@lucide/svelte/icons/history';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Field from '$lib/components/ui/field';
	import * as Select from '$lib/components/ui/select';
	import * as Empty from '$lib/components/ui/empty';
	import * as Avatar from '$lib/components/ui/avatar';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import RoleBadge from '$lib/components/role-badge.svelte';
	import PasswordRules from '$lib/components/password-rules.svelte';
	import { MAX_LENGTH, MIN_LENGTH, passwordOk } from '$lib/password';
	import TablePager from '$lib/components/table-pager.svelte';
	import {
		listUsers,
		createUser,
		updateUser,
		deleteUser,
		listUserLogins,
		type CreateUserInput,
		type UpdateUserInput
	} from '$lib/api/users';
	import { ApiError } from '$lib/api/client';
	import { auth } from '$lib/stores/auth.svelte';
	import { formatDate, formatDateTime } from '$lib/format';
	import { describeUserAgent } from '$lib/user-agent';
	import type { User, Role, LoginEvent } from '$lib/schemas/user';

	let users = $state<User[]>([]);
	let loading = $state(true);
	let dialogOpen = $state(false);
	let submitting = $state(false);
	let editing = $state<User | null>(null);
	let removing = $state<User | null>(null);
	let deletingNow = $state(false);
	let historyFor = $state<User | null>(null);
	let logins = $state<LoginEvent[]>([]);
	let loginsLoading = $state(false);
	let loginsPage = $state(1);
	let loginsTotal = $state(0);
	const LOGINS_PER_PAGE = 6;
	let query = $state('');
	let roleFilter = $state<'all' | Role>('all');
	let page = $state(1);
	let total = $state(0);
	let counts = $state({ all: 0, active: 0, faculty: 0, admin: 0 });
	const PER_PAGE = 10;
	let searchTimer: ReturnType<typeof setTimeout> | undefined;

	const blank: CreateUserInput = {
		email: '',
		username: '',
		password: '',
		phone_number: '',
		first_name: '',
		middle_name: '',
		last_name: '',
		suffix: '',
		role: 'student'
	};
	let form = $state<CreateUserInput>({ ...blank });

	const roleOptions = [
		{ value: 'student', label: 'Student' },
		{ value: 'faculty', label: 'Faculty' },
		{ value: 'admin', label: 'Admin' }
	] as const;

	const roleLabels: Record<string, string> = {
		student: 'Student',
		faculty: 'Faculty',
		admin: 'Admin'
	};

	const filterOptions = [
		{ value: 'all', label: 'All' },
		{ value: 'student', label: 'Students' },
		{ value: 'faculty', label: 'Faculty' },
		{ value: 'admin', label: 'Admins' }
	] as const;

	const filterLabels: Record<string, string> = {
		all: 'All roles',
		student: 'Students',
		faculty: 'Faculty',
		admin: 'Admins'
	};



	// An admin must not be able to lock themselves out of their own console.
	const isSelf = (user: User): boolean => user.id === auth.user?.id;
	const editingSelf = $derived(editing !== null && isSelf(editing));
	// Editing may leave the password untouched; creating may not.
	const passwordAcceptable = $derived(
		editing !== null && form.password.length === 0 ? true : passwordOk(form.password)
	);

	// Totals describe the whole roster, so they come from counting queries rather
	// than from whatever page happens to be on screen.
	const stats = $derived([
		{ label: 'Accounts', value: counts.all },
		{ label: 'Active', value: counts.active },
		{ label: 'Faculty', value: counts.faculty },
		{ label: 'Admins', value: counts.admin }
	]);

	function initials(name: string): string {
		return name
			.split(/\s+/)
			.filter(Boolean)
			.slice(0, 2)
			.map((part) => part[0]?.toUpperCase() ?? '')
			.join('');
	}

	async function refresh(): Promise<void> {
		try {
			const result = await listUsers({
				page,
				per_page: PER_PAGE,
				q: query.trim() || undefined,
				role: roleFilter === 'all' ? undefined : roleFilter
			});
			users = result.items;
			total = result.total;
			page = result.page;
		} catch {
			// keep the current list; a transient failure should not blank the page
		} finally {
			loading = false;
		}
	}

	/** Four cheap count-only queries, so the strip is not limited to one page. */
	async function refreshCounts(): Promise<void> {
		try {
			const [all, active, faculty, admin] = await Promise.all([
				listUsers({ per_page: 1 }),
				listUsers({ per_page: 1, active: true }),
				listUsers({ per_page: 1, role: 'faculty' }),
				listUsers({ per_page: 1, role: 'admin' })
			]);
			counts = {
				all: all.total,
				active: active.total,
				faculty: faculty.total,
				admin: admin.total
			};
		} catch {
			// leave the previous counts in place
		}
	}

	async function reload(): Promise<void> {
		await Promise.all([refresh(), refreshCounts()]);
	}

	/** Typing should not fire a request per keystroke. */
	function onSearch(): void {
		clearTimeout(searchTimer);
		searchTimer = setTimeout(() => {
			page = 1;
			void refresh();
		}, 300);
	}

	function onRoleChange(): void {
		page = 1;
		void refresh();
	}

	function openCreate(): void {
		editing = null;
		form = { ...blank };
		dialogOpen = true;
	}

	function openEdit(user: User): void {
		editing = user;
		form = {
			email: user.email,
			username: user.username,
			password: '',
			phone_number: user.phone_number ?? '',
			first_name: user.first_name,
			middle_name: user.middle_name ?? '',
			last_name: user.last_name,
			suffix: user.suffix ?? '',
			role: user.role
		};
		dialogOpen = true;
	}

	async function openHistory(user: User): Promise<void> {
		historyFor = user;
		logins = [];
		loginsPage = 1;
		loginsTotal = 0;
		await loadLogins();
	}

	async function loadLogins(): Promise<void> {
		if (!historyFor) return;
		loginsLoading = true;
		try {
			const result = await listUserLogins(historyFor.id, {
				page: loginsPage,
				per_page: LOGINS_PER_PAGE
			});
			logins = result.items;
			loginsTotal = result.total;
			loginsPage = result.page;
		} catch {
			// the dialog falls back to its empty state
		} finally {
			loginsLoading = false;
		}
	}

	async function save(event: SubmitEvent): Promise<void> {
		event.preventDefault();
		if (submitting) return;
		submitting = true;
		try {
			if (editing) {
				const patch: UpdateUserInput = {
					email: form.email.trim(),
					username: form.username.trim(),
					phone_number: form.phone_number?.trim() ?? '',
					first_name: form.first_name.trim(),
					middle_name: form.middle_name?.trim() ?? '',
					last_name: form.last_name.trim(),
					suffix: form.suffix?.trim() ?? ''
				};
				if (!isSelf(editing)) patch.role = form.role;
				// Blank means "keep the current password" rather than "clear it".
				if (form.password.length > 0) patch.password = form.password;
				await updateUser(editing.id, patch);
				toast.success(`Saved ${form.username}`);
			} else {
				await createUser({
					...form,
					phone_number: form.phone_number?.trim() || undefined,
					first_name: form.first_name.trim(),
					middle_name: form.middle_name?.trim(),
					last_name: form.last_name.trim(),
					suffix: form.suffix?.trim()
				});
				toast.success(`Created ${form.username}`);
			}
			form = { ...blank };
			editing = null;
			dialogOpen = false;
			await reload();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not save the user');
		} finally {
			submitting = false;
		}
	}

	async function toggleActive(user: User): Promise<void> {
		if (isSelf(user)) return;
		try {
			await updateUser(user.id, { is_active: !user.is_active });
			toast.success(`${user.display_name} is now ${user.is_active ? 'disabled' : 'active'}`);
			await reload();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Update failed');
		}
	}

	async function confirmDelete(): Promise<void> {
		if (!removing || deletingNow || isSelf(removing)) return;
		deletingNow = true;
		try {
			await deleteUser(removing.id);
			toast.success(`Deleted ${removing.display_name}`);
			removing = null;
			await reload();
		} catch (err) {
			toast.error(err instanceof ApiError ? err.message : 'Could not delete the user');
		} finally {
			deletingNow = false;
		}
	}

	onMount(() => {
		if (!auth.isAdmin) {
			void goto('/dashboard');
			return;
		}
		void reload();
	});
</script>

<svelte:head><title>Users - Sigurado</title></svelte:head>

<div class="flex flex-col gap-6">
	<div class="flex flex-wrap items-start justify-between gap-4">
		<div>
			<h1 class="text-2xl font-semibold tracking-tight">Users</h1>
			<p class="text-muted-foreground text-sm">
				Accounts that can sign in, enroll a finger, and check out materials.
			</p>
		</div>
		<Button onclick={openCreate}>
			<UserPlusIcon />
			Add user
		</Button>
	</div>

	<!-- roster at a glance -->
	<Card.Root>
		<Card.Content class="grid grid-cols-2 gap-6 py-6 md:grid-cols-4">
			{#each stats as stat (stat.label)}
				<div class="flex flex-col gap-1">
					<span class="text-muted-foreground text-xs tracking-wide uppercase">{stat.label}</span>
					{#if loading}
						<Skeleton class="h-8 w-12" />
					{:else}
						<span class="text-3xl font-semibold tabular-nums">{stat.value}</span>
					{/if}
				</div>
			{/each}
		</Card.Content>
	</Card.Root>

	<!-- roster -->
	<Card.Root>
		<Card.Header>
			<Card.Title>Roster</Card.Title>
			<Card.Description>Newest accounts appear as they are created.</Card.Description>
		</Card.Header>

		<Card.Content class="flex flex-col gap-4">
			<div class="flex flex-wrap items-center gap-3">
				<div class="relative min-w-56 flex-1">
					<SearchIcon
						class="text-muted-foreground pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2"
					/>
					<Input
						bind:value={query}
						oninput={onSearch}
						placeholder="Search name, username, or email"
						aria-label="Search users"
						class="pl-9"
					/>
				</div>
				<Select.Root type="single" bind:value={roleFilter} onValueChange={onRoleChange}>
					<Select.Trigger class="w-40">{filterLabels[roleFilter]}</Select.Trigger>
					<Select.Content>
						<Select.Group>
							{#each filterOptions as option (option.value)}
								<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
							{/each}
						</Select.Group>
					</Select.Content>
				</Select.Root>
			</div>

			<Separator />

			{#if loading}
				<div class="flex flex-col gap-4 py-2">
					{#each Array.from({ length: 4 }) as _, i (i)}
						<div class="flex items-center gap-3">
							<Skeleton class="size-9 rounded-full" />
							<div class="flex flex-1 flex-col gap-2">
								<Skeleton class="h-4 w-40" />
								<Skeleton class="h-3 w-24" />
							</div>
							<Skeleton class="h-5 w-16" />
						</div>
					{/each}
				</div>
			{:else if users.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><UsersIcon /></Empty.Media>
						<Empty.Title>
							{total === 0 && query.trim().length === 0 && roleFilter === 'all'
							? 'No users yet'
							: 'No matching users'}
						</Empty.Title>
						<Empty.Description>
							{users.length === 0
								? 'Add the first account so someone can enroll a fingerprint.'
								: 'Try a different search term or clear the role filter.'}
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="overflow-hidden rounded-lg border">
					<Table.Root class="min-w-[54rem]">
						<Table.Header>
							<Table.Row class="hover:bg-transparent">
								<Table.Head>Name</Table.Head>
								<Table.Head>Email</Table.Head>
								<Table.Head>Role</Table.Head>
								<Table.Head>Status</Table.Head>
								<Table.Head>Last sign-in</Table.Head>
								<Table.Head class="w-12"><span class="sr-only">Actions</span></Table.Head>
							</Table.Row>
						</Table.Header>
						<Table.Body>
							{#each users as user (user.id)}
								<Table.Row>
									<Table.Cell>
										<div class="flex items-center gap-3">
											<Avatar.Root class="size-9">
												<Avatar.Fallback class="text-xs">{initials(user.display_name)}</Avatar.Fallback>
											</Avatar.Root>
											<div class="flex min-w-0 flex-col">
												<div class="flex items-center gap-2">
													<span class="truncate font-medium">{user.display_name}</span>
													{#if isSelf(user)}
														<Badge variant="outline" class="h-5 px-1.5 text-[10px]">You</Badge>
													{/if}
												</div>
												<span class="text-muted-foreground truncate text-xs">@{user.username}</span>
											</div>
										</div>
									</Table.Cell>
									<Table.Cell class="text-muted-foreground">
										{user.email}
									</Table.Cell>
									<Table.Cell><RoleBadge role={user.role} /></Table.Cell>
									<Table.Cell>
										{#if user.is_active}
											<Badge variant="secondary">Active</Badge>
										{:else}
											<Badge variant="outline" class="text-muted-foreground">Disabled</Badge>
										{/if}
									</Table.Cell>
									<Table.Cell class="text-muted-foreground text-sm">
										{user.last_login_at ? formatDate(user.last_login_at) : 'Never'}
									</Table.Cell>
									<Table.Cell>
										<DropdownMenu.Root>
											<DropdownMenu.Trigger>
												{#snippet child({ props })}
													<Button {...props} variant="ghost" size="icon">
														<MoreHorizontalIcon />
														<span class="sr-only">Actions for {user.display_name}</span>
													</Button>
												{/snippet}
											</DropdownMenu.Trigger>
											<DropdownMenu.Content align="end">
												<DropdownMenu.Group>
													<DropdownMenu.Item onSelect={() => openEdit(user)}>
														<PencilIcon />
														Edit details
													</DropdownMenu.Item>
													<DropdownMenu.Item onSelect={() => openHistory(user)}>
														<HistoryIcon />
														Sign-in history
													</DropdownMenu.Item>
													{#if !isSelf(user)}
														<DropdownMenu.Item onSelect={() => toggleActive(user)}>
															{#if user.is_active}
																<UserXIcon />
																Disable account
															{:else}
																<UserCheckIcon />
																Enable account
															{/if}
														</DropdownMenu.Item>
													{/if}
												</DropdownMenu.Group>
												{#if !isSelf(user)}
													<DropdownMenu.Separator />
													<DropdownMenu.Group>
														<DropdownMenu.Item
															variant="destructive"
															onSelect={() => (removing = user)}
														>
															<Trash2Icon />
															Delete user
														</DropdownMenu.Item>
													</DropdownMenu.Group>
												{/if}
											</DropdownMenu.Content>
										</DropdownMenu.Root>
									</Table.Cell>
								</Table.Row>
							{/each}
						</Table.Body>
					</Table.Root>
				</div>

				<TablePager
					{total}
					perPage={PER_PAGE}
					{page}
					noun="users"
					onPage={(next) => {
						page = next;
						void refresh();
					}}
				/>
			{/if}
		</Card.Content>
	</Card.Root>
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<form onsubmit={save} class="grid gap-4">
			<Dialog.Header>
				<Dialog.Title>{editing ? 'Edit user' : 'Add a user'}</Dialog.Title>
			</Dialog.Header>

			<Separator class="-mx-4 w-auto" />

			<div class="grid gap-4">
				<Dialog.Description>
					{editing
						? 'Changes apply the next time they sign in.'
						: 'The account can enroll a fingerprint after signing in.'}
				</Dialog.Description>
				<Field.FieldGroup>
					<div class="grid gap-4 sm:grid-cols-2">
						<Field.Field>
							<Field.FieldLabel for="first_name">First name</Field.FieldLabel>
							<Input id="first_name" bind:value={form.first_name} placeholder="Juan" required />
						</Field.Field>

						<Field.Field>
							<Field.FieldLabel for="middle_name">Middle name</Field.FieldLabel>
							<Input id="middle_name" bind:value={form.middle_name} placeholder="Ramos" />
							<Field.FieldDescription>Optional.</Field.FieldDescription>
						</Field.Field>
					</div>

					<div class="grid gap-4 sm:grid-cols-[1fr_7rem]">
						<Field.Field>
							<Field.FieldLabel for="last_name">Last name</Field.FieldLabel>
							<Input id="last_name" bind:value={form.last_name} placeholder="Dela Cruz" required />
						</Field.Field>

						<Field.Field>
							<Field.FieldLabel for="suffix">Suffix</Field.FieldLabel>
							<Input id="suffix" bind:value={form.suffix} placeholder="Jr." />
						</Field.Field>
					</div>

					<div class="grid grid-cols-2 gap-4">
						<Field.Field>
							<Field.FieldLabel for="username">Username</Field.FieldLabel>
							<Input
								id="username"
								bind:value={form.username}
								placeholder="juan.r.delacruz"
								required
							/>
						</Field.Field>

						<Field.Field data-disabled={editingSelf ? '' : undefined}>
							<Field.FieldLabel for="role">Role</Field.FieldLabel>
							<Select.Root type="single" bind:value={form.role} disabled={editingSelf}>
								<Select.Trigger id="role" class="w-full">{roleLabels[form.role]}</Select.Trigger>
								<Select.Content>
									<Select.Group>
										{#each roleOptions as option (option.value)}
											<Select.Item value={option.value} label={option.label}>
												{option.label}
											</Select.Item>
										{/each}
									</Select.Group>
								</Select.Content>
							</Select.Root>
							{#if editingSelf}
								<Field.FieldDescription>You cannot change your own role.</Field.FieldDescription>
							{/if}
						</Field.Field>
					</div>

					<Field.Field>
						<Field.FieldLabel for="phone_number">Mobile number</Field.FieldLabel>
						<Input
							id="phone_number"
							bind:value={form.phone_number}
							placeholder="09171234567"
							inputmode="tel"
						/>
						<Field.FieldDescription>
							Optional. Faculty and admin numbers can receive SMS alerts.
						</Field.FieldDescription>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel for="email">Email</Field.FieldLabel>
						<Input
							id="email"
							type="email"
							bind:value={form.email}
							placeholder="juan.delacruz@school.edu"
							required
						/>
						<Field.FieldDescription>Used to sign in, alongside the username.</Field.FieldDescription>
					</Field.Field>

					<Field.Field>
						<Field.FieldLabel for="password">
							{editing ? 'New password' : 'Temporary password'}
						</Field.FieldLabel>
						<Input
							id="password"
							type="password"
							bind:value={form.password}
							placeholder={editing ? 'Leave blank to keep it' : 'e.g. Lab$tock1'}
							required={!editing}
							minlength={MIN_LENGTH}
							maxlength={MAX_LENGTH}
						/>
						<PasswordRules password={form.password} />
						{#if editing && form.password.length === 0}
							<Field.FieldDescription>Leave blank to keep the current one.</Field.FieldDescription>
						{/if}
					</Field.Field>
				</Field.FieldGroup>
			</div>

			<Dialog.Footer>
				<Button type="button" variant="outline" onclick={() => (dialogOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={submitting || !passwordAcceptable}>
					{#if submitting}<Loader2Icon class="animate-spin" />{/if}
					{editing ? 'Save changes' : 'Create'}
				</Button>
			</Dialog.Footer>
		</form>
	</Dialog.Content>
</Dialog.Root>

<AlertDialog.Root
	open={removing !== null}
	onOpenChange={(v) => {
		if (!v) removing = null;
	}}
>
	<AlertDialog.Content interactOutsideBehavior="close" escapeKeydownBehavior="close">
		<AlertDialog.Header>
			<AlertDialog.Title>Delete {removing?.display_name ?? 'this user'}?</AlertDialog.Title>
		</AlertDialog.Header>

		<Separator class="-mx-4 w-auto" />

		<AlertDialog.Description>
			This also removes their enrolled fingerprints and checkout records. Past audit events stay,
			but they will no longer carry a name. This cannot be undone.
		</AlertDialog.Description>

		<AlertDialog.Footer>
			<AlertDialog.Cancel disabled={deletingNow}>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action
				onclick={(event) => {
					event.preventDefault();
					void confirmDelete();
				}}
				disabled={deletingNow}
			>
				{#if deletingNow}<Loader2Icon class="animate-spin" />{/if}
				Delete user
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>

<Dialog.Root
	open={historyFor !== null}
	onOpenChange={(v) => {
		if (!v) historyFor = null;
	}}
>
	<Dialog.Content class="sm:max-w-lg">
		<Dialog.Header>
			<Dialog.Title>Sign-in history</Dialog.Title>
		</Dialog.Header>

		<Separator class="-mx-4 w-auto" />

		<div class="flex flex-col gap-4">
		<Dialog.Description>
				Every attempt for {historyFor?.display_name ?? 'this account'}, with the address and browser
				each came from.
			</Dialog.Description>

			{#if loginsLoading}
				<div class="flex flex-col gap-3">
					{#each Array.from({ length: 3 }) as _, i (i)}
						<div class="flex flex-col gap-2">
							<Skeleton class="h-4 w-48" />
							<Skeleton class="h-3 w-32" />
						</div>
					{/each}
				</div>
			{:else if logins.length === 0}
				<Empty.Root>
					<Empty.Header>
						<Empty.Media variant="icon"><MonitorIcon /></Empty.Media>
						<Empty.Title>No sign-ins recorded</Empty.Title>
						<Empty.Description>
							Attempts appear here from the next time this account signs in.
						</Empty.Description>
					</Empty.Header>
				</Empty.Root>
			{:else}
				<div class="flex max-h-80 flex-col gap-3 overflow-y-auto">
					{#each logins as item, i (item.id)}
						{#if i > 0}<Separator />{/if}
						<div class="flex items-start justify-between gap-3">
							<div class="flex min-w-0 flex-col gap-0.5">
								<span class="text-sm">{formatDateTime(item.created_at)}</span>
								<span class="text-muted-foreground truncate text-xs">
									{describeUserAgent(item.user_agent)} - {item.ip_address ?? 'unknown address'}
								</span>
								{#if item.reason}
									<span class="text-muted-foreground text-xs">{item.reason}</span>
								{/if}
							</div>
							<Badge variant={item.success ? 'secondary' : 'outline'} class="shrink-0">
								{item.success ? 'Success' : 'Failed'}
							</Badge>
						</div>
					{/each}
				</div>

				<TablePager
					total={loginsTotal}
					perPage={LOGINS_PER_PAGE}
					page={loginsPage}
					noun="sign-ins"
					onPage={(next) => {
						loginsPage = next;
						void loadLogins();
					}}
				/>
			{/if}
		</div>

		<Dialog.Footer>
			<Button onclick={() => (historyFor = null)}>Close</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
