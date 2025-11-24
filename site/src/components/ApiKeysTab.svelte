<script lang="ts">
	import type { ApiKey, ZoneRecordData } from '$lib/types';
	import { Check, Copy, XIcon } from '@lucide/svelte';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import { createMutation, createQuery } from '@tanstack/svelte-query';

	interface Props {
		jwtData: string;
		recordScopes: ZoneRecordData;
	}

	let { jwtData, recordScopes }: Props = $props();

	// Single source of truth for the "add key" form
	let addKeyData = $state({
		name: '',
		scope: '', // will hold the selected record's id
		zoneId: '',
		recordId: ''
	});

	let apiKeyReturned = $state('');

	// Dialog state
	let addDialogOpen = $state(false);
	let deleteDialogOpen = $state(false);

	// Current key for operations
	let currentApiKeyId = $state<string>('');

	// Operation feedback
	let operationMessage = $state<string>('');
	let operationSuccess = $state<boolean>(false);

	const apiKeysQuery = createQuery<ApiKey[]>(() => ({
		queryKey: ['access_tokens', jwtData],
		queryFn: async () => {
			if (!jwtData) {
				throw new Error('User not signed in');
			}
			const response = await fetch(`http://127.0.0.1:8080/api_keys`, {
				headers: {
					Authorization: `Bearer ${jwtData}`
				}
			});
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return await response.json();
		},
		enabled: !!jwtData
	}));

	const addKeyMutation = createMutation(() => ({
		mutationFn: async (data: { name: string; scope: string; zoneId: string; recordId: string }) => {
			const response = await fetch('http://127.0.0.1:8080/api_key', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: `Bearer ${jwtData}`
				},
				body: JSON.stringify(data)
			});

			if (!response.ok) {
				throw new Error('Failed to add token');
			}

			return await response.json();
		},
		onSuccess: (data) => {
			apiKeyReturned = data;
			operationSuccess = true;
			operationMessage = 'API key created successfully.';
			// reset form
			addKeyData.name = '';
			addKeyData.scope = '';
			addKeyData.zoneId = '';
			addKeyData.recordId = '';
			apiKeysQuery.refetch();
		},
		onError: (error: unknown) => {
			operationSuccess = false;
			operationMessage = error instanceof Error ? error.message : 'Failed to add token.';
		}
	}));

	const deleteKeyMutation = createMutation(() => ({
		mutationFn: async (data: { key_id: string }) => {
			const response = await fetch(`http://127.0.0.1:8080/api_key?key_id=${data.key_id}`, {
				method: 'DELETE',
				headers: {
					'Content-Type': 'application/json',
					Authorization: `Bearer ${jwtData}`
				}
			});

			if (!response.ok) {
				throw new Error('Failed to add token');
			}

			return await response.json();
		},
		onSuccess: () => {
			operationSuccess = true;
			operationMessage = 'API key deleted successfully.';
			// reset form
			addKeyData.name = '';
			addKeyData.scope = '';
			addKeyData.zoneId = '';
			addKeyData.recordId = '';
			apiKeysQuery.refetch();
			setTimeout(() => {
				deleteDialogOpen = false;
			}, 3000);
		},
		onError: (error: unknown) => {
			operationSuccess = false;
			operationMessage = error instanceof Error ? error.message : 'Failed to add token.';
		}
	}));

	function submitApiKeyData(event: Event) {
		event.preventDefault();

		addKeyMutation.mutate({
			name: addKeyData.name,
			scope: addKeyData.scope,
			zoneId: addKeyData.zoneId,
			recordId: addKeyData.recordId
		});
	}

	function openAddDialog() {
		addKeyData.name = '';
		addKeyData.scope = '';
		addKeyData.zoneId = '';
		addKeyData.recordId = '';
		operationMessage = '';
		addDialogOpen = true;
	}

	function openDeleteDialog(keyId: string) {
		currentApiKeyId = keyId;
		operationMessage = '';
		deleteDialogOpen = true;
	}

	let copied = $state(false);
	let timeoutId: number | null = null;

	async function copyKey() {
		try {
			await navigator.clipboard.writeText(apiKeyReturned);
			copied = true;

			if (timeoutId) clearTimeout(timeoutId);
			timeoutId = window.setTimeout(() => {
				copied = false;
			}, 1500);
		} catch (err) {
			console.error('Failed to copy API key:', err);
		}
	}

	// When a record is selected, we populate zoneId + recordId in state
	function handleRecordSelect(event: Event) {
		const select = event.target as HTMLSelectElement;
		const option = select.selectedOptions[0];

		if (!option) {
			addKeyData.zoneId = '';
			addKeyData.recordId = '';
			return;
		}

		const zoneId = option.dataset.zoneId ?? '';
		const recordId = option.dataset.recordId ?? option.value;

		addKeyData.zoneId = zoneId;
		addKeyData.recordId = recordId;
	}

	const animation =
		'transition transition-discrete opacity-0 translate-y-[100px] starting:data-[state=open]:opacity-0 starting:data-[state=open]:translate-y-[100px] data-[state=open]:opacity-100 data-[state=open]:translate-y-0';
</script>

<h1 class="mb-2 text-lg font-semibold">API Keys</h1>
<p class="mb-4 text-sm text-neutral-400">
	Manage your API keys used to manage DNS records on the Drago daemon
</p>

<div class="w-full rounded">
	<div class="mb-8 space-y-8">
		<div class="rounded border border-neutral-800 bg-neutral-900 p-4">
			<div class="mb-4 flex items-center justify-between border-b border-neutral-800 pb-2">
				<div>
					<h2 class="text-2xl font-bold">API Keys</h2>
				</div>
				<button class="preset-filled-500 btn font-semibold" onclick={openAddDialog}>
					+ Create Key
				</button>
			</div>
			{#if apiKeysQuery.isPending}
				<p class="p-4 text-neutral-500">Loading keys...</p>
			{:else if apiKeysQuery.isError}
				<p class="p-4 text-red-500">
					Error loading keys: {apiKeysQuery.error || 'Unknown error'}
				</p>
			{:else if apiKeysQuery.isSuccess && apiKeysQuery.data.length > 0}
				<div class="grid grid-cols-12 gap-4 px-2 text-xs font-bold text-neutral-400">
					<div class="col-span-3">NAME</div>
					<div class="col-span-3">DNS RECORD MANAGED</div>
					<div class="col-span-2">CREATED DATE</div>
					<div class="col-span-2">LAST USED</div>
					<div class="col-span-2 pr-2 text-right">ACTIONS</div>
				</div>
				{#each apiKeysQuery.data as apiKey}
					<div
						class="mb-2 grid grid-cols-12 rounded border border-neutral-800 bg-neutral-900 px-4 py-3"
					>
						<!-- API Key Field -->
						<div class="col-span-3 truncate break-all">
							<span class="font-mono text-sm">
								{apiKey.name}
							</span>
						</div>

						<div class="col-span-3 truncate break-all">
							<span class="font-mono text-sm">
								{apiKey.record_name}
							</span>
						</div>

						<div class="col-span-2 truncate break-all">
							<span class="font-mono text-sm">
								{apiKey.created_on}
							</span>
						</div>

						<div class="col-span-2 truncate break-all">
							<span class="font-mono text-sm">
								{#if apiKey.last_used}
									{apiKey.last_used}
								{:else}
									–
								{/if}
							</span>
						</div>

						<!-- Actions -->
						<div class="col-span-2 flex justify-end">
							<button
								onclick={() => openDeleteDialog(apiKey.id)}
								class="btn preset-filled-error-500 btn-sm"
							>
								Delete
							</button>
						</div>
					</div>
				{/each}
			{:else}
				<div class="w-full rounded p-4">
					<p>No API keys yet. Click "Add Key" to get started</p>
				</div>
			{/if}
		</div>
	</div>
</div>

<!-- Add Access Key Dialog -->
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
					<Dialog.Title class="text-lg font-bold">Add API Key</Dialog.Title>
					<Dialog.CloseTrigger
						class="btn-icon hover:preset-tonal"
						onclick={() => {
							addDialogOpen = false;
						}}
					>
						<XIcon class="size-4" />
					</Dialog.CloseTrigger>
				</header>

				<Dialog.Description class="sr-only">
					Add a new API key to use on your Drago Daemon
				</Dialog.Description>

				<!-- Operation feedback -->
				{#if operationMessage}
					<div
						class={`rounded p-2 ${
							operationSuccess ? 'bg-green-800 text-green-200' : 'bg-red-800 text-red-200'
						}`}
					>
						{operationMessage}
					</div>
				{/if}

				{#if !apiKeyReturned}
					<form onsubmit={submitApiKeyData} class="space-y-4">
						<!-- Key Name -->
						<div>
							<label for="key-name" class="mb-1 block text-sm font-medium text-neutral-300">
								Name
							</label>
							<input
								id="key-name"
								name="name"
								type="text"
								placeholder="Cool API Key Name"
								bind:value={addKeyData.name}
								max="255"
								required
								class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-neutral-100 shadow-sm sm:text-sm"
							/>
						</div>

						<!-- Record Scope -->
						<div>
							<label for="key-scope" class="mb-1 block text-sm font-medium text-neutral-300">
								Record Scope
							</label>
							<select
								id="key-scope"
								class={`w-full rounded border border-neutral-600 bg-neutral-800 p-2 ${
									addKeyData.scope ? 'text-neutral-100' : 'text-neutral-500'
								}`}
								bind:value={addKeyData.scope}
								onchange={handleRecordSelect}
								required
							>
								<option value="" disabled selected>
									Select a DNS record to dynamically update
								</option>
								{#each recordScopes as [zone, records]}
									<optgroup label={zone.name}>
										{#each records as record}
											<option value={record.id} data-zone-id={zone.id} data-record-id={record.id}>
												{record.name} ({record.content})
											</option>
										{/each}
									</optgroup>
								{/each}
							</select>
						</div>

						<!-- Footer -->
						<footer class="flex justify-end gap-2 pt-4">
							<Dialog.CloseTrigger
								class="btn preset-tonal"
								onclick={() => {
									addDialogOpen = false;
								}}
							>
								Cancel
							</Dialog.CloseTrigger>
							<button type="submit" class="btn preset-filled-primary-500 font-semibold">
								+ Create Key
							</button>
						</footer>
					</form>
				{:else}
					<p>Save your API key somewhere safe. It will not be shown again.</p>
					<div
						class="flex items-center gap-2 rounded border border-neutral-800 bg-neutral-900 px-3 py-2"
					>
						<span class="truncate font-mono text-sm text-neutral-100">
							{apiKeyReturned}
						</span>

						<button
							type="button"
							onclick={copyKey}
							class="ml-auto inline-flex items-center gap-1 rounded border border-neutral-700 bg-neutral-800 px-2 py-1 text-xs text-neutral-100 hover:bg-neutral-700"
						>
							{#if copied}
								<Check class="h-3 w-3 text-emerald-400" />
								<span>Copied</span>
							{:else}
								<Copy class="h-3 w-3" />
								<span>Copy</span>
							{/if}
						</button>
					</div>
					<footer class="flex justify-end gap-2 pt-4">
						<Dialog.CloseTrigger
							class="btn preset-tonal"
							onclick={() => {
								addDialogOpen = false;
							}}
						>
							Close
						</Dialog.CloseTrigger>
					</footer>
				{/if}
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>

<!-- Delete Access Key Dialog -->
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
					<Dialog.Title class="text-lg font-bold text-red-500">Delete API Key</Dialog.Title>
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
						class={`rounded p-2 ${
							operationSuccess ? 'bg-green-800 text-green-200' : 'bg-red-800 text-red-200'
						}`}
					>
						{operationMessage}
					</div>
				{/if}

				{#if deleteKeyMutation.isSuccess}
					<div class="flex flex-col items-center justify-center py-4">
						<div class="mb-4 rounded-full bg-green-800 p-3">
							<Check class="size-4 text-green-200" />
						</div>

						<p class="text-center text-green-200">API key has been successfully deleted.</p>
					</div>
				{:else}
					<Dialog.Description>
						Are you sure you want to delete this API key? This action cannot be undone.
					</Dialog.Description>
				{/if}

				<footer class="flex justify-end gap-2">
					{#if deleteKeyMutation.isSuccess}
						<Dialog.CloseTrigger
							class="btn preset-filled-primary-500"
							onclick={() => {
								deleteDialogOpen = false;
							}}
						>
							Close
						</Dialog.CloseTrigger>
					{:else}
						<Dialog.CloseTrigger
							class="btn preset-tonal"
							onclick={() => {
								deleteDialogOpen = false;
							}}
						>
							Cancel
						</Dialog.CloseTrigger>
						<button
							type="button"
							class="btn preset-filled-error-500"
							onclick={() =>
								deleteKeyMutation.mutate({
									key_id: currentApiKeyId
								})}
						>
							Delete
						</button>
					{/if}
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
