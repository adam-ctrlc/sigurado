<script lang="ts">
	import { onMount } from 'svelte';
	import InboxIcon from '@lucide/svelte/icons/inbox';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Empty from '$lib/components/ui/empty';
	import * as Select from '$lib/components/ui/select';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import TablePager from '$lib/components/table-pager.svelte';
	import { listMessages } from '$lib/api/sms';
	import { formatDateTime } from '$lib/format';
	import type { SmsMessage, SmsStatus } from '$lib/schemas/sms';

	let messages = $state<SmsMessage[]>([]);
	let loading = $state(true);
	let page = $state(1);
	let total = $state(0);
	let statusFilter = $state<'all' | SmsStatus>('all');
	const PER_PAGE = 10;

	const statusOptions = [
		{ value: 'all', label: 'Every message' },
		{ value: 'pending', label: 'Waiting to send' },
		{ value: 'sent', label: 'Sent' },
		{ value: 'failed', label: 'Failed' }
	] as const;

	const statusLabels: Record<string, string> = {
		all: 'Every message',
		pending: 'Waiting to send',
		sent: 'Sent',
		failed: 'Failed'
	};

	const statusVariant: Record<SmsStatus, 'default' | 'secondary' | 'outline'> = {
		pending: 'outline',
		sent: 'secondary',
		failed: 'outline'
	};

	async function refresh(): Promise<void> {
		loading = true;
		try {
			const result = await listMessages({
				page,
				per_page: PER_PAGE,
				status: statusFilter === 'all' ? undefined : statusFilter
			});
			messages = result.items;
			total = result.total;
			page = result.page;
		} catch {
			messages = [];
			total = 0;
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		void refresh();
		// The queue only drains when the node polls, so keep the view honest.
		const poll = setInterval(() => void refresh(), 15_000);
		return () => clearInterval(poll);
	});
</script>

<svelte:head><title>SMS Outbox - Sigurado</title></svelte:head>

<Card.Root>
	<Card.Header>
		<Card.Title>Outbox</Card.Title>
		<Card.Description>
			The server queues a message; the node sends it and reports back. Newest first.
		</Card.Description>
	</Card.Header>

	<Card.Content class="flex flex-col gap-4">
		<Select.Root
			type="single"
			bind:value={statusFilter}
			onValueChange={() => {
				page = 1;
				void refresh();
			}}
		>
			<Select.Trigger class="w-full sm:w-56">{statusLabels[statusFilter]}</Select.Trigger>
			<Select.Content>
				<Select.Group>
					{#each statusOptions as option (option.value)}
						<Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
					{/each}
				</Select.Group>
			</Select.Content>
		</Select.Root>

		{#if loading}
			<div class="flex flex-col gap-4 py-2">
				{#each Array.from({ length: 4 }) as _, i (i)}
					<div class="flex flex-col gap-2">
						<Skeleton class="h-4 w-72" />
						<Skeleton class="h-3 w-40" />
					</div>
				{/each}
			</div>
		{:else if messages.length === 0}
			<Empty.Root>
				<Empty.Header>
					<Empty.Media variant="icon"><InboxIcon /></Empty.Media>
					<Empty.Title>Nothing here yet</Empty.Title>
					<Empty.Description>
						Alerts appear the moment a reader grants or refuses something.
					</Empty.Description>
				</Empty.Header>
			</Empty.Root>
		{:else}
			<div class="overflow-hidden rounded-lg border">
				<Table.Root class="min-w-[56rem]">
					<Table.Header>
						<Table.Row class="hover:bg-transparent">
							<Table.Head>Message</Table.Head>
							<Table.Head>To</Table.Head>
							<Table.Head>Status</Table.Head>
							<Table.Head>Queued</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#each messages as message (message.id)}
							<Table.Row>
								<Table.Cell class="max-w-md">
									<p class="text-sm leading-relaxed">{message.body}</p>
									{#if message.error}
										<p class="text-destructive text-xs">{message.error}</p>
									{/if}
								</Table.Cell>
								<Table.Cell>
									<div class="flex flex-col">
										<span class="text-sm">{message.recipient_label ?? 'Removed'}</span>
										<span class="text-muted-foreground font-mono text-xs">
											{message.phone_number}
										</span>
									</div>
								</Table.Cell>
								<Table.Cell>
									<div class="flex flex-col gap-1">
										<Badge variant={statusVariant[message.status]} class="w-fit">
											{statusLabels[message.status]}
										</Badge>
										{#if message.attempts > 0 && message.status !== 'sent'}
											<span class="text-muted-foreground text-xs">
												{message.attempts}
												{message.attempts === 1 ? 'try' : 'tries'}
											</span>
										{/if}
									</div>
								</Table.Cell>
								<Table.Cell class="text-muted-foreground text-sm">
									{formatDateTime(message.created_at)}
									{#if message.sent_at}
										<span class="block text-xs">Sent {formatDateTime(message.sent_at)}</span>
									{/if}
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
				noun="messages"
				onPage={(next) => {
					page = next;
					void refresh();
				}}
			/>
		{/if}
	</Card.Content>
</Card.Root>
