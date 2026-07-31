import { api } from '$lib/api/client';
import {
	enrollmentSchema,
	enrollmentListSchema,
	type Enrollment
} from '$lib/schemas/enrollment';

export function verifyCode(code: string): Promise<Enrollment> {
	return api.post('/enrollment/verify-code', enrollmentSchema, { code });
}

export function myEnrollments(): Promise<Enrollment[]> {
	return api.get('/enrollment/mine', enrollmentListSchema);
}
