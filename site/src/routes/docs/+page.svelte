<script lang="ts">
	import Docs from '../../components/Docs.svelte';
	import Header from '../../components/Header.svelte';
</script>

<svelte:head>
	<title>Documentation — DragoDNS</title>
	<meta name="description" content="Official documentation for Drago DNS." />
</svelte:head>

<div class="min-h-screen">
	<Header />

	<main class="container mx-auto flex justify-center px-4 py-12">
		<Docs />
	</main>
</div>
