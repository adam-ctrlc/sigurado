<script lang="ts">
	import { Badge } from '$lib/components/ui/badge';
	import type { Role } from '$lib/schemas/user';

	let { role }: { role: Role } = $props();

	const labels: Record<Role, string> = {
		student: 'Student',
		faculty: 'Faculty',
		admin: 'Admin'
	};

	const classes: Record<Role, string> = {
		student: 'bg-muted text-muted-foreground',
		faculty: 'bg-accent text-accent-foreground',
		admin: 'bg-primary text-primary-foreground'
	};
</script>

<Badge class={classes[role]}>{labels[role]}</Badge>
