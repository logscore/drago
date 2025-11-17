<script lang="ts">
	import { goto } from '$app/navigation';
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';

	let value = $state('domains');

	async function handleSignOut() {
		await authClient.signOut();
		goto('/');
	}
</script>

<div class="min-h-screen bg-slate-50" style="margin: 1rem;">
	<!-- Main Content -->
	<main class="container mx-auto px-6 py-8">
		<div class="mx-auto max-w-5xl">
			<Tabs {value} onValueChange={(details) => (value = details.value)}>
				<Tabs.List
					class="mb-8 inline-flex gap-1 rounded-lg bg-slate-200/50
          p-1"
				>
					<Tabs.Trigger
						value="domains"
						class="rounded-md px-4 py-2 text-sm font-medium
            text-slate-700 transition-all hover:text-slate-900
            data-[state=active]:bg-white data-[state=active]:text-slate-900
            data-[state=active]:shadow-sm"
					>
						Domains
					</Tabs.Trigger>
					<Tabs.Trigger
						value="api keys"
						class="rounded-md px-4 py-2 text-sm font-medium
            text-slate-700 transition-all hover:text-slate-900
            data-[state=active]:bg-white data-[state=active]:text-slate-900
            data-[state=active]:shadow-sm"
					>
						API Keys
					</Tabs.Trigger>
					<Tabs.Trigger
						value="settings"
						class="rounded-md px-4 py-2 text-sm font-medium
            text-slate-700 transition-all hover:text-slate-900
            data-[state=active]:bg-white data-[state=active]:text-slate-900
            data-[state=active]:shadow-sm"
					>
						Settings
					</Tabs.Trigger>
					<Tabs.Indicator />
				</Tabs.List>

				<Tabs.Content value="domains">
					<div class="space-y-4">
						<div class="flex items-center justify-between">
							<div>
								<h2 class="text-2xl font-semibold text-slate-900">Domains</h2>
								<p class="mt-1 text-sm text-slate-600">
									Manage and monitor your registered domains
								</p>
							</div>
							<button
								class="rounded-lg bg-blue-600 px-4 py-2 text-sm
                font-medium text-white transition-colors
                hover:bg-blue-700"
							>
								Add Domain
							</button>
						</div>

						<div class="rounded-xl bg-white p-6 shadow-sm">
							<p class="text-center text-sm text-slate-500">
								No domains yet. Add your first domain to get started.
							</p>
						</div>
					</div>
				</Tabs.Content>

				<Tabs.Content value="api keys">
					<div class="space-y-4">
						<div class="flex items-center justify-between">
							<div>
								<h2 class="text-2xl font-semibold text-slate-900">API Keys</h2>
								<p class="mt-1 text-sm text-slate-600">Manage your API keys and access tokens</p>
							</div>
							<button
								class="rounded-lg bg-blue-600 px-4 py-2 text-sm
                font-medium text-white transition-colors
                hover:bg-blue-700"
							>
								Create Key
							</button>
						</div>

						<div class="rounded-xl bg-white p-6 shadow-sm">
							<p class="text-center text-sm text-slate-500">
								No API keys created yet. Generate your first key to access the API.
							</p>
						</div>
					</div>
				</Tabs.Content>

				<Tabs.Content value="settings">
					<div class="space-y-4">
						<h2 class="text-2xl font-semibold text-slate-900">Settings</h2>

						<div class="space-y-3">
							<div class="rounded-xl bg-white p-6 shadow-sm">
								<h3 class="mb-4 font-medium text-slate-900">Account</h3>
								<div class="rounded-lg bg-slate-50 p-4">
									<div class="flex items-center justify-between">
										<div>
											<div class="font-medium text-slate-200">Sign out</div>
										</div>
										<button onclick={handleSignOut} class="btn preset-filled-error-500">
											Sign Out
										</button>
									</div>
								</div>
							</div>
						</div>
					</div>
				</Tabs.Content>
			</Tabs>
		</div>
	</main>
</div>
