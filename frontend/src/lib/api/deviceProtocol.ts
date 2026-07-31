import { api, type DeviceAuth } from '$lib/api/client';
import {
	scanResponseSchema,
	enrollCodeResponseSchema,
	enrollBindResponseSchema,
	type ScanResponse,
	type EnrollCodeResponse,
	type EnrollBindResponse
} from '$lib/schemas/scan';

export function deviceScan(device: DeviceAuth, fingerToken: string): Promise<ScanResponse> {
	return api.device('/device/scan', scanResponseSchema, device, { finger_token: fingerToken });
}

export function requestEnrollCode(
	device: DeviceAuth,
	fingerToken: string | null
): Promise<EnrollCodeResponse> {
	return api.device('/device/enroll/request-code', enrollCodeResponseSchema, device, {
		finger_token: fingerToken
	});
}

export function deviceBind(device: DeviceAuth, fingerToken: string): Promise<EnrollBindResponse> {
	return api.device('/device/enroll/bind', enrollBindResponseSchema, device, {
		finger_token: fingerToken
	});
}
