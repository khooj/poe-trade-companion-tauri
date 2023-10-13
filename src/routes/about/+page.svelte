<script>
	export let data;
</script>

<h1>about</h1>
<p>this is the about page.</p>
{#each data.summaries as { slug, title }}
	<li><a href="/about/{slug}">{title}</a></li>
{/each}
