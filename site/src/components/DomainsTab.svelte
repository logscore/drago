<script lang="ts">
	import type { ZoneRecordData, AddRecordData, Zone } from '$lib/types';
	import { createQuery } from '@tanstack/svelte-query';

	interface Props {
		userId: string | undefined;
	}

	let { userId }: Props = $props();
	let deleteConfirmation = $state(false);

	const recordsQuery = createQuery<ZoneRecordData>(() => ({
		queryKey: ['records', userId],
		queryFn: async () => {
			3;
			if (!userId) {
				throw new Error('User ID is required');
			}
			const response = await fetch(`http://127.0.0.1:8080/records?user_id=${userId}`);
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return response.json();
		},
		enabled: !!userId
	}));

	async function handleAddRecord(recordData: AddRecordData) {
		// TODO: Implement API call to add record
		console.log('Adding', recordData);
	}

	async function onDelete(recordId: string) {
		// TODO: Implement API call to delete record
		console.log('Deleting', recordId);
	}

	// Dialog state
	let isDialogOpen = $state(false);
	let selectedZone: Zone | null = $state(null);

	// Form state
	let recordName = $state('');
	let recordContent = $state('');
	let recordType = $state('A');
	let recordTtl = $state(3600);
	let recordProxied = $state(0);

	const recordTypes = ['A', 'AAAA', 'CNAME', 'MX', 'TXT', 'NS', 'SRV', 'CAA'];

	function openDialog(zone: Zone) {
		selectedZone = zone;
		isDialogOpen = true;
		// Reset form
		recordName = '';
		recordContent = '';
		recordType = 'A';
		recordTtl = 3600;
		recordProxied = 0;
	}
</script>

<h1 class="mb-2 text-lg font-semibold">Domains</h1>
<p class="mb-4 text-sm text-neutral-400">Manage and monitor your registered domains.</p>

{#if recordsQuery.isPending}
	<p class="p-4 text-neutral-500">Loading records...</p>
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
						<h2 class="text-xl font-bold">{zone.name}</h2>
						<span class="text-xs text-neutral-500">ID: {zone.id}</span>
					</div>
					<button class="preset-filled-500 btn font-semibold" onclick={() => openDialog(zone)}>
						+ Add Record
					</button>
				</div>

				{#if records.length > 0}
					<div class="flex flex-col gap-2">
						<!-- Header -->
						<div class="grid grid-cols-12 gap-4 px-2 text-xs font-bold text-neutral-400">
							<div class="col-span-3">NAME</div>
							<div class="col-span-3">CONTENT</div>
							<div class="col-span-1">TYPE</div>
							<div class="col-span-1">TTL</div>
							<div class="col-span-1">PROXIED</div>
							<div class="col-span-3 text-right">ACTIONS</div>
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
								<div class="col-span-3 truncate">
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
										{#if record.proxied === 1}
											Proxied
										{:else}
											DNS
										{/if}</span
									>
								</div>

								<!-- Actions -->
								<div class="col-span-3 flex justify-end gap-2">
									<button
										onclick={(e) => {
											e.preventDefault();
											onDelete(record.id);
										}}
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

	{#if recordsQuery && !recordsQuery.data}
		<div class="w-full rounded p-4">
			<p>No domains yet.</p>
			<button class="mt-2 btn preset-filled">Add Domain</button>
		</div>
	{/if}
{/if}
