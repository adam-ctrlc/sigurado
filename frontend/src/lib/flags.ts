import type { Component } from 'svelte';
import PackageOpenIcon from '@lucide/svelte/icons/package-open';
import DoorClosedIcon from '@lucide/svelte/icons/door-closed';
import KeyRoundIcon from '@lucide/svelte/icons/key-round';
import FingerprintIcon from '@lucide/svelte/icons/fingerprint';
import MoonIcon from '@lucide/svelte/icons/moon';
import RepeatIcon from '@lucide/svelte/icons/repeat';
import UserXIcon from '@lucide/svelte/icons/user-x';
import LockIcon from '@lucide/svelte/icons/lock';
import ClockAlertIcon from '@lucide/svelte/icons/clock-alert';
import WifiOffIcon from '@lucide/svelte/icons/wifi-off';
import MailQuestionIcon from '@lucide/svelte/icons/mail-question';
import FlagIcon from '@lucide/svelte/icons/flag';
import type { Severity } from '$lib/api/flags';

export interface FlagKindInfo {
	/** Heading for the group of occurrences. */
	label: string;
	icon: Component;
	/** What the check looks for, in one line. */
	watches: string;
}

/**
 * Plain English for each check the server runs. The server sends the detail and
 * the advice; this only names and illustrates the group.
 */
export const FLAG_KINDS: Record<string, FlagKindInfo> = {
	unrecorded_opening: {
		label: 'Opened the cabinet, recorded nothing',
		icon: PackageOpenIcon,
		watches: 'A cabinet code issued at an opening that expired without a checkout behind it.'
	},
	tailgating: {
		label: 'Cabinet tried without a door scan',
		icon: DoorClosedIcon,
		watches: 'A registered finger at the cabinet with no door scan in front of it.'
	},
	code_guessing: {
		label: 'Checkout codes guessed',
		icon: KeyRoundIcon,
		watches: 'Repeated checkout codes the cabinet never issued to that person.'
	},
	finger_probing: {
		label: 'Unregistered fingers tried repeatedly',
		icon: FingerprintIcon,
		watches: 'The same reader refusing several unknown fingers in a short span.'
	},
	after_hours: {
		label: 'Cabinet opened outside lab hours',
		icon: MoonIcon,
		watches: 'An opening between 8 PM and 6 AM, when nobody is around to notice.'
	},
	repeat_openings: {
		label: 'Opened repeatedly in one visit',
		icon: RepeatIcon,
		watches: 'Several cabinet openings inside a single door window.'
	},
	disabled_account: {
		label: 'Disabled account still trying',
		icon: UserXIcon,
		watches: 'A finger belonging to a disabled account presented at a reader.'
	},
	sign_in_attempts: {
		label: 'Failed sign-ins on the website',
		icon: LockIcon,
		watches: 'Repeated wrong passwords for one account name.'
	},
	session_for_disabled_user: {
		label: 'Live window on a disabled account',
		icon: ClockAlertIcon,
		watches: 'An open door window belonging to somebody who has since been disabled.'
	},
	silent_reader: {
		label: 'Reader has gone quiet',
		icon: WifiOffIcon,
		watches: 'A device that stopped reporting in, which could be a fault or unplugging.'
	},
	unclaimed_enrollments: {
		label: 'Enrollment codes never claimed',
		icon: MailQuestionIcon,
		watches: 'Fingers enrolled at a reader that nobody ever bound to an account.'
	}
};

export function flagKind(kind: string): FlagKindInfo {
	return (
		FLAG_KINDS[kind] ?? {
			label: kind.replaceAll('_', ' '),
			icon: FlagIcon,
			watches: 'Something the audit trail could not explain.'
		}
	);
}

export const SEVERITY_LABELS: Record<Severity, string> = {
	high: 'Needs attention',
	medium: 'Worth a look',
	low: 'For information'
};

/** Badge styling per severity, using the theme tokens rather than raw colors. */
export const SEVERITY_CLASS: Record<Severity, string> = {
	high: 'bg-destructive/15 text-destructive',
	medium: 'bg-amber-500/15 text-amber-600 dark:text-amber-400',
	low: 'bg-primary/15 text-primary'
};

export const SEVERITY_RING: Record<Severity, string> = {
	high: 'border-destructive/40',
	medium: 'border-amber-500/40',
	low: 'border-border'
};
