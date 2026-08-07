import { browser } from '$app/environment';
import type { User } from '$lib/schemas/user';

const TOKEN_KEY = 'sigurado_token';

class AuthStore {
	token = $state<string | null>(browser ? localStorage.getItem(TOKEN_KEY) : null);
	user = $state<User | null>(null);

	get isAuthed(): boolean {
		return this.token !== null;
	}

	get isAdmin(): boolean {
		return this.user?.role === 'admin';
	}

	get isStaff(): boolean {
		return this.user?.role === 'admin' || this.user?.role === 'faculty';
	}

	/** A finger is bound to this account, so the readers know them. */
	get isEnrolled(): boolean {
		return this.user?.enrolled === true;
	}

	/**
	 * Everyone but an administrator has to enroll before the app is any use to
	 * them: without a finger bound, no door opens and nothing they do is recorded
	 * against them. Administrators are exempt because somebody has to be able to
	 * run the roster and pair the readers before anyone can enroll at all.
	 */
	get needsEnrollment(): boolean {
		return this.user !== null && this.user.role !== 'admin' && !this.isEnrolled;
	}

	setSession(token: string, user: User): void {
		this.token = token;
		this.user = user;
		if (browser) localStorage.setItem(TOKEN_KEY, token);
	}

	setUser(user: User): void {
		this.user = user;
	}

	clear(): void {
		this.token = null;
		this.user = null;
		if (browser) localStorage.removeItem(TOKEN_KEY);
	}
}

export const auth = new AuthStore();
