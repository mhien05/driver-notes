<script lang="ts">
  const stack = ['Svelte 5', 'TypeScript', 'Vite'] as const
</script>

<main class="app-shell">
  <header class="app-header">
    <span class="app-mark" aria-hidden="true">DN</span>
    <span>Driver Notes</span>
  </header>

  <section class="intro" aria-labelledby="intro-title">
    <p class="eyebrow">Nền tảng giao diện</p>
    <h1 id="intro-title">Ghi chép ca làm thật gọn.</h1>
    <p class="intro-copy">
      Giao diện Driver Notes đã sẵn sàng để phát triển các chức năng ghi thu nhập,
      chi phí và xem lịch sử dành cho tài xế giao hàng.
    </p>
    <ul class="stack" aria-label="Công nghệ frontend">
      {#each stack as technology}
        <li>{technology}</li>
      {/each}
    </ul>
  </section>
</main>
