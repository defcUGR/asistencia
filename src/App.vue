<template>
  <header
    class="bg-slate-50 dark:bg-slate-950 font-black px-8 py-5 text-3xl dark:text-white tracking-wide"
  >
    Asistencia DEFC
  </header>
  <main class="bg-white text-slate-950 dark:bg-black dark:text-slate-50 p-5">
    <h1 class="text-2xl font-semibold">Preparación de la sesión</h1>
    <h2>Archivo de censo JSON</h2>
    <h2>Puertos disponibles</h2>
    <ul>
      <li v-for="port in ports">{{ port }}</li>
    </ul>
  </main>
</template>

<script setup lang="ts">
const ports = ref([]);
import { invoke } from "@tauri-apps/api";

(async () => {
  ports.value = await invoke("get_serial_ports");
})();
</script>
