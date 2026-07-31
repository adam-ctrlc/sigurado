import { z } from 'zod';

export const enrollmentStatusSchema = z.enum([
	'pending_code',
	'awaiting_scan',
	'bound',
	'expired',
	'cancelled'
]);
export type EnrollmentStatus = z.infer<typeof enrollmentStatusSchema>;

export const enrollmentSchema = z.object({
	id: z.string(),
	device_id: z.string(),
	status: enrollmentStatusSchema,
	created_at: z.string(),
	code_expires_at: z.string(),
	bound_at: z.string().nullish()
});
export type Enrollment = z.infer<typeof enrollmentSchema>;

export const enrollmentListSchema = z.array(enrollmentSchema);
