<script>
	import OutgoingTradeElement from './Outgoing.svelte';
	import { listen, emit } from '@tauri-apps/api/event';
	import { writable, readable } from 'svelte/store';
	import { onDestroy, onMount } from 'svelte';
	import { WebviewWindow } from '@tauri-apps/api/window';

	const trades = writable([]);
	let unlisten, unlistenShow, unlistenHide;
	const outgoingTradesWindow = WebviewWindow.getByLabel('outgoing');

	onMount(async () => {
		unlisten = await listen('new-outgoing-trade', (event) => {
			console.log('new trade');
			trades.update((a) => {
				a.push(event.payload);
				return a;
			});
			if ($trades.length > 0) {
				emit('outgoing-trades-show-window', {});
			}
		});

		// not sure that listen callback can handle async funcs
		unlistenShow = await listen('outgoing-trades-show-window', (_e) => {
			outgoingTradesWindow.show();
		});

		unlistenHide = await listen('outgoing-trades-hide-window', (_e) => {
			outgoingTradesWindow.hide();
		});
	});

	onDestroy(() => {
		unlistenHide();
		unlistenShow();
		unlisten();
	});

	function removeFromTrades(uuid) {
		return () => {
			$trades = $trades.filter((t) => t.id !== uuid);
			if ($trades.length === 0) {
				emit('outgoing-trades-hide-window', {});
			}
		};
	}
</script>

<div class="w-96 min-h-full">
	<div>
		<p>trades: {$trades.length}</p>
	</div>
	<div class="overflow-y-auto">
		{#each $trades as trade (trade.id)}
			<OutgoingTradeElement {...trade} onCloseCallback={removeFromTrades(trade.id)} />
		{/each}
	</div>
</div>