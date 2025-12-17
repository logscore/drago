<script lang="ts">
	import { authClient } from '$lib/auth/authClient';
	import { goto } from '$app/navigation';
	import OtpInput from '../../../components/OtpInput.svelte';

	let code = $state('');
	let error = $state('');
	let loading = $state(false);

	const handleSubmit = async () => {
		loading = true;
		error = '';

		try {
			const formatted = code.replace(/-/g, '').toUpperCase();

			if (formatted.length !== 8) {
				error = 'Invalid device code.';
				loading = false;
				return;
			}

			const res = await authClient.device({
				query: { user_code: formatted }
			});

			if (!res.data) throw new Error();
			await goto(`/device/approve?user_code=${formatted}`);
		} catch {
			error = 'Invalid or expired device code.';
		}

		loading = false;
	};
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-950 p-6">
	<form
		onsubmit={handleSubmit}
		class="w-lg space-y-6 rounded-2xl border border-neutral-800 bg-neutral-900 p-8 shadow-xl"
	>
		<div class="space-y-2 text-center">
			<h1 class="text-xl font-semibold text-neutral-100">Device Authorization</h1>
			<p class="text-sm text-neutral-400">Enter the 8-character code from your CLI</p>
		</div>

		<div class="py-4">
			<OtpInput bind:value={code} disabled={loading} />
		</div>

		<button
			type="submit"
			disabled={loading || code.length !== 8}
			class="btn-primary btn w-full py-3 text-base disabled:opacity-50"
		>
			{loading ? 'Verifying…' : 'Continue'}
		</button>

		{#if error}
			<div class="rounded-md border border-red-900/50 bg-red-950/20 px-4 py-3">
				<p class="text-sm text-red-400">{error}</p>
			</div>
		{/if}
	</form>
</div>
