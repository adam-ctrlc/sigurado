import { studentGuide } from '$lib/guides/student';
import { facultyGuide } from '$lib/guides/faculty';
import { adminGuide } from '$lib/guides/admin';
import type { Guide, GuideSlug } from '$lib/guides/types';
import type { Role } from '$lib/schemas/user';

export const GUIDES: Record<GuideSlug, Guide> = {
	students: studentGuide,
	faculty: facultyGuide,
	admin: adminGuide
};

/** The guide written for a role, which is where that person lands by default. */
export function guideForRole(role: Role): Guide {
	if (role === 'admin') return adminGuide;
	if (role === 'faculty') return facultyGuide;
	return studentGuide;
}

/**
 * Which guides somebody may read. Staff see the student guide too, because
 * helping a student is easier with the instructions they were given in front of
 * you. Students see only their own.
 */
export function guidesForRole(role: Role): Guide[] {
	if (role === 'admin') return [studentGuide, facultyGuide, adminGuide];
	if (role === 'faculty') return [studentGuide, facultyGuide];
	return [studentGuide];
}
