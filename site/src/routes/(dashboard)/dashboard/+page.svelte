<!-- TODO: separate this page into multiple svelte components -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';
	import { createQuery } from '@tanstack/svelte-query';

	const session = authClient.useSession();

	let tab_value = $state('domains');

	// Track editing state by Record ID
	let editingState = $state();

	function toggleAddRecord(id: string) {
		editingState = true;
	}

	async function handleAddRecord(recordData: AddRecordData) {
		// TODO: Implement API call to add record
		console.log('Adding', recordData);
	}

	async function handleDelete(id: string) {
		// TODO: Implement API call to delete
		console.log('Deleting', id);
	}

	async function handleSignOut() {
		await authClient.signOut();
		goto('/');
	}

	const fetchRecords = async (): Promise<ZoneRecordData> => {
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

				{#if recordsQuery.isLoading}
					<p class="p-4 text-neutral-500">Loading records...</p>
				{:else if recordsQuery.error}
					<p class="p-4 text-red-500">Error loading records.</p>
				{:else if recordsQuery.data}
					<div class="space-y-8">
						{#each recordsQuery.data as [zone, records]}
							<div class="rounded border border-neutral-800 bg-neutral-900 p-4">
								<div
									class="mb-4 flex items-center justify-between border-b border-neutral-800 pb-2"
								>
									<div>
										<h2 class="text-xl font-bold">{zone.name}</h2>
										<span class="text-xs text-neutral-500">ID: {zone.id}</span>
									</div>
									<button class="preset-filled-500 btn font-semibold" onclick={handleAddRecord}>
										+ Add Record
									</button>
								</div>

								{#if records.length > 0}
									<div class="flex flex-col gap-2">
										<!-- Header -->
										<div class="grid grid-cols-12 gap-4 px-2 text-xs font-bold text-neutral-400">
											<div class="col-span-3">NAME</div>
											<div class="col-span-3">CONTENT</div>
											<div class="col-span-2">TYPE</div>
											<div class="col-span-1">TTL</div>
											<div class="col-span-1">PROXIED</div>
											<div class="col-span-2 text-right">ACTIONS</div>
										</div>

										<!-- Rows -->
										{#each records as record}
											{@const isEditing = editingState[record.id]}
											<div
												class="grid grid-cols-12 items-center gap-4 rounded bg-neutral-950/50 px-4 py-3"
											>
												<!-- Name Field -->
												<div class="col-span-3 break-all">
													{#if isEditing}
														<input
															class="input px-2 py-1"
															bind:value={record.name}
															placeholder="Name"
														/>
													{:else}
														<span class="font-mono text-sm">
															{#if record.name.split('.').length > 2}
																{record.name.split('.')[0]}
															{:else}
																{record.name}
															{/if}
														</span>
													{/if}
												</div>

												<!-- Content Field -->
												<div class="col-span-3 truncate">
													{#if isEditing}
														<input
															class="input px-2 py-1"
															bind:value={record.content}
															placeholder="Content"
														/>
													{:else}
														<span class="px-2 py-1 font-mono text-sm text-neutral-400"
															>{record.content}</span
														>
													{/if}
												</div>

												<!-- Type Field -->
												<div class="col-span-2 truncate">
													<span class="font-mono text-sm text-neutral-400">{record.type}</span>
												</div>

												<!-- TTL Field -->
												<div class="col-span-1 truncate">
													<span class="font-mono text-sm text-neutral-400">
														{#if record.ttl === 1}
															Auto
														{:else}
															{record.ttl / 60} min
														{/if}</span
													>
												</div>

												<!-- TTL Field -->
												<div class="col-span-1 truncate">
													<span class="font-mono text-sm text-neutral-400">
														{#if record.proxied === 1}
															Proxied
														{:else}
															DNS
														{/if}</span
													>
												</div>

												<!-- Actions -->
												<div class="col-span-2 flex justify-end gap-2">
													<!-- {#if isEditing} -->
													<button
														onclick={() => handleDelete(record.id)}
														class="btn preset-filled-error-500 btn-sm"
													>
														Delete
													</button>
													<!-- <button
															onclick={() => handleSave(record)}
															class="btn preset-filled-success-500 btn-sm"
														>
															Save
														</button>
													{:else}
														<button
															onclick={() => toggleEdit(record.id)}
															class="btn preset-tonal btn-sm"
														>
															Edit
														</button>
													{/if} -->
												</div>
											</div>
										{/each}
									</div>
								{:else}
									<p class="text-sm text-neutral-500 italic">No records found for this zone.</p>
								{/if}
							</div>
						{/each}
					</div>

					{#if recordsQuery.data.length === 0}
						<div class="w-full rounded p-4">
							<p>No domains yet.</p>
							<button class="mt-2 btn preset-filled">Add Domain</button>
						</div>
					{/if}
				{/if}
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
