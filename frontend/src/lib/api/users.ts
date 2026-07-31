import { z } from 'zod';
import { api } from '$lib/api/client';
import { pageSchema, queryString, type Paged, type PageRequest } from '$lib/api/page';
import {
	userSchema,
	fingerprintListSchema,
	loginEventSchema,
	type User,
	type Fingerprint,
	type LoginEvent,
	type Role
} from '$lib/schemas/user';

export interface CreateUserInput {
	email: string;
	username: string;
	password: string;
	/** Optional; only staff ever receive SMS alerts. */
	phone_number?: string;
	first_name: string;
	middle_name?: string;
	last_name: string;
	suffix?: string;
	role: Role;
}

export interface UpdateUserInput {
	email?: string;
	username?: string;
	/** An empty string clears it. */
	phone_number?: string;
	first_name?: string;
	/** An empty string clears it. */
	middle_name?: string;
	last_name?: string;
	suffix?: string;
	role?: Role;
	is_active?: boolean;
	password?: string;
}

export interface UserFilter extends PageRequest {
	/** Free text over the name parts, username and email. */
	q?: string;
	role?: Role;
	active?: boolean;
}

const userPageSchema = pageSchema(userSchema);

export function listUsers(filter: UserFilter = {}): Promise<Paged<User>> {
	return api.get(
		`/users${queryString({ ...filter, active: filter.active === undefined ? undefined : String(filter.active) })}`,
		userPageSchema
	);
}

export function createUser(input: CreateUserInput): Promise<User> {
	return api.post('/users', userSchema, input);
}

export function updateUser(id: string, input: UpdateUserInput): Promise<User> {
	return api.patch(`/users/${id}`, userSchema, input);
}

export function deleteUser(id: string): Promise<void> {
	return api.del(`/users/${id}`, z.void());
}

const loginPageSchema = pageSchema(loginEventSchema);

export function listUserLogins(
	userId: string,
	request: PageRequest = {}
): Promise<Paged<LoginEvent>> {
	return api.get(`/users/${userId}/logins${queryString({ ...request })}`, loginPageSchema);
}

export function listFingerprints(userId: string): Promise<Fingerprint[]> {
	return api.get(`/users/${userId}/fingerprints`, fingerprintListSchema);
}

export function deleteFingerprint(id: string): Promise<void> {
	return api.del(`/fingerprints/${id}`, z.void());
}
