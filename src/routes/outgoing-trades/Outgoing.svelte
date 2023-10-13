<script>
	export let itemName;
	export let playerName;
	export let time;
	export let cost;
	export let lastMessage;

	export let cutLength = 10;

	$: itemNameCutted = itemName.substring(0, cutLength) + '...';
	$: playerNameCutted = playerName.substring(0, cutLength) + '...';
	$: lastMessageCutted = lastMessage.substring(0, cutLength) + '...';
</script>

<div class="flex flex-col w-96 border-solid border-black border-4">
	<div class="flex justify-between">
		<div>{itemNameCutted}</div>
		<div>{playerNameCutted}</div>
		<div>{time}</div>
	</div>
	<div class="flex justify-between">
		<div>{cost}</div>
		<div>{lastMessageCutted}</div>
		<div>
			<button>chat</button>
			<button>home</button>
			<button>kick</button>
			<button>ty</button>
		</div>
	</div>
</div>