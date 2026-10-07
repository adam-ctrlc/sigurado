import { browser } from '$app/environment';

export const THEMES = ['gold', 'light', 'paper', 'dark', 'tech', 'contrast'] as const;

/** What the app looks like unless somebody has picked otherwise. */
export const DEFAULT_THEME: Theme = 'gold';
export type Theme = (typeof THEMES)[number];

const STORAGE_KEY = 'sigurado.theme';

function isTheme(value: string | null): value is Theme {
	return value !== null && (THEMES as readonly string[]).includes(value);
}

/** The dark-family themes all carry `.dark` on top of their own class. */
const DARK: readonly Theme[] = ['gold', 'dark', 'tech', 'contrast'];

export function isDark(theme: Theme): boolean {
	return DARK.includes(theme);
}

/**
 * Applies a theme by class on <html>. A dark-family theme also carries `dark`,
 * so every `dark:` variant in the components keeps working and the theme's own
 * class only has to override the color tokens.
 */
function apply(theme: Theme): void {
	if (!browser) return;
	const root = document.documentElement;
	const dark = isDark(theme);
	root.classList.toggle('dark', dark);
	root.classList.toggle('gold', theme === 'gold');
	root.classList.toggle('paper', theme === 'paper');
	root.classList.toggle('tech', theme === 'tech');
	root.classList.toggle('contrast', theme === 'contrast');
	root.style.colorScheme = dark ? 'dark' : 'light';
}

function initial(): Theme {
	if (!browser) return DEFAULT_THEME;
	const stored = localStorage.getItem(STORAGE_KEY);
	return isTheme(stored) ? stored : DEFAULT_THEME;
}

class ThemeStore {
	current = $state<Theme>(initial());

	set(theme: Theme): void {
		this.current = theme;
		apply(theme);
		if (browser) localStorage.setItem(STORAGE_KEY, theme);
	}

	/** Keeps the DOM in step with a value restored before hydration. */
	sync(): void {
		apply(this.current);
	}
}

export const theme = new ThemeStore();
