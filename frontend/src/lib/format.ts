const DATE: Intl.DateTimeFormatOptions = {
	month: 'long',
	day: 'numeric',
	year: 'numeric'
};

const TIME: Intl.DateTimeFormatOptions = {
	hour: 'numeric',
	minute: '2-digit',
	hour12: true
};

/** "July 19, 2026" */
export function formatDate(iso: string | null | undefined): string {
	if (!iso) return '';
	return new Date(iso).toLocaleDateString('en-US', DATE);
}

/** "July 19, 2026 at 2:14 PM" */
export function formatDateTime(iso: string | null | undefined): string {
	if (!iso) return '';
	const d = new Date(iso);
	return `${d.toLocaleDateString('en-US', DATE)} at ${d.toLocaleTimeString('en-US', TIME)}`;
}

/** "2:14:03 PM" — for the live feed, where the date is implied. */
export function formatClock(iso: string | null | undefined): string {
	if (!iso) return '';
	return new Date(iso).toLocaleTimeString('en-US', { ...TIME, second: '2-digit' });
}

/** "July 19, 2026 at 2:14:03.482 PM" — for a single record under inspection. */
export function formatPrecise(iso: string | null | undefined): string {
	if (!iso) return '';
	const d = new Date(iso);
	const ms = String(d.getMilliseconds()).padStart(3, '0');
	const clock = d.toLocaleTimeString('en-US', { ...TIME, second: '2-digit' });
	// Slot the milliseconds in front of the AM/PM marker.
	const withMs = clock.replace(/(\d{2})(?=\s*[AP]M)/i, `$1.${ms}`);
	return `${d.toLocaleDateString('en-US', DATE)} at ${withMs}`;
}

const UNITS: { limit: number; step: number; name: Intl.RelativeTimeFormatUnit }[] = [
	{ limit: 60, step: 1, name: 'second' },
	{ limit: 3600, step: 60, name: 'minute' },
	{ limit: 86400, step: 3600, name: 'hour' },
	{ limit: 604800, step: 86400, name: 'day' },
	{ limit: 2629800, step: 604800, name: 'week' },
	{ limit: Infinity, step: 2629800, name: 'month' }
];

const RELATIVE = new Intl.RelativeTimeFormat('en-US', { numeric: 'auto' });

/** "5 minutes ago", against `nowMs` so a caller can drive it from a ticker. */
export function formatRelative(iso: string | null | undefined, nowMs = Date.now()): string {
	if (!iso) return '';
	const seconds = (new Date(iso).getTime() - nowMs) / 1000;
	const magnitude = Math.abs(seconds);
	if (magnitude < 10) return 'just now';
	const unit = UNITS.find((u) => magnitude < u.limit) ?? UNITS[UNITS.length - 1];
	if (!unit) return '';
	return RELATIVE.format(Math.round(seconds / unit.step), unit.name);
}

/** "3 hours 12 minutes", "45 seconds" — for a span, not a point in time. */
export function formatDuration(totalSeconds: number): string {
	const seconds = Math.max(0, Math.round(totalSeconds));
	if (seconds < 60) return `${seconds} ${seconds === 1 ? 'second' : 'seconds'}`;

	const days = Math.floor(seconds / 86400);
	const hours = Math.floor((seconds % 86400) / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);

	const parts: string[] = [];
	if (days > 0) parts.push(`${days} ${days === 1 ? 'day' : 'days'}`);
	if (hours > 0) parts.push(`${hours} ${hours === 1 ? 'hour' : 'hours'}`);
	// Minutes are noise next to days, but they matter next to hours.
	if (minutes > 0 && days === 0) parts.push(`${minutes} ${minutes === 1 ? 'minute' : 'minutes'}`);
	return parts.length > 0 ? parts.join(' ') : '1 minute';
}
