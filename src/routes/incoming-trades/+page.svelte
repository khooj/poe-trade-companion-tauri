<script>
	import { listen, emit } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onDestroy, onMount } from 'svelte';
	import { WebviewWindow } from '@tauri-apps/api/window';
	import IncomingTrade from './IncomingTrade.svelte';

	let trades = [];
	let currentTrade = null;
	let unlisten, unlistenShow, unlistenHide, unlistenMoved;
	const incomingWindow = WebviewWindow.getByLabel('incoming');

	onMount(async () => {
		unlisten = await listen('new-incoming-trade', (ev) => {
			trades = [...trades, ev.payload];
			if (currentTrade === null) {
				currentTrade = trades[0];
			}
			if (trades.length > 0) {
				emit('incoming-trades-show-window', {});
			}
		});

		unlistenShow = await listen('incoming-trades-show-window', (_e) => {
			incomingWindow?.show();
		});

		unlistenHide = await listen('incoming-trades-hide-window', (_e) => {
			incomingWindow?.hide();
		});

		unlistenMoved = incomingWindow?.onMoved((payload) => {
			console.log(payload);
		});
	});

	onDestroy(() => {
		unlistenMoved();
		unlistenHide();
		unlistenShow();
		unlisten();
	});

	function removeCurrentTrade() {
		const idx = trades.findIndex((el) => el.id === currentTrade.id);
		if (idx !== undefined) {
			invoke('incoming_trade_remove', { id: currentTrade.id });
			trades = [...trades.slice(0, idx), ...trades.slice(idx + 1)];
			if (idx < trades.length) {
				currentTrade = trades[idx];
			} else {
				currentTrade = trades.at(-1);
			}
			if (trades.length === 0) {
				currentTrade = null;
			}
			trades = trades;
		}
		if (trades.length === 0) {
			emit('incoming-trades-hide-window', {});
		}
	}

	function callbacks(id) {
		const m = [
			['incoming_trade_chat', 'onChatCallback'],
			['incoming_trade_invite', 'onInviteCallback'],
			['incoming_trade_trade', 'onTradeCallback'],
			['incoming_trade_kick', 'onKickCallback'],
			['incoming_trade_ask', 'onAskToWaitCallback'],
			['incoming_trade_still', 'onStillInterestedCallback'],
			['incoming_trade_invite_party', 'onInviteToPartyCallback'],
			['incoming_trade_sold', 'onSoldAlreadyCallback'],
			['incoming_trade_ty', 'onTyCallback']
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

<div class="w-108 min-h-full">
	<div>trades: {trades.length}</div>
	<div class="flex justify-between">
		<div class="flex overflow-x-auto">
			{#each trades as trade, i (trade.id)}
				{#if currentTrade && currentTrade.id === trade.id}
					<button class="w-12 h-6 border-2 bg-slate-200" on:click={() => (currentTrade = trade)}>{i}</button>
				{:else}
					<button class="w-12 h-6 border-2" on:click={() => (currentTrade = trade)}>{i}</button>
				{/if}
			{/each}
		</div>
		<div class="flex">
			<button class="w-12 h-6 border-2" on:click={removeCurrentTrade}>close</button>
		</div>
	</div>
	{#if currentTrade}
		<IncomingTrade {...currentTrade} {...callbacks(currentTrade.id)}/>
	{/if}
</div>
