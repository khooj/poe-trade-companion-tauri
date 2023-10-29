<script>
	export let id;
	export let playerName;
	export let itemName;
	export let costNumber;
	export let costCurrency;
	export let stash;
	export let lastMessage;
	export let time;

	export let onChatCallback = () => {};
	export let onInviteCallback = () => {};
	export let onTradeCallback = () => {};
	export let onKickCallback = () => {};
	export let onAskToWaitCallback = () => {};
	export let onStillInterestedCallback = () => {};
	export let onInviteToPartyCallback = () => {};
	export let onSoldAlreadyCallback = () => {};
	export let onTyCallback = () => {};
</script>

<div class="flex flex-col">
	<div class="flex">
		<div class="border-2">
			<div>buyer: {playerName} {id}</div>
			<div>item: {itemName}</div>
			<div>price: {costNumber} {costCurrency}</div>
			<div>stash: {stash}</div>
			<div>msg: {lastMessage}</div>
		</div>
		<div class="flex border-2">
			<div>{time}</div>
		</div>
	</div>
	<div class="flex">
		<button on:click={onChatCallback} class="w-12 h-6 border-2">chat</button>
		<button on:click={onInviteCallback} class="w-12 h-6 border-2">inv</button>
		<button on:click={onTradeCallback} class="w-12 h-6 border-2">trade</button>
		<button on:click={onKickCallback} class="w-12 h-6 border-2">kick</button>
	</div>
	<div class="flex flex-col">
		<div class="flex">
			<button on:click={onAskToWaitCallback} class="w-36 h-12 border-2">ask to wait</button>
			<button on:click={onStillInterestedCallback} class="w-36 h-12 border-2">still interested?</button>
			<button on:click={onInviteToPartyCallback} class="w-36 h-12 border-2">invite to party</button>
		</div>
		<div class="flex">
			<button on:click={onSoldAlreadyCallback} class="w-36 h-12 border-2">sold already</button>
			<button on:click={onTyCallback} class="w-72 h-12 border-2">thanks you</button>
		</div>
	</div>
</div>
