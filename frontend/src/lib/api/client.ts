import type { z } from 'zod';
import { browser } from '$app/environment';
import { goto } from '$app/navigation';
import { env } from '$env/dynamic/public';
import { auth } from '$lib/stores/auth.svelte';

export const API_BASE = env['PUBLIC_API_BASE'] ?? 'http://localhost:8080/api';

export class ApiError extends Error {
	constructor(
		public status: number,
		public code: string,
		message: string
	) {
		super(message);
		this.name = 'ApiError';
	}
}

export interface DeviceAuth {
	id: string;
	secret: string;
}

interface RequestOptions {
	body?: unknown;
	form?: FormData;
	auth?: boolean;
	device?: DeviceAuth;
}

async function request<T>(
	method: string,
	path: string,
	schema: z.ZodType<T>,
	options: RequestOptions = {}
): Promise<T> {
	const { body, form, auth: useAuth = true, device } = options;
	const headers: Record<string, string> = {};

	if (device) {
		headers['X-Device-Id'] = device.id;
		headers['X-Device-Secret'] = device.secret;
	} else if (useAuth && auth.token !== null) {
		headers['Authorization'] = `Bearer ${auth.token}`;
	}

	let payload: BodyInit | undefined;
	if (form) {
		payload = form;
	} else if (body !== undefined) {
		headers['Content-Type'] = 'application/json';
		payload = JSON.stringify(body);
	}

	const res = await fetch(`${API_BASE}${path}`, { method, headers, body: payload });

	// A 401 from the sign-in call means the credentials were wrong, not that a
	// session lapsed: there is no session yet. Only the second case should clear
	// the stored token and bounce to the login page.
	if (res.status === 401 && !device) {
		if (path.startsWith('/auth/login')) {
			throw new ApiError(
				401,
				'invalid_credentials',
				'That username or password is not right. Please try again.'
			);
		}
		auth.clear();
		if (browser) void goto('/login');
		throw new ApiError(401, 'unauthorized', 'Your session has expired. Please sign in again.');
	}

	const text = await res.text();
	const data: unknown = text.length > 0 ? JSON.parse(text) : undefined;

	if (!res.ok) {
		const { code, message } = extractError(data);
		throw new ApiError(res.status, code, message);
	}

	return schema.parse(data);
}

function extractError(data: unknown): { code: string; message: string } {
	if (
		data !== null &&
		typeof data === 'object' &&
		'error' in data &&
		data.error !== null &&
		typeof data.error === 'object'
	) {
		const e = data.error as Record<string, unknown>;
		return {
			code: typeof e['code'] === 'string' ? e['code'] : 'error',
			message: typeof e['message'] === 'string' ? e['message'] : 'Request failed'
		};
	}
	return { code: 'error', message: 'Request failed' };
}

export const api = {
	get: <T>(path: string, schema: z.ZodType<T>) => request('GET', path, schema),
	post: <T>(path: string, schema: z.ZodType<T>, body?: unknown) =>
		request('POST', path, schema, { body }),
	patch: <T>(path: string, schema: z.ZodType<T>, body?: unknown) =>
		request('PATCH', path, schema, { body }),
	del: <T>(path: string, schema: z.ZodType<T>) => request('DELETE', path, schema),
	postForm: <T>(path: string, schema: z.ZodType<T>, form: FormData) =>
		request('POST', path, schema, { form }),
	device: <T>(path: string, schema: z.ZodType<T>, device: DeviceAuth, body?: unknown) =>
		request('POST', path, schema, { device, body, auth: false })
};
