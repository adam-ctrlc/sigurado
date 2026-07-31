import { z } from 'zod';

export const checkoutSchema = z.object({
	id: z.string(),
	user_id: z.string(),
	user_name: z.string().nullish(),
	note: z.string(),
	photo_path: z.string().nullish(),
	photo_mime: z.string().nullish(),
	access_session_id: z.string().nullish(),
	box_device_id: z.string().nullish(),
	created_at: z.string()
});
export type Checkout = z.infer<typeof checkoutSchema>;

export const checkoutListSchema = z.array(checkoutSchema);
