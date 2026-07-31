/**
 * The password rules, mirrored from the backend's `auth::policy` so the form can
 * show progress as someone types. The server is still the authority: this only
 * saves a round trip.
 */

export const MIN_LENGTH = 8;
export const MAX_LENGTH = 16;

export interface Rule {
	label: string;
	met: (password: string) => boolean;
}

export const RULES: Rule[] = [
	{
		label: `${MIN_LENGTH} to ${MAX_LENGTH} characters`,
		met: (p) => [...p].length >= MIN_LENGTH && [...p].length <= MAX_LENGTH
	},
	{ label: 'One uppercase letter', met: (p) => /\p{Lu}/u.test(p) },
	{ label: 'One lowercase letter', met: (p) => /\p{Ll}/u.test(p) },
	{ label: 'One symbol', met: (p) => [...p].some((c) => !/\p{L}|\p{N}/u.test(c)) },
	{ label: 'No spaces', met: (p) => p.length > 0 && !/\s/.test(p) }
];

export function passwordOk(password: string): boolean {
	return RULES.every((rule) => rule.met(password));
}
