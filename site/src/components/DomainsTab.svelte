<script lang="ts">
	import type { ZoneRecordData } from '$lib/types';
	import { XIcon } from '@lucide/svelte';
	import { Dialog, Portal } from '@skeletonlabs/skeleton-svelte';
	import type { CreateQueryResult } from '@tanstack/svelte-query';

	interface Props {
		jwtData: string;
		recordsQuery: CreateQueryResult<ZoneRecordData>;
		tab: string;
		missingToken: boolean;
	}

	let { jwtData, tab = $bindable(), recordsQuery, missingToken }: Props = $props();

	// Form state
	let ttl = $state(1);
	let content = $state('');
	let recordType = $state('A');
	let domainName = $state('');
	let proxied = $state(false);

	// Dialog state
	let addDialogOpen = $state(false);
	let deleteDialogOpen = $state(false);

	// Current zone and record for operations
	let currentZone = $state<{ id: string; name: string } | null>(null);
	let currentRecordId = $state<string>('');

	// Operation feedback
	let operationMessage = $state<string>('');
	let operationSuccess = $state<boolean>(false);

	async function handleAddDomain(zoneId: string | undefined, zoneName: string | undefined) {
		// Form validation
		if (!domainName || !content) {
			operationMessage = 'Please fill in all required fields';
			operationSuccess = false;
			return;
		}

		try {
			const response = await fetch(`${env.PUBLIC_BACKEND_URL}/record`, {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
					Authorization: `Bearer ${jwtData}`
				},
				body: JSON.stringify({
					zone_id: zoneId,
					zone_name: zoneName,
					name: domainName,
					record_type: recordType,
					content: content,
					ttl: ttl,
					proxied: proxied
				})
			});

			if (!response.ok) {
				throw new Error('Network response was not ok');
			}

			// Handle success
			operationMessage = 'Record added successfully';
			operationSuccess = true;

			// Reset form
			domainName = '';
			content = '';
			recordType = 'A';
			ttl = 1;
			proxied = false;

			// Close dialog
			addDialogOpen = false;

			// Refresh records
			recordsQuery.refetch();
		} catch (error) {
			console.error('Error adding record:', error);
			operationMessage = `Error adding record: ${error}`;
			operationSuccess = false;
		}
	}

	async function handleDelete(zoneId: string, recordId: string) {
		try {
			const response = await fetch(
				`${env.PUBLIC_BACKEND_URL}/record?record_id=${recordId}&zone_id=${zoneId}`,
				{
					headers: {
						Authorization: `Bearer ${jwtData}`
					},
					method: 'DELETE'
				}
			);

			if (!response.ok) {
				throw new Error('Network response was not ok');
			}

			// Handle success
			operationMessage = 'Record deleted successfully';
			operationSuccess = true;

			// Close dialog
			deleteDialogOpen = false;

			// Refresh records
			recordsQuery.refetch();
		} catch (error) {
			console.error('Error deleting record:', error);
			operationMessage = `Error deleting record: ${error}`;
			operationSuccess = false;
		}
	}

	function openAddDialog(zone: { id: string; name: string }) {
		currentZone = zone;
		addDialogOpen = true;

		// Reset form and message
		domainName = '';
		content = '';
		recordType = 'A';
		ttl = 1;
		proxied = false;
		operationMessage = '';
	}

	function openDeleteDialog(zoneId: string, recordId: string) {
		currentZone = { id: zoneId, name: '' };
		currentRecordId = recordId;
		deleteDialogOpen = true;
		operationMessage = '';
	}

	function handleGoToSettings() {
		tab = 'settings';
		console.log(tab);
	}

	const animation =
		'transition transition-discrete opacity-0 translate-y-[100px] starting:data-[state=open]:opacity-0 starting:data-[state=open]:translate-y-[100px] data-[state=open]:opacity-100 data-[state=open]:translate-y-0';
</script>

<h1 class="mb-2 text-lg font-semibold">Domains</h1>
<p class="mb-4 text-sm text-neutral-400">Manage and monitor your registered domains.</p>

{#if recordsQuery.isPending}
	<p class="p-4 text-neutral-500">Loading records...</p>
{:else if missingToken}
	<div class="rounded border border-amber-500/40 bg-amber-500/5 p-4 text-amber-100">
		<p class="mb-2 font-semibold">Missing DNS access token!</p>
		<p class="mb-4 text-sm text-amber-200/80">
			Head over to the Settings tab and add a DNS access token so Drago can sync your zones.
		</p>
		<button
			type="button"
			class="preset-filled-500 btn font-semibold"
			onclick={() => handleGoToSettings()}
		>
			Go to Settings
		</button>
	</div>
{:else if recordsQuery.isError}
	<p class="p-4 text-red-500">
		Error loading records: {recordsQuery.error || 'Unknown error'}
	</p>
{:else if recordsQuery.isSuccess}
	<div class="space-y-8">
		{#each recordsQuery.data as [zone, records]}
			<div class="rounded border border-neutral-800 bg-neutral-900 p-4">
				<div class="mb-4 flex items-center justify-between border-b border-neutral-800 pb-2">
					<div>
						<h2 class="text-2xl font-bold">{zone.name}</h2>
					</div>
					<button onclick={() => openAddDialog(zone)} class="preset-filled-500 btn font-semibold">
						+ Add Record
					</button>
				</div>

				{#if records.length > 0}
					<div class="flex flex-col gap-2">
						<!-- Header -->
						<div class="grid grid-cols-12 gap-4 px-2 text-xs font-bold text-neutral-400">
							<div class="col-span-3">NAME</div>
							<div class="col-span-4">CONTENT</div>
							<div class="col-span-1">TYPE</div>
							<div class="col-span-1">TTL</div>
							<div class="col-span-1">PROXIED</div>
							<div class="col-span-2 pr-2 text-right">ACTIONS</div>
						</div>

						<!-- Rows -->
						{#each records as record}
							<div class="grid grid-cols-12 items-center gap-4 rounded bg-neutral-950/50 px-4 py-3">
								<!-- Name Field -->
								<div class="col-span-3 truncate break-all">
									<span class="font-mono text-sm">
										{#if record.name.split('.').length > 2}
											{record.name.split('.')[0]}
										{:else}
											{record.name}
										{/if}
									</span>
								</div>

								<!-- Content Field -->
								<div class="col-span-4 truncate">
									<span class="px-2 py-1 font-mono text-sm text-neutral-400">{record.content}</span>
								</div>

								<!-- Type Field -->
								<div class="col-span-1 truncate">
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

								<!-- Proxied Field -->
								<div class="col-span-1 truncate">
									<span class="font-mono text-sm text-neutral-400">
										{#if record.proxied === true}
											Proxied
										{:else}
											DNS
										{/if}</span
									>
								</div>

								<!-- Actions -->
								<div class="col-span-2 flex justify-end gap-2">
									<button
										onclick={() => openDeleteDialog(zone.id, record.id)}
										class="btn preset-filled-error-500 btn-sm"
									>
										Delete
									</button>
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
{/if}

<!-- Add Record Dialog -->
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
					<Dialog.Title class="text-lg font-bold">Add DNS Record</Dialog.Title>
					<Dialog.CloseTrigger
						class="btn-icon hover:preset-tonal"
						onclick={() => {
							addDialogOpen = false;
						}}
					>
						<XIcon class="size-4" />
					</Dialog.CloseTrigger>
				</header>

				<Dialog.Description class="sr-only">Form to add a new DNS record</Dialog.Description>

				<!-- Operation feedback -->
				{#if operationMessage}
					<div
						class={`rounded p-2 ${operationSuccess ? 'bg-green-800 text-green-200' : 'bg-red-800 text-red-200'}`}
					>
						{operationMessage}
					</div>
				{/if}

				<form
					onsubmit={() => handleAddDomain(currentZone?.id, currentZone?.name)}
					class="space-y-4"
				>
					<!-- Name -->
					<div>
						<label for="record-name" class="mb-1 block text-sm font-medium text-neutral-300">
							Name
						</label>
						<input
							id="record-name"
							name="name"
							type="text"
							placeholder="@, www, or subdomain"
							bind:value={domainName}
							required
							class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-neutral-100 shadow-sm sm:text-sm"
						/>
					</div>

					<!-- Content -->
					<div>
						<label for="record-content" class="mb-1 block text-sm font-medium text-neutral-300">
							Content
						</label>
						<input
							id="record-content"
							name="content"
							type="text"
							placeholder="IPv4 address or target"
							bind:value={content}
							required
							class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-neutral-100 shadow-sm sm:text-sm"
						/>
					</div>

					<!-- TTL, Record Type, and Proxy Row -->
					<div class="flex gap-4">
						<!-- Record Type -->
						<div class="flex-1">
							<label for="record-type" class="mb-1 block text-sm font-medium text-neutral-300">
								Type
							</label>
							<select
								id="record-type"
								name="type"
								bind:value={recordType}
								class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-neutral-100 shadow-sm sm:text-sm"
							>
								<option value="A">A</option>
								<option value="CNAME">CNAME</option>
								<option value="TXT">TXT</option>
								<option value="MX">MX</option>
								<option value="AAAA">AAAA</option>
							</select>
						</div>

						<!-- TTL -->
						<div class="flex-1">
							<label for="record-ttl" class="mb-1 block text-sm font-medium text-neutral-300">
								TTL
							</label>
							<select
								id="record-ttl"
								name="ttl"
								bind:value={ttl}
								class="block w-full rounded-md border-neutral-600 bg-neutral-800 px-3 py-2 text-neutral-100 shadow-sm sm:text-sm"
							>
								<option value={1}>Auto</option>
								<option value={60}>1 min</option>
								<option value={300}>5 min</option>
								<option value={3600}>1 hour</option>
								<option value={86400}>1 day</option>
							</select>
						</div>

						<!-- Cloudflare Proxied -->
						<div class="flex items-center gap-2 pt-6">
							<input
								id="record-proxied"
								name="proxied"
								type="checkbox"
								bind:checked={proxied}
								class="text-white-100 h-4 w-4 rounded border-neutral-600 bg-neutral-800 focus:ring-neutral-600"
							/>
							<label for="record-proxied" class="text-sm font-medium text-neutral-300">
								Proxy
							</label>
						</div>
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
							+ Add Record
						</button>
					</footer>
				</form>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>

<!-- Delete Record Dialog -->
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
					<Dialog.Title class="text-lg font-bold text-red-500">Delete Record</Dialog.Title>
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
					Are you sure you want to delete this DNS record? This action cannot be undone.
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
						onclick={() => handleDelete(currentZone?.id || '', currentRecordId)}
					>
						Delete
					</button>
				</footer>
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog>
