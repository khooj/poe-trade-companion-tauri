<script>
	import { emit } from '@tauri-apps/api/event';
	import { exit } from '@tauri-apps/api/process';
	import { invoke } from '@tauri-apps/api/tauri';
	import { WebviewWindow } from '@tauri-apps/api/window';

	function newOutgoingTrade() {
		invoke('spawn_outgoing_trade', { msg: 'Hi tracechat test' });
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

	function showSettings() {
		const stx = WebviewWindow.getByLabel('settings');
		stx.show();
	}
</script>

<nav>
	<a href="/">home</a>
	<a href="/about">about</a>
</nav>

<h1 class="text-3xl font-bold underline">Home</h1>
<p>this is the home page.</p>
<div class="flex">
	<button class="border-2" on:click={newOutgoingTrade}>New outgoing trade event</button>
	<button class="border-2" on:click={newIncomingTrade}>New incoming trade event</button>
	<button class="border-2" on:click={exitProcess}>Exit</button>
	<button class="border-2" on:click={showSettings}>Settings</button>
</div>

<style lang="postcss">
	:global(html) {
		background-color: theme(colors.gray.100);
	}
</style>
