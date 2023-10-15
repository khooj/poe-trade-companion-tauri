<script>
	import { emit } from '@tauri-apps/api/event';

	function newOutgoingTrade() {
		emit('new-outgoing-trade', {
			itemName: "Einhar's Memory of Harvest Beasts",
			playerName: 'ShaterJug',
			time: '19:47',
			costNumber: '1.5',
			costCurrency: 'divine',
			lastMessage: 'This player has DND enabled'
		});
	}
</script>

<h1 class="text-3xl font-bold underline">Home</h1>
<p>this is the home page.</p>
<button on:click={newOutgoingTrade}>New outgoing trade event</button>

<style lang="postcss">
	:global(html) {
		background-color: theme(colors.gray.100);
	}
</style>
