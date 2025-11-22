<script lang="ts">
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import ApiKeysTab from '../../../components/ApiKeysTab.svelte';
	import SettingsTab from '../../../components/SettingsTab.svelte';
	import DomainsTab from '../../../components/DomainsTab.svelte';

	const session = authClient.useSession();

	let tab_value = $state('domains');

	// Derive user ID reactively
	const userId = $derived($session.data?.user?.id);

	// const apiKeysQuery = createQuery(() => ({
	// 	queryKey: ['api_keys', userId],
	// 	queryFn: async () => {
	// 		if (!userId) {
	// 			throw new Error('User ID is required');
	// 		}
	// 		const response = await fetch(`http://127.0.0.1:8080/api_keys?user_id=${userId}`);
	// 		if (!response.ok) {
	// 			throw new Error('Network response was not ok');
	// 		}
	// 		return response.json();
	// 	},
	// 	enabled: !!userId
	// }));
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
				<DomainsTab {userId} />
			</Tabs.Content>

			<Tabs.Content value="api keys" class="w-full">
				<ApiKeysTab {userId} />
			</Tabs.Content>

			<Tabs.Content value="settings" class="w-full">
				<SettingsTab {userId} />
			</Tabs.Content>
		</Tabs>
	</main>
</div>
