/**
 * Turns a raw User-Agent header into something readable in the audit UI.
 * Deliberately coarse: enough to recognize "that was not my laptop", not
 * a full parser.
 */

const BROWSERS: { label: string; test: RegExp }[] = [
	{ label: 'Edge', test: /Edg[A-Z]?\// },
	{ label: 'Opera', test: /OPR\/|Opera/ },
	{ label: 'Samsung Internet', test: /SamsungBrowser/ },
	{ label: 'Firefox', test: /Firefox\// },
	{ label: 'Chrome', test: /Chrome\/|CriOS/ },
	{ label: 'Safari', test: /Safari\// }
];

const PLATFORMS: { label: string; test: RegExp }[] = [
	{ label: 'Windows', test: /Windows NT/ },
	{ label: 'Android', test: /Android/ },
	{ label: 'iPhone', test: /iPhone/ },
	{ label: 'iPad', test: /iPad/ },
	{ label: 'macOS', test: /Mac OS X|Macintosh/ },
	{ label: 'Linux', test: /Linux/ }
];

export function describeUserAgent(value: string | null | undefined): string {
	if (!value) return 'Unknown device';
	const browser = BROWSERS.find((b) => b.test.test(value))?.label;
	const platform = PLATFORMS.find((p) => p.test.test(value))?.label;
	if (browser && platform) return `${browser} on ${platform}`;
	if (browser) return browser;
	if (platform) return platform;
	return value.slice(0, 40);
}
