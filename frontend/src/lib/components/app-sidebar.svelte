<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import LayoutDashboardIcon from '@lucide/svelte/icons/layout-dashboard';
	import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
	import PackageCheckIcon from '@lucide/svelte/icons/package-check';
	import ScrollTextIcon from '@lucide/svelte/icons/scroll-text';
	import FlagIcon from '@lucide/svelte/icons/flag';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import UserRoundIcon from '@lucide/svelte/icons/user-round';
	import UsersIcon from '@lucide/svelte/icons/users';
	import HardDriveIcon from '@lucide/svelte/icons/hard-drive';
	import MessageSquareIcon from '@lucide/svelte/icons/message-square';
	import TableIcon from '@lucide/svelte/icons/table';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import ShieldCheckIcon from '@lucide/svelte/icons/shield-check';
	import LogOutIcon from '@lucide/svelte/icons/log-out';
import XIcon from '@lucide/svelte/icons/x';
	import * as Sidebar from '$lib/components/ui/sidebar';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import * as Avatar from '$lib/components/ui/avatar';
	import { Separator } from '$lib/components/ui/separator';
	import { auth } from '$lib/stores/auth.svelte';
	import { allowedWhileUnenrolled } from '$lib/enrollment';
	import type { Role } from '$lib/schemas/user';
	import type { Component } from 'svelte';

	interface NavItem {
		href: string;
		label: string;
		icon: Component;
		roles: Role[];
	}

	const monitoring: NavItem[] = [
		{
			href: '/dashboard',
			label: 'Dashboard',
			icon: LayoutDashboardIcon,
			roles: ['student', 'faculty', 'admin']
		},
		{ href: '/logs', label: 'Audit Log', icon: ScrollTextIcon, roles: ['faculty', 'admin'] },
		{ href: '/flags', label: 'Flags', icon: FlagIcon, roles: ['faculty', 'admin'] }
	];

	const access: NavItem[] = [
		{ href: '/enrollment', label: 'Enrollment', icon: FingerprintIcon, roles: ['student', 'faculty', 'admin'] },
		{ href: '/checkout', label: 'Checkout', icon: PackageCheckIcon, roles: ['student', 'faculty', 'admin'] }
	];

	const account: NavItem[] = [
		{
			href: '/profile',
			label: 'Profile',
			icon: UserRoundIcon,
			roles: ['student', 'faculty', 'admin']
		}
	];

	const help: NavItem[] = [
		{ href: '/guides', label: 'Guides', icon: BookOpenIcon, roles: ['student', 'faculty', 'admin'] }
	];

	const administration: NavItem[] = [
		{ href: '/admin/users', label: 'Users', icon: UsersIcon, roles: ['admin'] },
		{ href: '/admin/devices', label: 'Devices', icon: HardDriveIcon, roles: ['admin'] },
		{ href: '/admin/alerts', label: 'SMS Alerts', icon: MessageSquareIcon, roles: ['admin'] },
		{ href: '/admin/sheets', label: 'Sheet', icon: TableIcon, roles: ['admin'] },
		{ href: '/admin/data', label: 'Data', icon: DatabaseIcon, roles: ['admin'] }
	];

	const role = $derived(auth.user?.role ?? 'student');
	const groups = $derived([
		{ label: 'Monitoring', items: monitoring.filter((i) => i.roles.includes(role)) },
		{ label: 'Access', items: access.filter((i) => i.roles.includes(role)) },
		{ label: 'Administration', items: administration.filter((i) => i.roles.includes(role)) },
		{ label: 'Account', items: account.filter((i) => i.roles.includes(role)) },
		{ label: 'Help', items: help.filter((i) => i.roles.includes(role)) }
	].filter((g) => g.items.length > 0));

	const initials = $derived(
		(auth.user?.display_name ?? '?')
			.split(' ')
			.map((part) => part[0] ?? '')
			.join('')
			.slice(0, 2)
			.toUpperCase()
	);

	function isActive(href: string): boolean {
		return page.url.pathname === href || page.url.pathname.startsWith(`${href}/`);
	}

	// Out of reach until a finger is bound. Shown but plainly unavailable, rather
	// than hidden, so the sidebar does not appear to change shape on enrollment.
	function isHeld(href: string): boolean {
		return auth.needsEnrollment && !allowedWhileUnenrolled(href);
	}

	let logoutOpen = $state(false);

	function logout(): void {
		auth.clear();
		void goto('/login');
	}
</script>

<Sidebar.Root collapsible="icon">
	<Sidebar.Header>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton size="lg" class={isHeld('/dashboard') ? 'pointer-events-none' : ''}>
					{#snippet child({ props })}
						<a href={isHeld('/dashboard') ? undefined : '/dashboard'} {...props}>
							<div
								class="bg-primary text-primary-foreground flex aspect-square size-8 items-center justify-center rounded-full"
							>
								<ShieldCheckIcon class="size-4" />
							</div>
							<div class="flex flex-col gap-0.5 leading-none">
								<span class="font-semibold">Sigurado</span>
								<span class="text-muted-foreground text-xs">Lab materials</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Header>

	<Sidebar.Content>
		{#each groups as group (group.label)}
			<Sidebar.Group>
				<Sidebar.GroupLabel>{group.label}</Sidebar.GroupLabel>
				<Sidebar.GroupContent>
					<Sidebar.Menu>
						{#each group.items as item (item.href)}
							{@const Icon = item.icon}
							{@const held = isHeld(item.href)}
							<Sidebar.MenuItem>
								<Sidebar.MenuButton
									isActive={isActive(item.href)}
									tooltipContent={held ? `${item.label}: enroll your fingerprint first` : item.label}
									class={held ? 'pointer-events-none opacity-45' : ''}
								>
									{#snippet child({ props })}
										<!-- No href while held, so it cannot be clicked, focused or
										     opened in a new tab. -->
										<a
											href={held ? undefined : item.href}
											aria-disabled={held ? 'true' : undefined}
											{...props}
										>
											<Icon />
											<span>{item.label}</span>
										</a>
									{/snippet}
								</Sidebar.MenuButton>
							</Sidebar.MenuItem>
						{/each}
					</Sidebar.Menu>
				</Sidebar.GroupContent>
			</Sidebar.Group>
		{/each}
	</Sidebar.Content>

	<Sidebar.Footer>
		<Sidebar.Menu>
			<Sidebar.MenuItem>
				<Sidebar.MenuButton
					size="lg"
					isActive={isActive('/profile')}
					tooltipContent={isHeld('/profile') ? 'Your profile: enroll your fingerprint first' : 'Your profile'}
					class={isHeld('/profile') ? 'pointer-events-none opacity-45' : ''}
				>
					{#snippet child({ props })}
						<a
							href={isHeld('/profile') ? undefined : '/profile'}
							aria-disabled={isHeld('/profile') ? 'true' : undefined}
							{...props}
						>
							<Avatar.Root class="size-8">
								<Avatar.Fallback>{initials}</Avatar.Fallback>
							</Avatar.Root>
							<div class="flex min-w-0 flex-col gap-0.5 leading-none">
								<span class="truncate font-medium">{auth.user?.display_name ?? ''}</span>
								<span class="text-muted-foreground truncate text-xs">
									@{auth.user?.username ?? ''}
								</span>
							</div>
						</a>
					{/snippet}
				</Sidebar.MenuButton>

				<Sidebar.MenuAction onclick={() => (logoutOpen = true)} title="Sign out">
					<LogOutIcon />
					<span class="sr-only">Sign out</span>
				</Sidebar.MenuAction>
			</Sidebar.MenuItem>
		</Sidebar.Menu>
	</Sidebar.Footer>

	<Sidebar.Rail />
</Sidebar.Root>

<AlertDialog.Root bind:open={logoutOpen}>
	<!-- AlertDialog ignores outside clicks and Escape by default; this is a
	     low-stakes confirm, so allow both. -->
	<AlertDialog.Content interactOutsideBehavior="close" escapeKeydownBehavior="close">
		<AlertDialog.Header class="relative w-full">
			<AlertDialog.Title>Sign out of Sigurado?</AlertDialog.Title>
			<AlertDialog.Cancel
				class="hover:bg-accent absolute top-1/2 right-0 flex size-7 -translate-y-1/2 items-center justify-center border-0 bg-transparent p-0 shadow-none"
			>
				<XIcon />
				<span class="sr-only">Close</span>
			</AlertDialog.Cancel>
		</AlertDialog.Header>

		<Separator class="-mx-4 w-auto" />

		<div>
			<AlertDialog.Description>
				You will need your username and password to sign back in. Any door session you
				already opened stays active until it expires.
			</AlertDialog.Description>
		</div>

		<AlertDialog.Footer>
			<AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
			<AlertDialog.Action onclick={logout}>Sign out</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
