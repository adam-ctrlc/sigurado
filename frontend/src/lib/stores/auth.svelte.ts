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
