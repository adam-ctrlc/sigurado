import { fetchEventSource } from '@microsoft/fetch-event-source';
import { api, API_BASE } from '$lib/api/client';
import { auth } from '$lib/stores/auth.svelte';
import { pageSchema, queryString, type Paged, type PageRequest } from '$lib/api/page';
import {
	eventSchema,
	type AccessEvent,
	type AccessEventType,
	type EventDecision
} from '$lib/schemas/event';

export interface LogFilter extends PageRequest {
	event_type?: AccessEventType;
	decision?: EventDecision;
	device_id?: string;
	user_id?: string;
	/** Free text; matched in the database, not here. */
	q?: string;
}

const logPageSchema = pageSchema(eventSchema);

export function listLogs(filter: LogFilter = {}): Promise<Paged<AccessEvent>> {
	return api.get(`/logs${queryString({ ...filter })}`, logPageSchema);
}

export interface LogStream {
	close(): void;
}

/**
 * Subscribe to the live audit feed over SSE. Uses fetch-event-source so the
 * bearer token can be sent (native EventSource cannot set headers).
 */
export function streamLogs(
	onEvent: (event: AccessEvent) => void,
	onError?: (err: unknown) => void
): LogStream {
	const controller = new AbortController();
	const headers: Record<string, string> = {};
	if (auth.token !== null) headers['Authorization'] = `Bearer ${auth.token}`;

	void fetchEventSource(`${API_BASE}/logs/stream`, {
		headers,
		signal: controller.signal,
		openWhenHidden: true,
		onmessage(msg) {
			if (msg.data.length === 0) return;
			try {
				onEvent(eventSchema.parse(JSON.parse(msg.data)));
			} catch (err) {
				onError?.(err);
			}
		},
		onerror(err) {
			onError?.(err);
		}
	});

	return {
		close: () => controller.abort()
	};
}
