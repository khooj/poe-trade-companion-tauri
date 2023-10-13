<script>
	import ChatSvg from './svg/chat.svelte';
	import HideoutSvg from './svg/hideout.svelte';
	import KickSvg from './svg/kick.svelte';
	import TySvg from './svg/ty.svelte';

	const svgHeight = 20;
	const svgWidth = 20;

	export let itemName;
	export let playerName;
	export let time;
	export let cost;
	export let lastMessage;

	export let cutLength = 15;

	$: itemNameCutted = itemName.substring(0, cutLength) + '...';
	$: playerNameCutted = playerName.substring(0, cutLength) + '...';
	$: lastMessageCutted = lastMessage.substring(0, cutLength) + '...';
</script>

<div class="flex w-96 border-solid border-black border-2">
	<div class="flex flex-col grow border-solid border-2">
		<div class="flex justify-between">
			<div>{itemNameCutted}</div>
			<div>{playerNameCutted}</div>
			<div>{time}</div>
		</div>
		<div class="flex justify-between">
			<div>{cost}</div>
			<div>{lastMessageCutted}</div>
			<div>
				<button><ChatSvg height={svgHeight} width={svgWidth} /></button>
				<button><HideoutSvg height={svgHeight} width={svgWidth} /></button>
				<button><KickSvg height={svgHeight} width={svgWidth} /></button>
				<button><TySvg height={svgHeight} width={svgWidth} /></button>
			</div>
		</div>
	</div>
	<button class="border-solid border-2">close</button>
</div>