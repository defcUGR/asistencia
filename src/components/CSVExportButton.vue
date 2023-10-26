<template>
  <el-dropdown
    split-button
    type="primary"
    @click="exportCSV"
    @command="exportExtra"
  >
    <el-icon class="el-icon--left"><IconFileTypeCsv /></el-icon>
    Exportar CSV
    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item :icon="IconListDetails" command="lime_survey"
          >LimeSurvey</el-dropdown-item
        >
      </el-dropdown-menu>
    </template>
  </el-dropdown>
  <Teleport to="body">
    <div class="fixed w-full top-4 flex justify-center z-50">
      <Transition name="progress">
        <el-card shadow="always" v-if="exportCSVOngoing" class="relative w-72">
          <div class="!flex !flex-row gap-2" v-if="!exportCSVDone">
            <IconFileExport
              class="block h-10 w-10 flex-shrink-0"
            ></IconFileExport>
            <div class="w-full">
              <strong class="text-lg font-bold">Exportando datos</strong>
              <el-progress class="mt-0.5" :percentage="exportCSVProgress" />
            </div>
          </div>
          <div class="!flex !flex-row gap-2" v-else>
            <IconCheck
              class="block h-10 w-10 flex-shrink-0 text-green-400"
            ></IconCheck>
            <div class="w-full">
              <strong class="text-lg font-bold text-green-400"
                >Exportando datos</strong
              >
              <el-progress class="mt-0.5" :percentage="100" status="success" />
            </div>
          </div>
        </el-card>
      </Transition>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import {
  IconFileTypeCsv,
  IconCheck,
  IconFileExport,
  IconListDetails,
} from "@tabler/icons-vue";
import { invoke } from "@tauri-apps/api";
import { save } from "@tauri-apps/api/dialog";
import { listen } from "@tauri-apps/api/event";
import { homeDir } from "@tauri-apps/api/path";
import { ElMessage } from "element-plus";
import { AttendantChecks } from "../../src-tauri/bindings/AttendantChecks";
import { RawAttendant } from "../../src-tauri/bindings/RawAttendant";
import { LimeSurveyExportItem } from "../../src-tauri/bindings/LimeSurveyExportItem";
import { raiseError } from "../error";

const { appContext } = getCurrentInstance()!;

const props = defineProps<{
  scanned: {
    data: Partial<RawAttendant>;
    new: boolean;
    checks: AttendantChecks | undefined;
    time: Date;
    dialogOpen: boolean;
    stillPresent: boolean;
  }[];
}>();
const { scanned } = toRefs(props);

// TODO Loading state for CSV export button
const exportCSVProgress = ref(0);
const exportCSVOngoing = ref(false);
const exportCSVDone = ref(false);

const checkCompleteData = () => {
  if (!scanned.value.every((sc) => sc.data.full_name)) {
    raiseError(
      "Selecciona una persona del censo para las nuevas TUIs antes de exportar"
    );
    return false;
  }
  return true;
};

const getExportPath = async () => {
  const filePath = await save({
    title: "Exportar datos como CSV",
    defaultPath: await homeDir(),
    filters: [
      {
        name: "CSV",
        extensions: ["csv"],
      },
    ],
  });
  if (!filePath) {
    ElMessage.warning(
      {
        message: "Exportar a CSV cancelado",
      },
      appContext
    );
    return;
  }
  return filePath;
};

const exportCSV = async () => {
  if (!checkCompleteData()) return;

  const filePath = await getExportPath();
  if (!filePath) return;

  exportCSVDone.value = false;
  exportCSVOngoing.value = true;

  let counter = 0;
  const unlisten = await listen("export_csv_progress", () => {
    exportCSVProgress.value = (++counter * 100) / scanned.value.length;
  });
  invoke("export_csv", {
    filePath,
    data: scanned.value.map((sc) => ({
      new: sc.new,
      time: sc.time,
      tui: sc.data.tui,
      full_name: sc.data.full_name,
    })),
  })
    .catch((error) => raiseError("Error exportando CSV: " + error))
    .then(() => {
      unlisten();
      exportCSVDone.value = true;
      setTimeout(() => (exportCSVOngoing.value = false), 2000);
    });
};

const exportExtra = async (command: "lime_survey") => {
  if (!["lime_survey"].includes(command)) {
    raiseError("Plataforma no soportada para exportar datos");
    return;
  }

  if (!checkCompleteData()) return;

  const filePath = await getExportPath();
  if (!filePath) return;

  exportCSVDone.value = false;
  exportCSVOngoing.value = true;

  let counter = 0;
  const unlisten = await listen("export_" + command + "_progress", () => {
    exportCSVProgress.value = (++counter * 100) / scanned.value.length;
  });

  const data = (() => {
    switch (command) {
      case "lime_survey":
        return scanned.value
          .filter(
            (sc) =>
              sc.stillPresent &&
              (sc.checks!.has_own_vote ||
                sc.checks!.is_delegado ||
                (sc.checks!.is_subdelegado &&
                  (
                    scanned.value.find(
                      (del) =>
                        del.checks?.is_delegado &&
                        del.data.degree === sc.data.degree &&
                        del.data.course === sc.data.course &&
                        del.data.group === sc.data.group
                    ) ?? { checks: { has_own_vote: true } }
                  ).checks?.has_own_vote))
          )
          .map(
            (sc, idx) =>
              ({
                tid: idx,
                firstname: sc.data.name,
                lastname: sc.data.last_name,
                email: sc.data.email,
              } as LimeSurveyExportItem)
          );
    }
  })();

  invoke("export_" + command, {
    filePath,
    data,
  })
    .catch((e) => raiseError(e))
    .then(() => {
      unlisten();
      exportCSVDone.value = true;
      setTimeout(() => (exportCSVOngoing.value = false), 2000);
    });
};
</script>

<style>
.progress-leave-to,
.progress-enter-from {
  transform: translateY(-180px);
  opacity: 0;
}
</style>
