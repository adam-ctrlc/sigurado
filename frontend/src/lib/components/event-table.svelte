<script lang="ts">
	import ActivityIcon from '@lucide/svelte/icons/activity';
	import * as Table from '$lib/components/ui/table';
	import * as Empty from '$lib/components/ui/empty';
	import { Badge } from '$lib/components/ui/badge';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import { DECISION_CLASS, EVENT_LABELS, eventPerson, fingerLabel } from '$lib/events';
	import type { AccessEvent } from '$lib/schemas/event';

	interface Props {
		events: AccessEvent[];
		loading?: boolean;
		rows?: number;
	}

	let { events, loading = false, rows = 8 }: Props = $props();

	import { formatClock } from '$lib/format';
</script>

<div class="rounded-lg border">
	<Table.Root class="min-w-[48rem]">
		<Table.Header>
			<Table.Row>
				<Table.Head class="w-24">Time</Table.Head>
				<Table.Head>Event</Table.Head>
				<Table.Head>User</Table.Head>
				<Table.Head>Device</Table.Head>
				<Table.Head>Detail</Table.Head>
			</Table.Row>
		</Table.Header>
		<Table.Body>
			{#if loading}
				{#each Array.from({ length: rows }) as _, i (i)}
					<Table.Row>
						{#each Array.from({ length: 5 }) as _, c (c)}
							<Table.Cell><Skeleton class="h-4 w-full" /></Table.Cell>
						{/each}
					</Table.Row>
				{/each}
			{:else if events.length === 0}
				<Table.Row class="hover:bg-transparent">
					<Table.Cell colspan={5} class="p-0">
						<Empty.Root class="border-0">
							<Empty.Header>
								<Empty.Media variant="icon">
									<ActivityIcon />
								</Empty.Media>
								<Empty.Title>No activity yet</Empty.Title>
								<Empty.Description>
									Scans, denials and enrollments appear here the moment they happen.
								</Empty.Description>
							</Empty.Header>
						</Empty.Root>
					</Table.Cell>
				</Table.Row>
			{:else}
				{#each events as event (event.id)}
					<Table.Row>
						<Table.Cell class="text-muted-foreground font-mono text-xs">{formatClock(event.created_at)}</Table.Cell>
						<Table.Cell>
							<Badge class={DECISION_CLASS[event.decision]}>{EVENT_LABELS[event.event_type]}</Badge>
						</Table.Cell>
						<Table.Cell>
							{@const person = eventPerson(event)}
							{#if person.name}
								<span class="text-sm">{person.name}</span>
							{:else}
								<span class="text-muted-foreground font-mono text-xs">{fingerLabel(event)}</span>
							{/if}
						</Table.Cell>
						<Table.Cell>{event.device_name ?? '-'}</Table.Cell>
						<Table.Cell class="text-muted-foreground max-w-xs truncate">
							{event.message ?? ''}
						</Table.Cell>
					</Table.Row>
				{/each}
			{/if}
		</Table.Body>
	</Table.Root>
</div>
