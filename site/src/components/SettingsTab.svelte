<script lang="ts">
	import { goto } from '$app/navigation';
	import { authClient } from '$lib/auth/authClient';
	import type { AccessToken, AddAccessToken } from '$lib/types';
	import { createQuery } from '@tanstack/svelte-query';

	interface Props {
		userId: string | undefined;
	}
	const { userId }: Props = $props();

	let deleteDialogOpen = false;
	let deleteTokenId: string | undefined;

	async function handleSignOut() {
		await authClient.signOut();
		goto('/');
	}

	const accessTokensQuery = createQuery<AccessToken[]>(() => ({
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

	async function deleteToken(tokenId: string) {
		// TODO: Implement API call to delete record
		try {
			const response = await fetch(`http://127.0.0.1:8080/access_token?token_id=${tokenId}`, {
				method: 'DELETE'
			});

			if (!response.ok) {
				throw new Error('Network response was not ok');
			}

			// Close dialog
			deleteDialogOpen = false;

			// Refresh records
			accessTokensQuery.refetch();
		} catch (error) {
			console.error('Error deleting record:', error);
			operationMessage = `Error deleting record: ${error}`;
			operationSuccess = false;
		}
	}
</script>

<h1 class="mb-2 text-lg font-semibold">Settings</h1>
<p class="mb-4 text-sm text-neutral-400">Manage your account settings.</p>

<div class="w-full rounded">
	{#if accessTokensQuery.isPending}
		<p class="p-4 text-neutral-500">Loading records...</p>
	{:else if accessTokensQuery.isError}
		<p class="p-4 text-red-500">
			Error loading records: {accessTokensQuery.error || 'Unknown error'}
		</p>
	{:else if accessTokensQuery.isSuccess}
		<div class="mb-8 space-y-8">
			<div class="rounded border border-neutral-800 bg-neutral-900 p-4">
				<div class="mb-4 flex items-center justify-between border-b border-neutral-800 pb-2">
					<div>
						<h2 class="text-2xl font-bold">DNS Access Tokens</h2>
					</div>
					<button
						class="preset-filled-500 btn font-semibold"
						onclick={() => console.log('Adding Provider')}
					>
						+ Add Provider
					</button>
				</div>
				<div class="grid grid-cols-12 gap-4 px-2 text-xs font-bold text-neutral-400">
					<div class="col-span-4">NAME</div>
					<div class="col-span-6">CREATED DATE</div>
					<div class="col-span-2 pr-2 text-right">ACTIONS</div>
				</div>
				{#each accessTokensQuery.data as accessToken}
					<div
						class="mb-2 grid grid-cols-12 rounded border border-neutral-800 bg-neutral-900 px-4 py-3"
					>
						<!-- Access Token Field -->
						<div class="col-span-4 truncate break-all">
							<span class="font-mono text-sm">
								{accessToken.name}
							</span>
						</div>

						<div class="col-span-6 truncate break-all">
							<span class="font-mono text-sm">
								{accessToken.created_on}
							</span>
						</div>

						<!-- Actions -->
						<div class="col-span-2 flex justify-end">
							<button
								onclick={(e) => {
									e.preventDefault();
									onDelete(accessToken.id);
								}}
								class="btn preset-filled-error-500 btn-sm"
							>
								Delete
							</button>
						</div>
					</div>
				{/each}
			</div>
		</div>

		{#if accessTokensQuery && !accessTokensQuery.data}
			<div class="w-full rounded p-4">
				<p>No DNS providers yet.</p>
			</div>
		{/if}
	{/if}

	<button onclick={handleSignOut} class="hover:bg-red-550 btn bg-red-500"> Sign Out </button>
</div>
