import { z } from 'zod';
import { api } from '$lib/api/client';
import { pageSchema, queryString, type Paged, type PageRequest } from '$lib/api/page';
import {
	smsRecipientSchema,
	smsRecipientListSchema,
	smsMessageSchema,
	smsCandidateListSchema,
	type SmsRecipient,
	type SmsCandidate,
	type SmsMessage,
	type SmsStatus
} from '$lib/schemas/sms';

export interface CreateRecipientInput {
	user_id: string;
	/** Optional: also writes the number onto the account. */
	phone_number?: string;
	notify_cabinet_opened: boolean;
	notify_access_denied: boolean;
	notify_door_opened: boolean;
}

export interface UpdateRecipientInput {
	/** Writes through to the user record. */
	phone_number?: string;
	notify_cabinet_opened?: boolean;
	notify_access_denied?: boolean;
	notify_door_opened?: boolean;
	is_active?: boolean;
}

export function listRecipients(): Promise<SmsRecipient[]> {
	return api.get('/sms/recipients', smsRecipientListSchema);
}

/** Staff who are eligible but not subscribed yet. */
export function listCandidates(): Promise<SmsCandidate[]> {
	return api.get('/sms/candidates', smsCandidateListSchema);
}

export function createRecipient(input: CreateRecipientInput): Promise<SmsRecipient> {
	return api.post('/sms/recipients', smsRecipientSchema, input);
}

export function updateRecipient(id: string, input: UpdateRecipientInput): Promise<SmsRecipient> {
	return api.patch(`/sms/recipients/${id}`, smsRecipientSchema, input);
}

export function deleteRecipient(id: string): Promise<void> {
	return api.del(`/sms/recipients/${id}`, z.void());
}

export function sendTest(id: string): Promise<SmsMessage> {
	return api.post(`/sms/recipients/${id}/test`, smsMessageSchema);
}

export interface MessageFilter extends PageRequest {
	status?: SmsStatus;
}

const messagePageSchema = pageSchema(smsMessageSchema);

export function listMessages(filter: MessageFilter = {}): Promise<Paged<SmsMessage>> {
	return api.get(`/sms/messages${queryString({ ...filter })}`, messagePageSchema);
}
