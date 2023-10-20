<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Input, Button } from "flowbite-vue";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <Input id="greet-input" v-model="name" placeholder="Enter a name..." />
    <Button gradient="blue" shadow>Greet</Button>
  </form>

  <p>{{ greetMsg }}</p>
</template>
