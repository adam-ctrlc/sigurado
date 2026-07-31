import { api } from '$lib/api/client';
import {
	deviceSchema,
	deviceListSchema,
	deviceWithSecretSchema,
	deviceConnectionListSchema,
	type Device,
	type DeviceConnection,
	type DeviceWithSecret,
	type DeviceKind
} from '$lib/schemas/device';

export function listDevices(): Promise<Device[]> {
	return api.get('/devices', deviceListSchema);
}

export function createDevice(
	name: string,
	kind: DeviceKind,
	smsCapable = false
): Promise<DeviceWithSecret> {
	return api.post('/devices', deviceWithSecretSchema, {
		name,
		kind,
		sms_capable: smsCapable
	});
}

export interface UpdateDeviceInput {
	sms_capable?: boolean;
	is_active?: boolean;
}

export function updateDevice(id: string, input: UpdateDeviceInput): Promise<Device> {
	return api.patch(`/devices/${id}`, deviceSchema, input);
}

export function getDevice(id: string): Promise<Device> {
	return api.get(`/devices/${id}`, deviceSchema);
}

export function rotateSecret(id: string): Promise<DeviceWithSecret> {
	return api.post(`/devices/${id}/rotate-secret`, deviceWithSecretSchema);
}

export function listConnections(id: string): Promise<DeviceConnection[]> {
	return api.get(`/devices/${id}/connections`, deviceConnectionListSchema);
}
