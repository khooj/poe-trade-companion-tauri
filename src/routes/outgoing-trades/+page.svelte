<script>
	import OutgoingTradeElement from './Outgoing.svelte';
	export let data;
</script>

<div>
	{#each data.trades as trade}
		<OutgoingTradeElement { ...trade }/>
	{/each}
</div>