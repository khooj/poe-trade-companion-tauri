<script>
	import ChatSvg from './svg/chat.svelte';
	import HideoutSvg from './svg/hideout.svelte';
	import KickSvg from './svg/kick.svelte';
	import TySvg from './svg/ty.svelte';

	const svgHeight = 20;
	const svgWidth = 20;

	export let id;
	export let itemName;
	export let playerName;
	export let time;
	export let costNumber;
	export let costCurrency;
	export let lastMessage;

	export let cutLength = 15;

	export let onChatCallback = () => {};
	export let onHideoutCallback = () => {};
	export let onKickCallback = () => {};
	export let onTyCallback = () => {};
	export let onCloseCallback = () => {};

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
			<div>{costNumber} {costCurrency}</div>
			<div>{lastMessageCutted}</div>
			<div>
				<button on:click={onChatCallback}><ChatSvg height={svgHeight} width={svgWidth} /></button>
				<button on:click={onHideoutCallback}><HideoutSvg height={svgHeight} width={svgWidth} /></button>
				<button on:click={onKickCallback}><KickSvg height={svgHeight} width={svgWidth} /></button>
				<button on:click={onTyCallback}><TySvg height={svgHeight} width={svgWidth} /></button>
			</div>
		</div>
	</div>
	<button class="border-solid border-2" on:click={onCloseCallback}>close</button>
</div>
