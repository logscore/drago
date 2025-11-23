<script lang="ts">
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import ApiKeysTab from '../../../components/ApiKeysTab.svelte';
	import SettingsTab from '../../../components/SettingsTab.svelte';
	import DomainsTab from '../../../components/DomainsTab.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import type { ZoneRecordData } from '$lib/types';

	let tab_value = $state('domains');

	let jwtData = $state<string>('');

	$effect(() => {
		async function fetchJWT() {
			const { data, error } = await authClient.token();
			console.log(data);
			if (error) {
				console.error(error.message);
			}
			if (data) {
				jwtData = data.token;
			}
		}
		fetchJWT();
	});

	const recordsQuery = createQuery<ZoneRecordData>(() => ({
		queryKey: ['records', jwtData],
		queryFn: async () => {
			if (!jwtData) {
				throw new Error('User not signed in');
			}
			const response = await fetch(`http://127.0.0.1:8080/records`, {
				headers: {
					Authorization: `Bearer ${jwtData}`
				}
			});
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return response.json();
		},
		enabled: !!jwtData
	}));
</script>

<div class="mx-auto min-h-screen max-w-7xl p-4">
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
				<DomainsTab {jwtData} />
			</Tabs.Content>

			<Tabs.Content value="api keys" class="w-full">
				<ApiKeysTab {jwtData} recordScopes={recordsQuery.data!} />
			</Tabs.Content>

			<Tabs.Content value="settings" class="w-full">
				<SettingsTab {jwtData} />
			</Tabs.Content>
		</Tabs>
	</main>
</div>
