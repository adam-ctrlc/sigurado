/**
 * Where somebody who has not enrolled yet is still allowed to go.
 *
 * Enrollment, because that is the way out. The guides, because they explain how
 * to do it. Everything else is held until a finger is bound, since none of it
 * works for a person the readers cannot recognise.
 */
export const ALLOWED_WHILE_UNENROLLED = ['/enrollment', '/guides'];

export function allowedWhileUnenrolled(pathname: string): boolean {
	return ALLOWED_WHILE_UNENROLLED.some(
		(path) => pathname === path || pathname.startsWith(`${path}/`)
	);
}
