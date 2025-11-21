<script lang="ts">
	import { goto } from '$app/navigation';
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { createQuery } from '@tanstack/svelte-query';

	const session = authClient.useSession();
	let tab_value = $state('domains');

	async function handleSignOut() {
		await authClient.signOut();
		goto('/');
	}

	const fetchRecords = async () => {
		const response = await fetch(`http://127.0.0.1:8080/records?user_id=${$session.data?.user.id}`);
		if (!response.ok) {
			throw new Error('Network response was not ok');
		}
		return response.json();
	};

	const recordsQuery = createQuery<ZoneRecordData>(() => ({
		queryKey: ['records'],
		queryFn: fetchRecords
	}));

	const fetchAccessKeys = async () => {
		const response = await fetch(
			`http://127.0.0.1:8080/access_keys?user_id=${$session.data?.user.id}`
		);
		if (!response.ok) {
			throw new Error('Network response was not ok');
		}
		return response.json();
	};

	const accessKeysQuery = createQuery(() => ({
		queryKey: ['access_keys'],
		queryFn: fetchAccessKeys
	}));
</script>

<div class="mx-auto min-h-screen w-7xl p-4">
	<main class="w-full">
		<Tabs value={tab_value} onValueChange={(details) => (tab_value = details.value)}>
			<Tabs.List class="mb-6 flex w-full gap-2 border-b pb-2">
				<Tabs.Trigger value="domains" class="px-3 py-2 hover:bg-neutral-800">Domains</Tabs.Trigger>
				<Tabs.Trigger value="api keys" class="px-3 py-2 hover:bg-neutral-800">API Keys</Tabs.Trigger
				>
				<Tabs.Trigger value="settings" class="px-3 py-2 hover:bg-neutral-800">Settings</Tabs.Trigger
				>
				<Tabs.Indicator />
			</Tabs.List>

			<Tabs.Content value="domains" class="w-full">
				<h1 class="mb-2 text-lg font-semibold">Domains</h1>
				<p class="mb-4 text-sm text-neutral-400">Manage and monitor your registered domains.</p>
				<div class="w-full rounded p-4">
					<p>No domains yet.</p>
					<button class="btn preset-filled"> Add Domain </button>
				</div>
			</Tabs.Content>

			<Tabs.Content value="api keys" class="w-full">
				<h1 class="mb-2 text-lg font-semibold">API Keys</h1>
				<p class="mb-4 text-sm text-neutral-400">Manage your API keys used on the Drago daemon</p>
				<div class="w-full rounded p-4">
					<p>No API keys created yet.</p>
					<button class="btn preset-filled"> Create Key </button>
				</div>
			</Tabs.Content>

			<Tabs.Content value="settings" class="w-full">
				<h1 class="mb-2 text-lg font-semibold">Settings</h1>
				<p class="mb-4 text-sm text-neutral-400">Manage your account settings.</p>

				<div class="w-full rounded p-4">
					<button onclick={handleSignOut} class="hover:bg-red-550 btn bg-red-500">
						Sign Out
					</button>
				</div>
			</Tabs.Content>
		</Tabs>
	</main>
</div>
