<script>
	import { listen, emit } from '@tauri-apps/api/event';
	import { writable } from 'svelte/store';
	import { onDestroy, onMount } from 'svelte';
	import { WebviewWindow } from '@tauri-apps/api/window';
	import IncomingTrade from './IncomingTrade.svelte';

	let trades = [];
	let currentTrade = null;
	let unlisten, unlistenShow, unlistenHide;
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
	});

	onDestroy(() => {
		unlistenHide();
		unlistenShow();
		unlisten();
	});

	function removeCurrentTrade() {
		const idx = trades.findIndex((el) => el.id === currentTrade.id);
		if (idx !== undefined) {
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
			['incoming-trade-chat', 'onChatCallback'],
			['incoming-trade-invite', 'onInviteCallback'],
			['incoming-trade-trade', 'onTradeCallback'],
			['incoming-trade-kick', 'onKickCallback'],
			['incoming-trade-ask', 'onAskToWaitCallback'],
			['incoming-trade-still', 'onStillInterestedCallback'],
			['incoming-trade-invite-party', 'onInviteToPartyCallback'],
			['incoming-trade-sold', 'onSoldAlreadyCallback'],
			['incoming-trade-ty', 'onTyCallback']
		];
		return m.reduce(
			(acc, [evType, prop]) => ({
				...acc,
				[prop]: () => {
					emit(evType, { id });
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
