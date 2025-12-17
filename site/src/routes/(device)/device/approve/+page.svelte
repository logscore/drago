<script lang="ts">
	import { authClient } from '$lib/auth/authClient';
	import { goto } from '$app/navigation';
	import type { PageData } from './$types';
	import { page } from '$app/state';

	const userCode = page.url.searchParams.get('user_code');
	console.log(userCode);
	let processing = $state(false);
	let error = $state('');

	const handleApprove = async () => {
		processing = true;
		error = '';

		try {
			await authClient.device.approve({ userCode });
			await goto('/dashboard');
		} catch (err) {
			console.error(err);
			error = 'Failed to approve device. Please try again.';
		}

		processing = false;
	};

	const handleDeny = async () => {
		processing = true;
		error = '';

		try {
			await authClient.device.deny({ userCode });
			await goto('/dashboard');
		} catch (err) {
			console.error(err);
			error = 'Failed to deny device. Please try again.';
		}

		processing = false;
	};
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-950 p-6">
	<div class="w-full max-w-xl rounded-2xl border border-neutral-800 bg-neutral-900 shadow-xl">
		<div class="space-y-6 p-8">
			<!-- Header -->
			<header class="space-y-2 text-center">
				<h1 class="text-2xl font-semibold text-neutral-100">Device Authorization Request</h1>
				<p class="text-sm text-neutral-400">
					A DragoDNS CLI application is requesting access to your account.
				</p>
			</header>

			<!-- Device Code -->
			<div class="space-y-2 rounded-xl bg-neutral-800 p-6 text-center">
				<div class="text-xs font-medium tracking-wide text-neutral-400 uppercase">Device Code</div>
				<div
					class="inline-block rounded-lg border border-neutral-700 bg-neutral-900 px-4 py-2 font-mono text-2xl font-bold tracking-widest text-neutral-100"
				>
					{userCode}
				</div>
			</div>

			<!-- Actions -->
			<div class="grid grid-cols-2 gap-4">
				<button
					onclick={handleApprove}
					disabled={processing}
					class="btn-primary btn w-full disabled:opacity-60"
				>
					{processing ? 'Processing…' : 'Approve'}
				</button>

				<button
					onclick={handleDeny}
					disabled={processing}
					class="btn-outline btn-neutral btn w-full disabled:opacity-60"
				>
					{processing ? 'Processing…' : 'Deny'}
				</button>
			</div>

			<!-- Error -->
			{#if error}
				<div class="rounded-lg border border-red-900 bg-red-950 px-4 py-3 text-sm text-red-300">
					{error}
				</div>
			{/if}

			<!-- Info -->
			<section class="space-y-4 border-t border-neutral-800 pt-6">
				<h3 class="text-sm font-semibold text-neutral-200">What this means</h3>

				<ul class="list-disc space-y-2 pl-5 text-sm text-neutral-400">
					<li>
						<strong class="text-neutral-300">Approving</strong> allows the CLI application to manage DNS
						records on your behalf
					</li>
					<li>
						<strong class="text-neutral-300">Denying</strong> blocks this device from accessing your account
					</li>
					<li>You can revoke access at any time from account settings</li>
				</ul>

				<div
					class="rounded-lg border border-neutral-700 bg-neutral-800 px-4 py-3 text-sm text-neutral-300"
				>
					<strong>Security note:</strong> Only approve devices you recognize and trust.
				</div>
			</section>
		</div>
	</div>
</div>
