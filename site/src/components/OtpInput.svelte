<script lang="ts">
	let { length = 8, value = $bindable(), disabled = false } = $props();

	let inputs: HTMLInputElement[] = [];

	const updateValue = () => {
		value = inputs.map((i) => i.value).join('');
	};

	const onInput = (e: Event, index: number) => {
		const input = e.target as HTMLInputElement;
		const char = input.value.replace(/[^A-Z0-9]/gi, '').toUpperCase();

		input.value = char.slice(-1);
		updateValue();

		if (char && index < length - 1) {
			inputs[index + 1]?.focus();
		}
	};

	const onKeyDown = (e: KeyboardEvent, index: number) => {
		if (e.key === 'Backspace' && !inputs[index].value && index > 0) {
			inputs[index - 1].focus();
		}

		if (e.key === 'ArrowLeft' && index > 0) {
			inputs[index - 1].focus();
		}

		if (e.key === 'ArrowRight' && index < length - 1) {
			inputs[index + 1].focus();
		}
	};

	const onPaste = (e: ClipboardEvent) => {
		e.preventDefault();
		const pasted =
			e.clipboardData
				?.getData('text')
				.replace(/[^A-Z0-9]/gi, '')
				.toUpperCase()
				.slice(0, length) || '';

		pasted.split('').forEach((char, i) => {
			if (inputs[i]) inputs[i].value = char;
		});

		updateValue();
		inputs[Math.min(pasted.length, length - 1)]?.focus();
	};
</script>

<div class="flex items-center justify-center gap-2">
	{#each Array(length) as _, i}
		<input
			bind:this={inputs[i]}
			type="text"
			inputmode="text"
			maxlength="1"
			{disabled}
			oninput={(e) => onInput(e, i)}
			onkeydown={(e) => onKeyDown(e, i)}
			onpaste={onPaste}
			class="h-14 w-12 rounded-lg border-2 border-neutral-700 bg-neutral-800 text-center font-mono text-xl font-semibold text-neutral-100 transition-all focus:border-neutral-500 focus:bg-neutral-700 focus:ring-2 focus:ring-neutral-500/20 focus:outline-none disabled:opacity-50"
		/>
		{#if i === 3}
			<div class="text-2xl font-bold text-neutral-500">-</div>
		{/if}
	{/each}
</div>
