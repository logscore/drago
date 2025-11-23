<script lang="ts">
	import { goto } from '$app/navigation';
	import { authClient } from '$lib/auth/authClient';
	import { XIcon } from '@lucide/svelte';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import type { AccessToken, AddAccessToken } from '$lib/types';
	import { createQuery } from '@tanstack/svelte-query';

	interface Props {
		userId: string | undefined;
	}
	const { userId }: Props = $props();

	// Dialog state
	let addDialogOpen = $state(false);
	let deleteDialogOpen = $state(false);

	// Current token for operations
	let currentTokenId = $state<string>('');

	// Form state for adding a token
	let tokenName = $state('');
	let tokenValue = $state('');

	// Operation feedback
	let operationMessage = $state<string>('');
	let operationSuccess = $state<boolean>(false);

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

	async function addAccessToken() {
		// Form validation
		if (!tokenName) {
			operationMessage = 'Please enter a token name';
			operationSuccess = false;
			return;
		}

		try {
			const response = await fetch('http://127.0.0.1:8080/access_token', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json'
				},
				body: JSON.stringify({
					user_id: userId,
					name: tokenName,
					token: tokenValue
				})
			});

			if (!response.ok) {
				throw new Error('Network response was not ok');
			}

			// Handle success
			operationMessage = 'Access token added successfully';
			operationSuccess = true;

			// Reset form
			tokenName = '';

			// Close dialog
			addDialogOpen = false;

			// Refresh records
			accessTokensQuery.refetch();
		} catch (error) {
			console.error('Error adding access token:', error);
			operationMessage = `Error adding access token: ${error}`;
			operationSuccess = false;
		}
	}

	async function deleteToken(tokenId: string) {
		try {
			const response = await fetch(`http://127.0.0.1:8080/access_token?token_id=${tokenId}`, {
				method: 'DELETE'
			});

			if (!response.ok) {
				throw new Error('Network response was not ok');
			}

			// Handle success
			operationMessage = 'Access token deleted successfully';
			operationSuccess = true;

			// Close dialog
			deleteDialogOpen = false;

			// Refresh records
			accessTokensQuery.refetch();
		} catch (error) {
			console.error('Error deleting access token:', error);
			operationMessage = `Error deleting access token: ${error}`;
			operationSuccess = false;
		}
	}

	function openAddDialog() {
		// Reset form and message
		tokenName = '';
		tokenValue = '';
		operationMessage = '';
		addDialogOpen = true;
	}

	function openDeleteDialog(tokenId: string) {
		currentTokenId = tokenId;
		operationMessage = '';
		deleteDialogOpen = true;
	}

	const animation =
		'transition transition-discrete opacity-0 translate-y-[100px] starting:data-[state=open]:opacity-0 starting:data-[state=open]:translate-y-[100px] data-[state=open]:opacity-100 data-[state=open]:translate-y-0';
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
					<button class="preset-filled-500 btn font-semibold" onclick={openAddDialog}>
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
								onclick={() => openDeleteDialog(accessToken.id)}
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

<!-- Add Access Token Dialog -->
<Dialog
	open={addDialogOpen}
	onInteractOutside={() => {
		addDialogOpen = false;
	}}
>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content
				class="w-full max-w-xl space-y-4 card bg-surface-100-900 p-4 shadow-xl {animation}"
			>
				<header class="flex items-center justify-between">
					<Dialog.Title class="text-lg font-bold">Add DNS Access Token</Dialog.Title>
					<Dialog.CloseTrigger
						class="btn-icon hover:preset-tonal"
						onclick={() => {
							addDialogOpen = false;
						}}
					>
						<XIcon class="size-4" />
					</Dialog.CloseTrigger>
				</header>

				<Dialog.Description class="sr-only">Form to add a new DNS access token</Dialog.Description>

				<!-- Operation feedback -->
				{#if operationMessage}
					<div
						class={`rounded p-2 ${operationSuccess ? 'bg-green-800 text-green-200' : 'bg-red-800 text-red-200'}`}
					>
						{operationMessage}
					</div>
				{/if}

				<form onsubmit={addAccessToken} class="space-y-4">
					<!-- Token Name -->
					<div>
						<label for="token-name" class="mb-1 block text-sm font-medium text-gray-300">
							Name
						</label>
						<input
							id="token-name"
							name="name"
							type="text"
							placeholder="Cool Token Name"
							bind:value={tokenName}
							max="255"
							required
							class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-gray-100 shadow-sm sm:text-sm"
						/>
					</div>

					<div>
						<label for="token-value" class="mb-1 block text-sm font-medium text-gray-300">
							Value
						</label>
						<input
							id="token-value"
							name="value"
							type="text"
							placeholder="4e545fc3ld..."
							bind:value={tokenValue}
							required
							class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-gray-100 shadow-sm sm:text-sm"
						/>
					</div>

					<!-- Footer -->
					<footer class="flex justify-end gap-2 pt-4">
						<Dialog.CloseTrigger
							class="btn preset-tonal"
							onclick={() => {
								addDialogOpen = false;
							}}>Cancel</Dialog.CloseTrigger
						>
						<button type="submit" class="btn preset-filled-primary-500 font-semibold">
							+ Add Token
						</button>
					</footer>
				</form>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>

<!-- Delete Access Token Dialog -->
<Dialog
	open={deleteDialogOpen}
	onInteractOutside={() => {
		deleteDialogOpen = false;
	}}
>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-surface-50-950/50" />
		<Dialog.Positioner class="fixed inset-0 z-50 flex items-center justify-center p-4">
			<Dialog.Content
				class="w-full max-w-md space-y-4 card border border-red-500/20 bg-surface-100-900 p-6 shadow-xl {animation}"
			>
				<header class="flex items-center justify-between">
					<Dialog.Title class="text-lg font-bold text-red-500">Delete Access Token</Dialog.Title>
					<Dialog.CloseTrigger
						class="btn-icon hover:preset-tonal"
						onclick={() => {
							deleteDialogOpen = false;
						}}
					>
						<XIcon class="size-4" />
					</Dialog.CloseTrigger>
				</header>

				<!-- Operation feedback -->
				{#if operationMessage}
					<div
						class={`rounded p-2 ${operationSuccess ? 'bg-green-800 text-green-200' : 'bg-red-800 text-red-200'}`}
					>
						{operationMessage}
					</div>
				{/if}

				<Dialog.Description>
					Are you sure you want to delete this DNS access token? This action cannot be undone.
				</Dialog.Description>

				<footer class="flex justify-end gap-2">
					<Dialog.CloseTrigger
						class="btn preset-tonal"
						onclick={() => {
							deleteDialogOpen = false;
						}}>Cancel</Dialog.CloseTrigger
					>
					<button
						type="button"
						class="btn preset-filled-error-500"
						onclick={() => deleteToken(currentTokenId)}
					>
						Delete
					</button>
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
