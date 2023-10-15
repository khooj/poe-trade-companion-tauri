<script>
	import OutgoingTradeElement from './Outgoing.svelte';
	import { listen } from '@tauri-apps/api/event';
	import { writable, readable } from 'svelte/store';
	import { onDestroy, onMount } from 'svelte';

	const trades = writable([]);
	let unlisten;

	onMount(async () => {
		unlisten = await listen('new-outgoing-trade', (event) => {
			console.log('new trade');
			event.payload.id = event.payload.id ?? self.crypto.randomUUID();
			trades.update((a) => {
				a.push(event.payload);
				return a;
			});
		});
	});

	onDestroy(async () => {
		unlisten();
	});

	function removeFromTrades(uuid) {
		return () => {
			$trades = $trades.filter((t) => t.id !== uuid);
		};

	}
</script>

<div>
	{#each $trades as trade (trade.id)}
		<OutgoingTradeElement {...trade} onCloseCallback={removeFromTrades(trade.id)}/>
	{/each}
</div>
