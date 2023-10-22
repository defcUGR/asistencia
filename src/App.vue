<template>
  <header
    class="bg-slate-50 dark:bg-slate-950 font-black px-8 py-5 text-3xl dark:text-white tracking-wide"
  >
    Asistencia DEFC
  </header>
  <main
    class="bg-white text-slate-950 dark:bg-[#141414] dark:text-[#E5EAF3] p-5 pb-16"
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
                ? ((configured = true), (scanning = true))
                : {}
          "
        >
          Empezar <el-icon class="el-icon--right"><IconChevronRight /></el-icon>
        </el-button>
      </div>
    </template>
    <template v-else>
      <!--* Page Title -->
      <div class="flex flex-row justify-between items-center mb-3">
        <h1
          class="text-2xl font-semibold"
          :class="scanning ? '' : 'text-rose-400'"
        >
          {{ scanning ? "Escaneando..." : "Escáner pausado" }}
        </h1>
        <div>
          <el-button text type="info" @click="dataDialogOpen = true"
            >Ver datos completos</el-button
          >
          <el-button type="primary">
            <el-icon class="el-icon--left"><IconFileTypeCsv /></el-icon>
            Exportar CSV
          </el-button>
        </div>
      </div>

      <!--* Data Dialog -->
      <el-dialog v-model="dataDialogOpen" title="Datos del censo">
        <VueJsonPretty :data="userData" virtual :height="600" />
      </el-dialog>

      <el-card
        v-for="(att, idx) in scanned"
        :key="att.data.tui"
        class="mt-2 relative !min-h-[110px]"
      >
        <el-text class="!absolute !right-5 !top-5" type="info">{{
          format(att.time, "HH:mm")
        }}</el-text>

        <!--* Action buttons -->
        <div class="absolute right-5 bottom-5">
          <el-button @click="att.dialogOpen = true">
            <el-icon class="el-icon--left"><IconCode /></el-icon>
            Datos
          </el-button>
          <el-button type="danger" @click="deleteAttendant(idx)"
            ><el-icon class="el-icon--left"><IconTrashX /></el-icon
            >Eliminar</el-button
          >
        </div>

        <!--* Data dialog -->
        <el-dialog
          v-model="att.dialogOpen"
          :title="`Datos de ${att.data.full_name}`"
        >
          <VueJsonPretty :data="att" />
        </el-dialog>

        <h3 class="inline font-bold text-lg mr-2" v-if="att.data.full_name">
          {{ att.data.full_name }}
        </h3>
        <el-select-v2
          v-else
          v-model="att.data.full_name"
          :options="availableNames"
          filterable
          placeholder="Seleccionar nombre"
          class="mr-2"
          @change="updateAttendant(idx)"
        ></el-select-v2>
        <span class="text-zinc-500">{{ att.data.tui }}</span>

        <p class="mb-1">
          {{ att.data.degree }} {{ att.data.course ? "·" : "" }}
          {{ att.data.course }} {{ att.data.group ? "·" : "" }}
          {{ att.data.group }}
        </p>

        <div>
          <template v-for="(checkVal, checkKey) in att.checks">
            <el-tag
              :color="tagColor(checkKey)"
              v-if="checkVal"
              class="!text-white mr-2"
              type="info"
              >{{ tagName(checkKey, att.data.pronouns) }}</el-tag
            >
          </template>
        </div>
      </el-card>

      <!--* Bottom buttons -->
      <div class="w-screen fixed bottom-0 left-0 p-4">
        <p class="mb-4 w-full text-center italic text-zinc-500">
          {{ input }}
        </p>
        <el-button
          type="danger"
          class="w-full"
          :disabled="!(filePath && !fileError && selectedPort !== undefined)"
          @click="scanning = false"
          v-if="scanning"
        >
          Parar Escáner
          <el-icon class="el-icon--right"><IconPlayerPause /></el-icon>
        </el-button>

        <el-button
          type="success"
          class="w-full"
          :disabled="!(filePath && !fileError && selectedPort !== undefined)"
          @click="scanning = true"
          v-else
        >
          Retomar Escáner
          <el-icon class="el-icon--right"><IconPlayerPlay /></el-icon>
        </el-button>
      </div>
    </template>
  </main>
</template>

<script setup lang="ts">
import { onKeyStroke, useDark } from "@vueuse/core";
import {
  IconChevronRight,
  IconUpload,
  IconCheck,
  IconLoader2,
  IconAlertHexagon,
  IconPlayerPause,
  IconPlayerPlay,
  IconTrashX,
  IconCode,
  IconFileTypeCsv,
} from "@tabler/icons-vue";
import { invoke } from "@tauri-apps/api";
import { open } from "@tauri-apps/api/dialog";
import { homeDir } from "@tauri-apps/api/path";
import { open as systemOpen } from "@tauri-apps/api/shell";
import VueJsonPretty from "vue-json-pretty";
import "vue-json-pretty/lib/styles.css";
import type { Attendant } from "../src-tauri/bindings/Attendant";
import type { AttendantChecks } from "../src-tauri/bindings/AttendantChecks";
import type { RawAttendant } from "../src-tauri/bindings/RawAttendant";
import format from "date-fns/format";

useDark();

const configured = ref(false);
const scanning = ref(false);

const dataDialogOpen = ref(false);

const ports = ref(
  [] as {
    port_name: string;
    port_type: string;
  }[]
);
const selectedPort = ref();

const userData: Ref<Attendant[] | undefined> = ref();

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

const availableNames = computed(() =>
  userData.value
    ?.filter((usr) => !usr.raw.tui)
    .map((usr) => ({ value: usr.raw.full_name, label: usr.raw.full_name }))
);

watch(availableNames, (val) => console.info("names", val));

const input = ref("");
onKeyStroke(
  ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
  (e) => {
    e.preventDefault();
    if (scanning.value) input.value += e.key;
  },
  { dedupe: true }
);
onKeyStroke(["Delete", "Backspace"], (e) => {
  e.preventDefault();
  if (input.value.length > 0 && scanning)
    input.value = input.value.slice(0, input.value.length - 1);
});

type Checks = keyof AttendantChecks;

const scanned = ref(
  [] as {
    data: Partial<RawAttendant>;
    new: boolean;
    checks: AttendantChecks | {};
    time: Date;
    dialogOpen: boolean;
  }[]
);
watch(input, () => {
  if (input.value.length !== 7) {
    return;
  }

  const readInput = input.value;
  input.value = "";

  const fdUser = userData.value?.find((usr) => usr.raw.tui === readInput);

  if (fdUser) {
    scanned.value.push({
      data: fdUser.raw,
      checks: fdUser.checks,
      new: false,
      time: new Date(),
      dialogOpen: false,
    });
  } else {
    scanned.value.push({
      data: {
        tui: readInput,
      },
      checks: {},
      new: true,
      time: new Date(),
      dialogOpen: false,
    });
  }
});

const tagName = computed(
  () => (key: Checks, pronouns: "F" | "M" | "N" | null | undefined) => {
    const end =
      pronouns === null || pronouns === undefined || pronouns === "N"
        ? "x"
        : ({
            F: "a",
            M: "o",
          }[pronouns] as "x" | "a" | "o");
    return {
      is_delegado: "Delegad" + end,
      is_subdelegado: "Subdelegad" + end,
      is_electo: "Elect" + end,
      is_claustro: "Claustral",
      is_junta_de_centro: "Junta de Centro",
      is_voluntario: "Voluntari" + end,
    }[key];
  }
);

const tagColor = computed(
  () => (key: Checks) =>
    ({
      is_delegado: "#2cf38c44",
      is_subdelegado: "#d7f74044",
      is_electo: "#a12cf344",
      is_claustro: "#e3591644",
      is_junta_de_centro: "#167de344",
      is_voluntario: "#e5b5d344",
    }[key])
);

const updateAttendant = (idx: number) => {
  const fdUser = userData.value?.find(
    (usr) => usr.raw.full_name === scanned.value[idx].data.full_name
  );

  if (!fdUser) {
    console.error("Full name not found", scanned.value[idx].data.full_name);
    return;
  }
  scanned.value[idx] = {
    data: { ...fdUser.raw, tui: scanned.value[idx].data.tui },
    checks: fdUser.checks,
    new: true,
    time: scanned.value[idx].time,
    dialogOpen: false,
  };
};

const deleteAttendant = (idx: number) => scanned.value.splice(idx, 1);
</script>
