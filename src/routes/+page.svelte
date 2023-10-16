<script>
	import { emit } from '@tauri-apps/api/event';

	function newOutgoingTrade() {
		emit('new-outgoing-trade', {
			id: self.crypto.randomUUID(),
			itemName: "Einhar's Memory of Harvest Beasts",
			playerName: 'ShaterJug',
			time: '19:47',
			costNumber: '1.5',
			costCurrency: 'divine',
			lastMessage: 'This player has DND enabled'
		});
	}

	function newIncomingTrade() {
		emit('new-incoming-trade', {
			id: self.crypto.randomUUID(),
			buyer: 'FrostBlade_Ninja',
			item: 'Divine Orb',
			price: '225 Chaos Orb',
			stash: 'Ancestor (tab: asd / pos: 9, 2)',
			lastMessage: '',
			time: '19:49'
		});
	}
</script>

<nav>
	<a href="/">home</a>
	<a href="/about">about</a>
	<a href="/outgoing-trades">outgoing</a>
</nav>

<h1 class="text-3xl font-bold underline">Home</h1>
<p>this is the home page.</p>
<button on:click={newOutgoingTrade}>New outgoing trade event</button>
<button on:click={newIncomingTrade}>New incoming trade event</button>

<style lang="postcss">
	:global(html) {
		background-color: theme(colors.gray.100);
	}
</style>
