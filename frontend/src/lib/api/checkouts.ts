import { z } from 'zod';
import { api } from '$lib/api/client';
import { pageSchema, queryString, type Paged, type PageRequest } from '$lib/api/page';
import { checkoutSchema, type Checkout } from '$lib/schemas/checkout';

export interface CreateCheckoutInput {
	note: string;
	/** The code the cabinet showed on its LCD. Required. */
	code: string;
	photo?: File;
	accessSessionId?: string;
	boxDeviceId?: string;
}

export interface CheckoutFilter extends PageRequest {
	/** Free text over the note, plus the person's name for staff. */
	q?: string;
	user_id?: string;
}

const checkoutPageSchema = pageSchema(checkoutSchema);

export function listCheckouts(filter: CheckoutFilter = {}): Promise<Paged<Checkout>> {
	return api.get(`/checkouts${queryString({ ...filter })}`, checkoutPageSchema);
}

export function getCheckout(id: string): Promise<Checkout> {
	return api.get(`/checkouts/${id}`, checkoutSchema);
}

export function createCheckout(input: CreateCheckoutInput): Promise<Checkout> {
	const form = new FormData();
	form.set('note', input.note);
	form.set('code', input.code);
	if (input.photo) form.set('photo', input.photo);
	if (input.accessSessionId) form.set('access_session_id', input.accessSessionId);
	if (input.boxDeviceId) form.set('box_device_id', input.boxDeviceId);
	return api.postForm('/checkouts', checkoutSchema, form);
}

const pendingCodeSchema = z.object({
	waiting: z.boolean(),
	expires_at: z.string().nullish(),
	issued_at: z.string().nullish()
});

export type PendingCode = z.infer<typeof pendingCodeSchema>;

/** Whether the cabinet has issued a code for me that is still good. */
export function pendingCode(): Promise<PendingCode> {
	return api.get('/checkouts/pending-code', pendingCodeSchema);
}
