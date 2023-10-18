<script>
	import { emit } from '@tauri-apps/api/event';
	import { exit } from '@tauri-apps/api/process';

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

	function exitProcess() {
		exit(0);
	}
</script>

<nav>
	<a href="/">home</a>
	<a href="/about">about</a>
	<a href="/outgoing-trades">outgoing</a>
</nav>

<h1 class="text-3xl font-bold underline">Home</h1>
<p>this is the home page.</p>
<button class="border-2" on:click={newOutgoingTrade}>New outgoing trade event</button>
<button class="border-2" on:click={newIncomingTrade}>New incoming trade event</button>
<button class="border-2" on:click={exitProcess}>Exit</button>

<style lang="postcss">
	:global(html) {
		background-color: theme(colors.gray.100);
	}
</style>
