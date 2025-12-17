<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { authClient } from '$lib/auth/authClient';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';

	let value = $state('signin');
	let email = $state('');
	let password = $state('');
	let name = $state('');
	let loading = $state(false);
	let error = $state('');
	let redirectUrl = $state('/dashboard');

	$effect(() => {
		const redirect_search_param = page.url.searchParams.get('redirect_url');

		console.log(redirect_search_param);

		if (redirect_search_param) {
			redirectUrl = redirect_search_param;
		}
	});

	async function handleSubmit() {
		loading = true;
		error = '';

		try {
			if (value === 'signin') {
				await authClient.signIn.email(
					{
						email,
						password,
						rememberMe: true
					},
					{
						onSuccess() {
							goto(`${redirectUrl}`);
						},
						onError(ctx) {
							error = ctx.error.message;
						}
					}
				);
			} else {
				await authClient.signUp.email(
					{
						email: email,
						name: name,
						password: password
					},
					{
						onSuccess() {
							goto(`${redirectUrl}`);
						},
						onError(ctx) {
							error = ctx.error.message;
						}
					}
				);
			}
		} catch (err) {
			error = 'Unexpected error, try again.';
		} finally {
			loading = false;
		}
	}
</script>

<div class="mx-auto mt-16 max-w-90 card p-6">
	<Tabs {value} onValueChange={(details) => (value = details.value)}>
		<Tabs.List>
			<Tabs.Trigger class="flex-1 hover:bg-neutral-800 hover:text-neutral-100" value="signin"
				>Sign In</Tabs.Trigger
			>
			<Tabs.Trigger class="flex-1 hover:bg-neutral-800 hover:text-neutral-100" value="signup"
				>Sign Up</Tabs.Trigger
			>
			<Tabs.Indicator />
		</Tabs.List>

		<Tabs.Content value="signin">
			<form class="mt-4 space-y-4" onsubmit={handleSubmit}>
				<div class="form-control">
					<label class="label"
						>Email
						<input class="input-bordered input" type="email" bind:value={email} required /></label
					>
				</div>

				<div class="form-control">
					<label class="label"
						>Password
						<input
							class="input-bordered input"
							type="password"
							bind:value={password}
							required
						/></label
					>
				</div>

				{#if error}
					<div class="alert alert-error">
						<span>{error}</span>
					</div>
				{/if}

				<div class="form-control mt-6">
					<button class="btn preset-filled" type="submit" disabled={loading}>
						{#if loading}
							Signing in...
						{:else}
							Sign In
						{/if}
					</button>
				</div>
			</form>
		</Tabs.Content>

		<Tabs.Content value="signup">
			<form class="mt-4 space-y-4" onsubmit={handleSubmit}>
				<div class="form-control">
					<label class="label"
						>Name
						<input class="input-bordered input" type="text" bind:value={name} required /></label
					>
				</div>

				<div class="form-control">
					<label class="label"
						>Email
						<input class="input-bordered input" type="email" bind:value={email} required /></label
					>
				</div>

				<div class="form-control">
					<label class="label"
						>Password
						<input
							class="input-bordered input"
							type="password"
							bind:value={password}
							required
						/></label
					>
				</div>

				{#if error}
					<div class="alert alert-error">
						<span>{error}</span>
					</div>
				{/if}

				<div class="form-control mt-6">
					<button class="btn preset-filled" type="submit" disabled={loading}>
						{#if loading}
							Signing up...
						{:else}
							Sign Up
						{/if}
					</button>
				</div>
			</form>
		</Tabs.Content>
	</Tabs>
</div>
