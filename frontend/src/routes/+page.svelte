<script lang="ts">
	import SiteHeader from '$lib/components/landing/site-header.svelte';
	import Hero from '$lib/components/landing/hero.svelte';
	import TrackRecord from '$lib/components/landing/track-record.svelte';
	import SequenceSteps from '$lib/components/landing/sequence-steps.svelte';
	import GuaranteesGrid from '$lib/components/landing/guarantees-grid.svelte';
	import AuditPreview from '$lib/components/landing/audit-preview.svelte';
	import HardwareGrid from '$lib/components/landing/hardware-grid.svelte';
	import ClosingCall from '$lib/components/landing/closing-call.svelte';
	import SiteFooter from '$lib/components/landing/site-footer.svelte';
	import { auth } from '$lib/stores/auth.svelte';

	const href = $derived(auth.isAuthed ? '/dashboard' : '/login');
	const label = $derived(auth.isAuthed ? 'Open the dashboard' : 'Sign in');
</script>

<svelte:head><title>Sigurado - Dual-biometric materials accountability</title></svelte:head>

<!-- `dark` rides along with `landing` so the components' own `dark:` variants
     still resolve inside, exactly as they do for the dark-family themes on
     <html>. The page ignores the theme picker on purpose: this palette is the
     only one it is drawn for, and the toggle stays behind sign-in. -->
<div class="dark gold min-h-screen bg-background text-foreground">
	<SiteHeader {href} {label} />
	<Hero {href} {label} />
	<TrackRecord {href} {label} />
	<SequenceSteps />
	<GuaranteesGrid />
	<AuditPreview {href} {label} />
	<HardwareGrid />
	<ClosingCall {href} {label} />
	<SiteFooter />
</div>
