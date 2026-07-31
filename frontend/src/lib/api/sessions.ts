import { api } from '$lib/api/client';
import { sessionListSchema, type AccessSession } from '$lib/schemas/session';

export function activeSessions(): Promise<AccessSession[]> {
	return api.get('/sessions/active', sessionListSchema);
}

export function sessionHistory(): Promise<AccessSession[]> {
	return api.get('/sessions', sessionListSchema);
}
