<script lang="ts">
	import GuideView from '$lib/components/guide-view.svelte';
	import { GUIDES } from '$lib/guides';

	const guide = GUIDES.admin;
</script>

<svelte:head><title>{guide.label} - Guides - Sigurado</title></svelte:head>

<GuideView {guide} />
