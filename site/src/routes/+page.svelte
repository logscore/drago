<script lang="ts">
	import Header from '../components/Header.svelte';
</script>

<svelte:head>
	<title>DragoDNS — Dynamic DNS</title>
	<meta name="description" content="Reliable dynamic DNS for remote access with instant updates." />
</svelte:head>

<Header />
<main class="container mx-auto px-4">
	<!-- Hero -->
	<section class="grid items-center gap-8 py-16 md:grid-cols-2">
		<div>
			<h1 class="mb-4 text-4xl font-bold md:text-5xl">Dynamic DNS Made Simple</h1>
			<p class="mb-6 text-xl text-surface-600 dark:text-surface-400">
				Access your devices anywhere with a human-friendly hostname.
			</p>
		</div>

		<div class="overflow-hidden rounded-xl border border-surface-200 dark:border-surface-700">
			<!-- Linux-only notice -->
			<div
				class="flex items-center gap-3 bg-amber-50 px-4 py-2 text-amber-900 dark:bg-amber-900/20 dark:text-amber-200"
			>
				<img src="/tux.svg" alt="Linux Logo" class="size-6" />
				<span class="text-sm font-medium">Supported OS: Linux only</span>
			</div>

			<!-- Code block -->
			<div class="overflow-auto bg-surface-100 p-6 dark:bg-surface-800">
				<pre class="text-sm leading-relaxed"><code
						># Install the client
curl -sSL https://www.dragodns.com/install.sh | sh

# Initialize the client (have a Cloudflare API token ready)
drago init

# Start the daemon
drago start</code
					></pre>
			</div>
		</div>
	</section>

	<!-- Features -->
	<section id="features" class="py-12">
		<div class="mb-10 text-center">
			<h2 class="mb-3 text-3xl font-bold">Why DragoDNS?</h2>
			<p class="text-lg text-surface-600 dark:text-surface-400">
				Fast, lightweight, simple, and global DDNS.
			</p>
		</div>

		<div class="grid gap-6 md:grid-cols-4">
			<div class="rounded-xl border border-surface-200 p-6 dark:border-surface-700">
				<div class="mb-3 text-3xl">⚡</div>
				<h3 class="mb-2 text-xl font-bold">Fast Propagation</h3>
				<p class="text-surface-600 dark:text-surface-400">Updates in seconds, not minutes.</p>
			</div>

			<div class="rounded-xl border border-surface-200 p-6 dark:border-surface-700">
				<div class="mb-3 text-3xl">🪶</div>
				<h3 class="mb-2 text-xl font-bold">Lightweight</h3>
				<p class="text-surface-600 dark:text-surface-400">~1.5 MB binary, 0.1 % CPU usage</p>
			</div>

			<div class="rounded-xl border border-surface-200 p-6 dark:border-surface-700">
				<div class="mb-3 text-3xl">✨</div>
				<h3 class="mb-2 text-xl font-bold">Simple</h3>
				<p class="text-surface-600 dark:text-surface-400">Run one command and it just works</p>
			</div>

			<div class="rounded-xl border border-surface-200 p-6 dark:border-surface-700">
				<img src="/cloudflare.svg" alt="Cloudflare Logo" class="size-13" />
				<h3 class="mb-2 text-xl font-bold">Built on Cloudflare</h3>
				<p class="text-surface-600 dark:text-surface-400">Low latency worldwide DNS propagation.</p>
			</div>
		</div>
	</section>

	<!-- Pricing -->
	<section id="pricing" class="py-12">
		<div class="mb-10 text-center">
			<h2 class="mb-3 text-3xl font-bold">Simple Pricing</h2>
			<p class="text-lg text-surface-600 dark:text-surface-400">Start free. Scale as you grow.</p>
		</div>

		<div class="grid gap-6 md:grid-cols-2">
			<div class="rounded-xl border border-surface-200 p-6 text-center dark:border-surface-700">
				<h3 class="mb-2 text-xl font-bold">Free</h3>
				<p class="mb-4 text-3xl font-bold">$0</p>
				<ul class="mb-6 space-y-2 text-left">
					<li>✓ 1 hostname</li>
					<li>✓ Community support</li>
				</ul>
				<a
					href="/auth"
					class="block w-full rounded border border-surface-300 px-4 py-2 transition-colors hover:bg-surface-100 dark:border-surface-600 dark:hover:bg-surface-800"
					>Get Started</a
				>
			</div>

			<div
				class="relative overflow-hidden rounded-xl border-2 border-dashed border-yellow-400 p-6 text-center"
			>
				<!-- Dimmed Content -->
				<div class="pointer-events-none opacity-30 grayscale">
					<h3 class="mb-2 text-xl font-bold">Pro</h3>
					<p class="mb-4 text-3xl font-bold">
						$1<span class="text-lg text-surface-600 dark:text-surface-400">/record</span>
					</p>
					<ul class="mb-6 space-y-2 text-left">
						<li>✓ More records</li>
						<li>✓ Priority updates</li>
						<li>✓ Email support</li>
					</ul>
					<div class="block w-full rounded bg-primary-500 px-4 py-2 text-white">Go Pro</div>
				</div>

				<!-- Crossed Caution Tapes -->
				<div
					class="pointer-events-none absolute inset-0 flex items-center justify-center bg-black/40"
				>
					<!-- Tape 1 (Forward Slash) -->
					<div
						class="absolute h-10 w-[140%] rotate-15 border-y-2 border-yellow-400 bg-[repeating-linear-gradient(45deg,#000,#000_20px,#eab308_20px,#eab308_40px)] shadow-xl"
					></div>
					<!-- Tape 2 (Back Slash) -->
					<div
						class="absolute flex h-10 w-[140%] -rotate-15 items-center justify-center border-y-2 border-yellow-400 bg-[repeating-linear-gradient(-45deg,#000,#000_20px,#eab308_20px,#eab308_40px)] shadow-xl"
					>
						<span
							class="bg-black px-3 py-0.5 text-xs font-black tracking-widest text-yellow-400 uppercase outline outline-yellow-400"
						>
							Coming Soon — For now, enjoy unlimited DDNS
						</span>
					</div>
				</div>
			</div>
		</div>
	</section>

	<!-- Footer -->
	<section class="border-t border-surface-200 py-10 dark:border-surface-700">
		<div class="container mx-auto flex flex-col items-center justify-between gap-4 md:flex-row">
			<div>© 2025 DragoDNS. All rights reserved.</div>
		</div>
	</section>
</main>
