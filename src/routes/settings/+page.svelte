<script>
	import { listen, emit } from '@tauri-apps/api/event';
	import { invoke } from '@tauri-apps/api/tauri';
	import { onDestroy, onMount } from 'svelte';
	import { WebviewWindow } from '@tauri-apps/api/window';
	import { open } from '@tauri-apps/api/dialog';
	import _ from 'lodash';

	const settingsWindow = WebviewWindow.getByLabel('settings');

	onMount(async () => {
	});

	let logpath = "";

	function saveSetting() {
		invoke('update_logpath_stx', { logpath });
	}

	function onClose() {
		settingsWindow.hide();
	}

	async function pickFile() {
		const selected = await open({
			multiple: false,
			filters: [{
				name: 'Text files',
				extensions: ['txt', 'log']
			}]
		});
		logpath = selected;
	}
</script>

<div class="w-108 min-h-full">
	<input type="text" bind:value={logpath} />
	<button on:click={pickFile}>select file</button>
	<button on:click={saveSetting}>save</button>
	<button on:click={onClose}>close</button>
</div>
