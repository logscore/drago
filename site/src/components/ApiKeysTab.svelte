<script lang="ts">
	import type { ApiKey } from '$lib/types';
	import { createQuery } from '@tanstack/svelte-query';

	interface Props {
		userId: string | undefined;
	}
	const { userId }: Props = $props();

	const apiKeysQuery = createQuery<ApiKey[]>(() => ({
		queryKey: ['access_keys', userId],
		queryFn: async () => {
			if (!userId) {
				throw new Error('User ID is required');
			}
			const response = await fetch(`http://127.0.0.1:8080/access_tokens?user_id=${userId}`);
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return response.json();
		},
		enabled: !!userId
	}));
</script>

<h1 class="mb-2 text-lg font-semibold">API Keys</h1>
<p class="mb-4 text-sm text-neutral-400">Manage your API keys used on the Drago daemon</p>
<div class="w-full rounded p-4">
	<p>No API keys created yet.</p>
	<button class="btn preset-filled"> Create Key </button>
</div>
