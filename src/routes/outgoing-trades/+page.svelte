<script>
	import OutgoingTradeElement from './Outgoing.svelte';
	import { listen, emit } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { writable } from 'svelte/store';
	import { onDestroy, onMount } from 'svelte';
	import { WebviewWindow } from '@tauri-apps/api/window';
	import _ from 'lodash';

	const trades = writable([]);
	let unlisten, unlistenShow, unlistenHide, unlistenMoved;
	const outgoingTradesWindow = WebviewWindow.getByLabel('outgoing');

	onMount(async () => {
		unlisten = await listen('new-outgoing-trade', (event) => {
			console.log(event);
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

		unlistenMoved = outgoingTradesWindow?.onMoved(
			_.debounce(({ payload }) => {
				invoke('update_position_stx', { position: [payload.x, payload.y], window: 'outgoing' });
			}, 1000)
		);
	});

	onDestroy(() => {
		unlistenMoved();
		unlistenHide();
		unlistenShow();
		unlisten();
	});

	function removeFromTrades(uuid) {
		return () => {
			$trades = $trades.filter((t) => t.id !== uuid);
			invoke('trade_close', { id: uuid });
			if ($trades.length === 0) {
				emit('outgoing-trades-hide-window', {});
			}
		};
	}

	function callbacks(id) {
		const m = [
			['outgoing_trade_chat', 'onChatCallback'],
			['outgoing_trade_hideout', 'onHideoutCallback'],
			['outgoing_trade_kick', 'onKickCallback'],
			['outgoing_trade_ty', 'onTyCallback']
		];
		return m.reduce(
			(acc, [evType, prop]) => ({
				...acc,
				[prop]: () => {
					invoke(evType, { id });
				}
			}),
			{}
		);
	}
</script>

<div class="w-96 min-h-full">
	<div>
		<p>trades: {$trades.length}</p>
	</div>
	<div class="overflow-y-auto">
		{#each $trades as trade (trade.id)}
			<OutgoingTradeElement {...trade} onCloseCallback={removeFromTrades(trade.id)} {...callbacks(trade.id)} />
		{/each}
	</div>
</div>
