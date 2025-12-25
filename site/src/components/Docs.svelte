<script lang="ts">
	const installCmd = 'curl -sSL https://www.dragodns.com/install.sh | sh';
</script>

<div class="mx-auto min-h-screen max-w-3xl space-y-12 p-8 text-neutral-200">
	<header class="space-y-2 border-b border-neutral-800 pb-8">
		<h1 class="text-4xl font-bold tracking-tight text-neutral-100">Quickstart</h1>
		<p class="text-lg text-neutral-400">Install and configure the Drago daemon for dynamic DNS.</p>
		<p class="border-l-4 border-amber-500 p-2 text-lg text-amber-500">
			NOTE: Drago currently only supports Cloudflare as a DNS provider. To request another provider,
			message <span
				><a href="https://x.com/logscore" class="text-blue-400 underline">@logscore</a></span
			> on X
		</p>
	</header>

	<div class="space-y-12">
		<!-- Step 1 -->
		<section class="space-y-4">
			<div class="flex items-center gap-3">
				<span
					class="flex h-8 w-8 items-center justify-center rounded-full bg-neutral-100 text-sm font-bold text-neutral-950"
					>1</span
				>
				<h2 class="text-xl font-bold text-neutral-100">Installation</h2>
			</div>
			<div
				class="overflow-x-auto rounded-lg border border-neutral-800 bg-neutral-900 p-4 font-mono text-sm"
			>
				<code class="text-neutral-300">{installCmd}</code>
			</div>
		</section>

		<!-- Step 2 -->
		<section class="space-y-4">
			<div class="flex items-center gap-3">
				<span
					class="flex h-8 w-8 items-center justify-center rounded-full bg-neutral-100 text-sm font-bold text-neutral-950"
					>2</span
				>
				<h2 class="text-xl font-bold text-neutral-100">Initialize</h2>
			</div>
			<p class="text-neutral-400">
				Sign in, add your Cloudflare API key, and configure your first record.
			</p>
			<div class="rounded-lg border border-neutral-800 bg-neutral-900 p-4 font-mono text-sm">
				<code class="text-neutral-300">drago init</code>
			</div>
		</section>

		<!-- Step 3 -->
		<section class="space-y-4">
			<div class="flex items-center gap-3">
				<span
					class="flex h-8 w-8 items-center justify-center rounded-full bg-neutral-100 text-sm font-bold text-neutral-950"
					>3</span
				>
				<h2 class="text-xl font-bold text-neutral-100">Start the Drago Daemon</h2>
			</div>
			<p class="text-neutral-400">
				Run the daemon to keep your DNS record updated for this machine.
			</p>
			<div class="rounded-lg border border-neutral-800 bg-neutral-900 p-4 font-mono text-sm">
				<code class="text-neutral-300">drago start</code>
			</div>
		</section>
	</div>

	<footer class="border-t pt-6 text-sm text-neutral-500">
		<p>Commands: init | start | stop | status | restart</p>
	</footer>
</div>
