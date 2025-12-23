<script lang="ts">
	import { AppBar } from '@skeletonlabs/skeleton-svelte';
</script>

<AppBar class=" shadow-md shadow-neutral-950">
	<div class="container mx-auto flex items-center justify-between px-4">
		<a class="flex gap-2 text-xl font-bold" href="/"
			><img src="/logo.svg" alt="DragoDNS Logo" height="28" width="28" />DragoDNS</a
		>
		<nav class="flex gap-3">
			<a class="rounded px-3 py-2 hover:bg-surface-100 dark:hover:bg-surface-800" href="/#features"
				>Features</a
			>
			<a class="rounded px-3 py-2 hover:bg-surface-100 dark:hover:bg-surface-800" href="/#pricing"
				>Pricing</a
			>
			<a class="rounded px-3 py-2 hover:bg-surface-100 dark:hover:bg-surface-800" href="/docs"
				>Docs</a
			>
			<a
				class="rounded bg-primary-500 px-4 py-2 text-white transition-colors hover:bg-primary-600"
				href="/auth">Sign Up</a
			>
		</nav>
	</div>
</AppBar>
