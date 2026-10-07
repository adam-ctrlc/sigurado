import type { Component } from 'svelte';
import DoorOpenIcon from '@lucide/svelte/icons/door-open';
import PackageIcon from '@lucide/svelte/icons/package';
import ScrollTextIcon from '@lucide/svelte/icons/scroll-text';
import LockKeyholeIcon from '@lucide/svelte/icons/lock-keyhole';
import UserXIcon from '@lucide/svelte/icons/user-x';
import ClockIcon from '@lucide/svelte/icons/clock';
import CameraIcon from '@lucide/svelte/icons/camera';
import CpuIcon from '@lucide/svelte/icons/cpu';
import MicrochipIcon from '@lucide/svelte/icons/microchip';
import PlugZapIcon from '@lucide/svelte/icons/plug-zap';
import WifiIcon from '@lucide/svelte/icons/wifi';
import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
import MonitorIcon from '@lucide/svelte/icons/monitor';
import MessageSquareIcon from '@lucide/svelte/icons/message-square';
import type { EventDecision } from '$lib/schemas/event';

export interface Item {
	icon: Component;
	title: string;
	text: string;
}

export const sequence: { step: string; icon: Component; title: string; text: string }[] = [
	{
		step: 'One',
		icon: DoorOpenIcon,
		title: 'Scan at the door',
		text: 'A registered finger unlocks the door and opens a short window at the cabinet, held in their name.'
	},
	{
		step: 'Two',
		icon: PackageIcon,
		title: 'Scan at the cabinet',
		text: 'The cabinet releases only for the same person, and only while their window is still open.'
	},
	{
		step: 'Three',
		icon: ScrollTextIcon,
		title: 'Recorded either way',
		text: 'Granted or denied, the decision is written down with the identity, the reader, and the time.'
	}
];

export const proof: { icon: Component; text: string }[] = [
	{ icon: UserXIcon, text: 'Tailgating refused' },
	{ icon: ClockIcon, text: 'Timed access window' },
	{ icon: ScrollTextIcon, text: 'Named audit trail' }
];

/**
 * Every figure here is a fact about the build, not a claim about adoption.
 * 120s is the shipped SESSION_TTL_SECS, 11 is the number of checks in
 * flag_service, and reader events are genuinely append-only, so an
 * administrator editing one is not a thing the API offers.
 */
export const stats: { value: string; label: string; note: string }[] = [
	{ value: '2', label: 'Readers in sequence', note: 'Door, then cabinet' },
	{ value: '120s', label: 'Access window', note: 'Default, then it closes' },
	{ value: '11', label: 'Checks over the trail', note: 'Run on the flags page' },
	{ value: '0', label: 'Editable reader records', note: 'Admins included' }
];

/** The real bill of materials, standing in for the usual row of client logos. */
export const parts: { icon: Component; label: string }[] = [
	{ icon: MicrochipIcon, label: 'ESP32 DevKit v1' },
	{ icon: FingerprintIcon, label: 'R307 / AS608' },
	{ icon: MonitorIcon, label: '16x2 I2C LCD' },
	{ icon: PlugZapIcon, label: '12V solenoid' },
	{ icon: MessageSquareIcon, label: 'SIM800L' }
];

// The same vocabulary and badge colors the real audit log uses.
export const preview: {
	time: string;
	label: string;
	decision: EventDecision;
	person: string;
	detail: string;
}[] = [
	{
		time: '9:41:07 AM',
		label: 'Cabinet unlocked',
		decision: 'granted',
		person: 'Juan Ramos Dela Cruz',
		detail: 'cabinet:3'
	},
	{
		time: '9:41:02 AM',
		label: 'Door opened',
		decision: 'granted',
		person: 'Juan Ramos Dela Cruz',
		detail: 'door:7'
	},
	{
		time: '9:38:55 AM',
		label: 'Cabinet denied: no door scan first',
		decision: 'denied',
		person: 'Ana Villareal',
		detail: 'tailgating blocked'
	},
	{
		time: '9:22:13 AM',
		label: 'Enrollment code issued',
		decision: 'info',
		person: 'Unknown finger',
		detail: 'door:12'
	}
];

export const guarantees: Item[] = [
	{
		icon: LockKeyholeIcon,
		title: 'A key cannot be lent',
		text: 'There is nothing to borrow, copy, or sign out. The finger at the reader is the identity in the log.'
	},
	{
		icon: UserXIcon,
		title: 'Tailgating gets refused',
		text: 'Walking in behind someone leaves no door session, so the cabinet stays locked and the attempt is recorded.'
	},
	{
		icon: ClockIcon,
		title: 'The window closes on its own',
		text: 'A door scan is good for a set number of seconds. Wander off and the cabinet no longer knows you.'
	},
	{
		icon: CameraIcon,
		title: 'Materials, not just doors',
		text: 'What left the shelf is logged item by item, with an optional photo, against the person who took it.'
	}
];

export const hardware: Item[] = [
	{
		icon: MicrochipIcon,
		title: 'Two reader nodes',
		text: 'One at the door, one on the cabinet. Each holds its own fingerprint templates and authenticates to the server with its own secret.'
	},
	{
		icon: PlugZapIcon,
		title: 'Solenoid and limit switch',
		text: 'The lock is driven through a relay, and a limit switch reports whether the door actually closed behind you.'
	},
	{
		icon: WifiIcon,
		title: 'Heartbeats and NTP time',
		text: 'Nodes report in on a schedule, so the console shows which readers are live and every record carries a synced timestamp.'
	}
];

export const entries: Item[] = [
	{
		icon: FingerprintIcon,
		title: 'Enroll a finger',
		text: 'Claim a one-time code from the reader and bind the finger to your account.'
	},
	{
		icon: PackageIcon,
		title: 'Record a checkout',
		text: 'List what you took, quantity by quantity, with an optional photo.'
	},
	{
		icon: CpuIcon,
		title: 'Watch the readers',
		text: 'See which nodes are online, for how long, and every decision as it lands.'
	}
];

// Public surface only: everything behind sign-in stays out of the footer.
export const footerColumns: {
	title: string;
	links: { label: string; href?: string }[];
}[] = [
	{
		title: 'How it works',
		links: [
			{ label: 'The sequence', href: '#sequence' },
			{ label: 'What it prevents', href: '#prevents' },
			{ label: 'The record', href: '#record' },
			{ label: 'Hardware', href: '#hardware' }
		]
	},
	{
		title: 'At the readers',
		links: [
			{ label: 'Fingerprint at the door' },
			{ label: 'Fingerprint at the cabinet' },
			{ label: 'One-time enrollment codes' },
			{ label: 'Solenoid lock and limit switch' }
		]
	},
	{
		title: 'Access',
		links: [{ label: 'Sign in', href: '/login' }, { label: 'Accounts are made by an admin' }]
	}
];
