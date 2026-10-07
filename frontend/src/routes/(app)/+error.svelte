<script lang="ts">
	import { page } from '$app/state';
	import ErrorView from '$lib/components/error-view.svelte';
</script>

<svelte:head><title>{page.status} - Sigurado</title></svelte:head>

<!-- Inside the app shell on purpose: the sidebar stays put, so a wrong turn in
     here is one click from being undone rather than a dead end. -->
<div class="flex flex-1 items-center justify-center overflow-hidden">
	<ErrorView status={page.status} detail={page.error?.message} />
</div>
