<script lang="ts">
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import ApiKeysTab from '../../../components/ApiKeysTab.svelte';
	import SettingsTab from '../../../components/SettingsTab.svelte';
	import DomainsTab from '../../../components/DomainsTab.svelte';
	import { createQuery } from '@tanstack/svelte-query';
	import type { ZoneRecordData } from '$lib/types';
	import { PUBLIC_BACKEND_URL } from '$env/static/public';

	let tab = $state<string>('domains');
	let missingToken = $state<boolean>(false);

	let jwtData = $state<string>('');

	$effect(() => {
		async function fetchJWT() {
			const { data, error } = await authClient.token();
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

			missingToken = false;

			const response = await fetch(`${PUBLIC_BACKEND_URL}/records`, {
				headers: {
					Authorization: `Bearer ${jwtData}`
				}
			});

			if (!response.ok) {
				if (response.status === 404) {
					missingToken = true;
					throw new Error('Missing DNS access token');
				}

				throw new Error('Network response was not ok');
			}
			return response.json();
		},
		enabled: !!jwtData
	}));
</script>

<div class="mx-auto min-h-screen max-w-7xl p-4">
	<main class="w-full">
		<Tabs value={tab} onValueChange={(details) => (tab = details.value)}>
			<Tabs.List class="mb-6 flex w-full gap-2 border-b pb-2">
				<Tabs.Trigger value="domains" class="px-3 py-2 hover:bg-neutral-800">Domains</Tabs.Trigger>
				<Tabs.Trigger value="api keys" class="px-3 py-2 hover:bg-neutral-800">API Keys</Tabs.Trigger
				>
				<Tabs.Trigger value="settings" class="px-3 py-2 hover:bg-neutral-800">Settings</Tabs.Trigger
				>
				<Tabs.Indicator />
			</Tabs.List>

			<Tabs.Content value="domains" class="w-full">
				<DomainsTab {jwtData} {recordsQuery} bind:tab {missingToken} />
			</Tabs.Content>

			<Tabs.Content value="api keys" class="w-full">
				<ApiKeysTab {jwtData} {recordsQuery} bind:tab {missingToken} />
			</Tabs.Content>

			<Tabs.Content value="settings" class="w-full">
				<SettingsTab {jwtData} />
			</Tabs.Content>
		</Tabs>
	</main>
</div>
