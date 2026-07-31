import type { AccessEvent, AccessEventType, EventDecision } from '$lib/schemas/event';

/** Plain English for each event the readers can report. */
export const EVENT_LABELS: Record<AccessEventType, string> = {
	door_granted: 'Door opened',
	door_denied_unregistered: 'Door denied: finger not registered',
	door_denied_inactive: 'Door denied: account disabled',
	box_granted: 'Cabinet unlocked',
	box_denied_no_session: 'Cabinet denied: no door scan first',
	box_denied_unregistered: 'Cabinet denied: finger not registered',
	box_denied_session_expired: 'Cabinet denied: window expired',
	enroll_code_issued: 'Enrollment code issued',
	enroll_bound: 'Fingerprint bound',
	enroll_failed: 'Enrollment failed',
	checkout_code_rejected: 'Checkout code refused'
};

export type EventStage = 'Door' | 'Cabinet' | 'Enrollment' | 'Checkout';

export function eventStage(type: AccessEventType): EventStage {
	if (type.startsWith('door')) return 'Door';
	if (type.startsWith('box')) return 'Cabinet';
	if (type.startsWith('checkout')) return 'Checkout';
	return 'Enrollment';
}

export const DECISION_CLASS: Record<EventDecision, string> = {
	granted: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400',
	denied: 'bg-destructive/15 text-destructive',
	info: 'bg-primary/15 text-primary'
};

/**
 * Who the event is about. `user_name` is who the reader identified at the time.
 * `finger_owner_name` is who that finger turned out to belong to, resolved from
 * the token when the event itself predates the enrollment.
 */
export function eventPerson(event: AccessEvent): {
	name: string | null;
	/** True when the name came from the finger rather than the reader. */
	retroactive: boolean;
} {
	if (event.user_name) return { name: event.user_name, retroactive: false };
	if (event.finger_owner_name) return { name: event.finger_owner_name, retroactive: true };
	return { name: null, retroactive: false };
}

/** The finger as printed on the reader, e.g. "door:7". */
export function fingerLabel(event: AccessEvent): string {
	return event.finger_token ?? 'an unrecorded finger';
}

/**
 * One sentence naming the finger, the reader, and the person, so a row reads on
 * its own without cross-referencing the ID columns.
 */
export function describeEvent(event: AccessEvent): string {
	const finger = fingerLabel(event);
	const reader = event.device_name ?? 'an unnamed reader';
	const { name, retroactive } = eventPerson(event);
	const person = name
		? retroactive
			? `${name} (identified later, once ${finger} was enrolled)`
			: name
		: 'nobody on record';

	switch (event.event_type) {
		case 'door_granted':
			return `${finger} matched ${person} at ${reader}, so the door opened and a cabinet window started.`;
		case 'door_denied_unregistered':
			return `${finger} is not registered to anyone, so ${reader} kept the door locked.`;
		case 'door_denied_inactive':
			return `${finger} belongs to ${person}, whose account is disabled, so ${reader} kept the door locked.`;
		case 'box_granted':
			return `${finger} matched ${person}, who had just scanned at the door, so ${reader} released the lock.`;
		case 'box_denied_no_session':
			return `${finger} was presented at ${reader} with no door scan behind it. This is the tailgating case the sequence exists to stop.`;
		case 'box_denied_unregistered':
			return `${finger} is not registered to anyone, so ${reader} kept the cabinet locked.`;
		case 'box_denied_session_expired':
			return `${finger} matched ${person}, but the door window had already closed, so ${reader} kept the cabinet locked.`;
		case 'enroll_code_issued':
			return `${reader} issued a one-time code for ${finger}. The finger stays unclaimed until someone enters that code on the website, and this event names them once they do.`;
		case 'enroll_bound':
			return `${finger} is now bound to ${person} and identifies them at ${reader} from here on.`;
		case 'enroll_failed':
			return `An enrollment attempt for ${finger} at ${reader} did not complete.`;
		case 'checkout_code_rejected':
			return `${person} tried to record a checkout with a code the cabinet never issued to them. Repeated attempts are someone guessing.`;
	}
}

/** Free-text search across everything a reader recorded about an event. */
export function matchesQuery(event: AccessEvent, query: string): boolean {
	const q = query.trim().toLowerCase();
	if (q.length === 0) return true;
	return [
		EVENT_LABELS[event.event_type],
		event.event_type,
		event.decision,
		event.user_name,
		event.finger_owner_name,
		event.device_name,
		event.finger_token,
		event.message
	].some((field) => (field ?? '').toLowerCase().includes(q));
}
