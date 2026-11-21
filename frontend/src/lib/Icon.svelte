<script lang="ts">
  export let name: string;
  export let size = 22;
  export let stroke = 1.8;

  const paths: Record<string, string> = {
    home: "m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-6H9v6H4a1 1 0 0 1-1-1z",
    list: "M4 6h16M4 12h16M4 18h16",
    clock: "M12 3a9 9 0 1 0 0 18 9 9 0 0 0 0-18Zm0 5v5l3 2",
    user: "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM4 21a8 8 0 0 1 16 0",
    plus: "M12 5v14M5 12h14",
    arrow: "M5 12h14m-6-6 6 6-6 6",
    back: "M19 12H5m6-6-6 6 6 6",
    sun: "M12 3v2m0 14v2M3 12h2m14 0h2M5.6 5.6 7 7m10 10 1.4 1.4M18.4 5.6 17 7M7 17l-1.4 1.4M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z",
    moon: "M20.5 15.5A9 9 0 0 1 8.5 3.5 9 9 0 1 0 20.5 15.5Z",
    logout: "M10 4H5a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h5m4-4 4-4-4-4m4 4H9",
    truck:
      "M3 6h11v11H3zM14 10h4l3 3v4h-7zM7 20a2 2 0 1 0 0-4 2 2 0 0 0 0 4Zm11 0a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z",
    wallet:
      "M4 6h15a2 2 0 0 1 2 2v11H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h14M16 14h5m-5 0a1 1 0 1 0 2 0 1 1 0 0 0-2 0Z",
    fuel: "M5 20V5a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v15M3 20h15M6 8h9M16 8l3 3v6a2 2 0 0 0 4 0v-6l-3-3",
    gift: "M4 10h16v11H4zM2 7h20v3H2zM12 7v14M12 7c-4-5-8-3-6 0 1 2 6 0 6 0Zm0 0c4-5 8-3 6 0-1 2-6 0-6 0Z",
    x: "M5 5l14 14M19 5 5 19",
    chevron: "m9 5 7 7-7 7",
    refresh:
      "M20 7v5h-5M4 17v-5h5M5.5 9A7 7 0 0 1 18 7l2 5M4 12l2 5a7 7 0 0 0 12.5-2",
    check: "m4 12 5 5L20 6",
    pencil: "m4 20 4-.8L20 7l-3-3L4.8 16zM15 6l3 3",
    trash: "M4 7h16M9 7V4h6v3m3 0-1 13H7L6 7m4 4v6m4-6v6",
    wifi: "M3 9a15 15 0 0 1 18 0M6 12a10 10 0 0 1 12 0m-9 3a5 5 0 0 1 6 0m-3 4h.01",
    calendar: "M4 5h16v16H4zM8 3v4m8-4v4M4 10h16",
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
>
  <path d={paths[name] ?? paths.home} />
</svg>
