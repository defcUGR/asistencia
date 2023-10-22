<template>
  <header
    class="bg-slate-50 dark:bg-slate-950 font-black px-8 py-5 text-3xl dark:text-white tracking-wide"
  >
    Asistencia DEFC
  </header>
  <main
    class="bg-white text-slate-950 dark:bg-[#141414] dark:text-[#E5EAF3] p-5"
  >
    <template v-if="!configured">
      <h1 class="text-2xl font-semibold mb-3">Preparación de la sesión</h1>
      <h2 class="text-xl mb-2">Archivo de censo JSON</h2>
      <div
        class="rounded-lg border-[var(--el-border-color)] border-2 border-dashed flex flex-col items-center justify-center p-5"
        drag
        @click.prevent="openFile"
        :class="
          processingFile && filePath
            ? 'cursor-wait'
            : filePath
            ? ' cursor-default'
            : 'cursor-pointer'
        "
      >
        <template v-if="!filePath">
          <IconUpload class="w-[10%] h-[10%] stroke-1 mb-2"></IconUpload>
          <div class="font-light italic">
            Arrastra el archivo o
            <em class="text-sky-400">haz click para seleccionar</em>
          </div>
        </template>
        <template v-else-if="processingFile">
          <IconLoader2
            class="w-[10%] h-[10%] stroke-1 mb-2 animate-spin"
          ></IconLoader2>
          <div class="font-light italic">
            Procesando
            <span
              class="bg-zinc-950 px-1.5 py-0.5 rounded cursor-pointer"
              @click.stop="systemOpenFile"
              >{{ filePath }}</span
            >...
          </div>
        </template>
        <template v-else-if="fileError">
          <IconAlertHexagon
            class="w-[10%] h-[10%] stroke-1 mb-2"
          ></IconAlertHexagon>
          <div class="font-light italic">
            Error procesando
            <span
              class="bg-zinc-950 px-1.5 py-0.5 rounded cursor-pointer"
              @click.stop="systemOpenFile"
              >{{ filePath }}</span
            >:<br />
            {{ fileError }}
          </div>
        </template>
        <template v-else>
          <IconCheck class="w-[10%] h-[10%] stroke-1 mb-2"></IconCheck>
          <div class="font-light italic">
            Procesado
            <span
              class="bg-zinc-950 px-1.5 py-0.5 rounded cursor-pointer"
              @click.stop="systemOpenFile"
              >{{ filePath }}</span
            >
            exitosamente
          </div>
        </template>
      </div>
      <div class="text-sm text-[var(--el-border-color)]">
        Debe de ser un CSV con las columnas correctas
      </div>
      <h2 class="text-xl my-2">Puertos disponibles</h2>
      <ul>
        <li @click="selectedPort = 0">
          <el-card
            shadow="hover"
            :class="selectedPort === 0 ? '!border-sky-400' : ''"
          >
            <h3 class="font-bold text-lg">Teclado</h3>
            <p>Simula la entrada del lector usando el teclado</p>
          </el-card>
        </li>
        <li
          v-for="(port, index) in ports"
          @click="selectedPort = index + 1"
          class="mt-2"
        >
          <el-card
            shadow="hover"
            :class="selectedPort === index + 1 ? '!border-sky-400' : ''"
          >
            <h3 class="font-bold text-lg">{{ port.port_name }}</h3>
            <template v-for="(val, prop) in port"
              ><p v-if="prop != 'port_name'">
                <span class="capitalize">{{ prop.replace("_", " ") }}</span
                >: {{ val }}
              </p></template
            >
          </el-card>
        </li>
      </ul>
      <div class="w-screen fixed bottom-0 left-0 p-4">
        <el-button
          type="primary"
          class="w-full"
          :disabled="!(filePath && !fileError && selectedPort !== undefined)"
          @click="
            () =>
              filePath && !fileError && selectedPort !== undefined
                ? (configured = true)
                : {}
          "
        >
          Empezar <el-icon class="el-icon--right"><IconChevronRight /></el-icon>
        </el-button>
      </div>
    </template>
    <template v-else>
      <div class="flex flex-row justify-between items-center mb-3">
        <h1 class="text-2xl font-semibold">Escaneando...</h1>
        <el-button text type="info" @click="dataDialogOpen = true"
          >Ver datos</el-button
        >
      </div>

      <el-dialog v-model="dataDialogOpen" title="Datos del censo">
        {{ userData }}
      </el-dialog>
    </template>
  </main>
</template>

<script setup lang="ts">
import { useDark } from "@vueuse/core";
import {
  IconChevronRight,
  IconUpload,
  IconCheck,
  IconLoader2,
  IconAlertHexagon,
} from "@tabler/icons-vue";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";
import { homeDir } from "@tauri-apps/api/path";
import { open as systemOpen } from "@tauri-apps/api/shell";

useDark();

const configured = ref(false);

const dataDialogOpen = ref(false);

const ports = ref(
  [] as {
    port_name: string;
    port_type: string;
  }[]
);
const selectedPort = ref();

const userData = ref();

const filePath = ref();
const processingFile = ref(true);
const fileError = ref();
const openFile = async () => {
  fileError.value = false;

  //* Pick
  const path = await open({
    multiple: false,
    filters: [
      {
        name: "Hoja CSV",
        extensions: ["csv"],
      },
    ],
    defaultPath: await homeDir(),
  });
  if (!path || Array.isArray(path)) return;
  filePath.value = path;

  //* Process
  try {
    userData.value = await invoke("process_csv", { path });
  } catch (e) {
    console.error(e);
    fileError.value = e;
  } finally {
    processingFile.value = false;
  }
};
const systemOpenFile = () => systemOpen(filePath.value);

(async () => {
  ports.value = await invoke("get_serial_ports");
})();
</script>
