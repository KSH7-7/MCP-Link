<script>
  import { onMount } from 'svelte';
  import { move_window, Position } from 'tauri-plugin-positioner-api';
  
  onMount(async () => {
    try {
      await move_window(Position.RightBottom);
    } catch (error) {
    }
  });
</script>

<div class="popup-content">
  <h1>팝업</h1>
</div>

<style>
  .popup-content {
    padding: 16px;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }
</style>