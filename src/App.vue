<template>
  <ErrorBoundary />

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
              class="bg-zinc-600 text-white dark:bg-zinc-950 px-1.5 py-0.5 rounded cursor-pointer"
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
              class="bg-zinc-600 text-white dark:bg-zinc-950 px-1.5 py-0.5 rounded cursor-pointer"
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
        <li
          v-for="(port, index) in ports"
          @click="selectedPort = index"
          class="mt-2"
        >
          <el-card
            shadow="hover"
            :class="selectedPort === index ? '!border-sky-400' : ''"
          >
            <h3 class="font-bold text-lg">
              {{ port.info === "keyboard" ? "Teclado" : port.info.port_name }}
            </h3>
            <p v-if="port.info === 'keyboard'">
              Simula la entrada del lector usando el teclado
            </p>
            <template v-else>
              <template v-for="(val, prop) in port.port_type">
                <p>
                  <span class="capitalize">{{ prop.replace("_", " ") }}</span
                  >: {{ val }}
                </p>
              </template>
            </template>
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
                ? enterScanning()
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
        <div class="flex flex-row gap-2">
          <el-button
            class="!px-1 !py-0."
            text
            type="info"
            :_disabled="scanning"
            @click="
              () =>
                scanning
                  ? raiseError('Pausa el escáner para poder volver atrás')
                  : goBack()
            "
          >
            <el-icon class="el-icon--left !mr-0.5"><IconChevronLeft /></el-icon>
            Volver
          </el-button>
          <h1
            class="text-2xl font-semibold"
            :class="scanning ? '' : 'text-rose-400'"
          >
            {{ scanning ? "Escaneando..." : "Escáner pausado" }}
          </h1>
        </div>

        <div>
          <el-button text @click="dataDialogOpen = true" class="mr-2"
            >Ver datos completos</el-button
          >
          <CSVExportButton :scanned="scanned" />
        </div>
      </div>

      <!--* Data Dialog -->
      <el-dialog v-model="dataDialogOpen" title="Datos del censo">
        <VueJsonPretty :data="userData" virtual :height="600" />
      </el-dialog>

      <el-card
        v-for="(att, idx) in scanned"
        :key="att.data.tui ?? ''"
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

        <div
          v-if="att.new"
          class="inline-block h-3 mr-2 aspect-square w-auto bg-green-400 rounded-full"
        ></div>
        <h3 class="inline font-bold text-lg mr-2" v-if="att.data.full_name">
          <span v-if="att.data.nickname" class="italic"
            >({{ att.data.nickname }})&nbsp;</span
          >{{ att.data.full_name }}
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
              class="!dark:text-white mr-2"
              type="info"
              >{{
                tagName(
                  checkKey,
                  att.data.pronouns as "F" | "M" | "N" | null | undefined
                )
              }}</el-tag
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
          @click="stopScan()"
          v-if="scanning"
        >
          Parar Escáner
          <el-icon class="el-icon--right"><IconPlayerPause /></el-icon>
        </el-button>

        <el-button
          type="success"
          class="w-full"
          :disabled="!(filePath && !fileError && selectedPort !== undefined)"
          @click="restartScan()"
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
import { useDark } from "@vueuse/core";
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
  IconChevronLeft,
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
import { PortService } from "./ports";
import { raiseError } from "./error";
import { tryit } from "radash";
import { ElMessage } from "element-plus";

const { appContext } = getCurrentInstance()!;

useDark();

const configured = ref(false);
console.info("set configured", configured);

const dataDialogOpen = ref(false);
console.info("set dataDialogOpen", dataDialogOpen);

PortService.fetchPorts();
console.info("_set fetchPorts");

const ports = PortService.usePorts;
const selectedPort = ref();
console.info("set selectedPort", selectedPort);

const userData: Ref<Attendant[] | undefined> = ref();
console.info("set userData", userData);

const filePath = ref();
const processingFile = ref(true);
const fileError = ref();
const openFile = async () => {
  console.info("openFile enter");

  processingFile.value = false;

  //* Pick
  const [error, path] = await tryit(open)({
    multiple: false,
    filters: [
      {
        name: "Hoja CSV",
        extensions: ["csv"],
      },
    ],
    defaultPath: await homeDir(),
  });
  if (error) {
    raiseError(error);
    return;
  }
  if (!path) {
    ElMessage.info(
      {
        message: "Archivo anterior conservado",
      },
      appContext
    );
    return;
  }
  if (Array.isArray(path)) {
    raiseError("Sólo un archivo CSV soportado");
    return;
  }
  filePath.value = path;
  fileError.value = false;
  processingFile.value = true;

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

const availableNames = computed(() =>
  userData.value
    ?.filter((usr) => !usr.raw.tui)
    .map((usr) => ({ value: usr.raw.full_name, label: usr.raw.full_name }))
);

const input = ref("");

type Checks = keyof AttendantChecks;

const scanned = ref(
  [] as {
    data: Partial<RawAttendant>;
    new: boolean;
    checks: AttendantChecks | undefined;
    time: Date;
    dialogOpen: boolean;
  }[]
);

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
      has_own_vote: "",
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
      has_own_vote: "",
    }[key])
);

const updateAttendant = (idx: number) => {
  console.info("updateAttendant");

  const fdUser = userData.value?.find(
    (usr) => usr.raw.full_name === scanned.value[idx].data.full_name
  );

  if (!fdUser) {
    console.error("Full name not found", scanned.value[idx].data.full_name);
    raiseError(
      `No se ha podido encontrar a ningún usuario con el nombre ${scanned.value[idx].data.full_name} para actualizar`
    );
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

const scanning = PortService.useScanning();

const getPort = () => ports.value[selectedPort.value];

const enterScanning = async () => {
  console.info("go scanning");
  configured.value = true;

  console.info("Mounting port", ports.value[selectedPort.value]);
  const installSuccess = await getPort().install(input);
  if (!installSuccess) return;
  getPort().listen(userData, input, scanned);
};

const stopScan = () => getPort().stop();

const restartScan = () => getPort().restart();

const goBack = () => {
  if (scanning.value) {
    raiseError("Tried to unlisten port while scanning");
    return;
  }
  getPort().unlisten();
  getPort().uninstall();
  configured.value = false;
  console.log("back to configure");
}; // I know that it is already paused
</script>
