import { z } from 'zod';
import { api } from '$lib/api/client';
import { pageSchema, queryString, type Paged, type PageRequest } from '$lib/api/page';
import { loginResponseSchema, type LoginResponse } from '$lib/schemas/auth';
import {
	userSchema,
	loginEventSchema,
	type User,
	type LoginEvent
} from '$lib/schemas/user';

export function login(identifier: string, password: string): Promise<LoginResponse> {
	return api.post('/auth/login', loginResponseSchema, { identifier, password });
}

export function me(): Promise<User> {
	return api.get('/auth/me', userSchema);
}

const loginPageSchema = pageSchema(loginEventSchema);

export function myLogins(request: PageRequest = {}): Promise<Paged<LoginEvent>> {
	return api.get(`/auth/my-logins${queryString({ ...request })}`, loginPageSchema);
}

export interface UpdateMeInput {
	email?: string;
	username?: string;
	/** An empty string clears it. */
	phone_number?: string;
	first_name?: string;
	/** An empty string clears it. */
	middle_name?: string;
	last_name?: string;
	suffix?: string;
}

export function updateMe(input: UpdateMeInput): Promise<User> {
	return api.patch('/auth/me', userSchema, input);
}

export function changePassword(currentPassword: string, newPassword: string): Promise<void> {
	return api.post('/auth/password', z.void(), {
		current_password: currentPassword,
		new_password: newPassword
	});
}
